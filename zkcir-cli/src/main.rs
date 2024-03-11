use clap::Parser;
use common::{get_parsed_cargo, targets::TargetFramework};
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    env::{self},
    fs::{self, File},
    io::{self, Write},
    path::Path,
    process::{self, Command},
};
use tempfile::tempdir;
use terminal::{create_new_pb, get_formatted_left_output, OutputColor};
use toml::Value;
use walkdir::{DirEntry, WalkDir};
use zkcir::{ir::Cir, END_DISCRIMINATOR, START_DISCRIMINATOR};

use args::Args;

mod args;
mod terminal;

fn start(current_dir: &Path, args: &Args, pb: &ProgressBar) -> Result<(), String> {
    let start_time = std::time::Instant::now();

    if !args.json && !args.source {
        pb.abandon();
        return Err("Either `--json` and/or `--source` must be enabled".to_string());
    }

    pb.set_message(": cargo");
    let mut parsed_cargo = get_parsed_cargo(Path::new("Cargo.toml"))?;

    let dependencies = parsed_cargo
        .get("dependencies")
        .ok_or("No dependencies found in `Cargo.toml`")?;

    pb.println(format!(
        "{} {}",
        get_formatted_left_output("Found", OutputColor::Green),
        current_dir.join("Cargo.toml").display()
    ));

    pb.set_message(": target framework");
    let target_framework = if dependencies.get("plonky2").is_some() {
        TargetFramework::Plonky2
    } else if dependencies.get("halo2").is_some() {
        TargetFramework::Halo2
    } else {
        panic!("No supported target framework found");
    };

    pb.println(format!(
        "{} target framework ({})",
        get_formatted_left_output("Found", OutputColor::Green),
        target_framework
    ));
    pb.inc(1);

    pb.set_message(": temp dir");
    let temp_dir = tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;

    pb.println(format!(
        "{} {}",
        get_formatted_left_output("Created", OutputColor::Green),
        temp_dir.path().display()
    ));
    pb.inc(1);

    pb.set_message(": copy contents");
    let num_copied_files = copy_to(current_dir, temp_dir.path())
        .map_err(|e| format!("Failed to copy contents to temp dir: {}", e))?;

    pb.println(format!(
        "{} {num_copied_files} files to {}",
        get_formatted_left_output("Cloned", OutputColor::Green),
        temp_dir.path().display()
    ));
    pb.inc(1);

    let parsed_cargo_table = parsed_cargo
        .as_table_mut()
        .ok_or("Root of Cargo.toml is not a table")?;

    let patch_table = parsed_cargo_table
        .entry("patch")
        .or_insert(Value::Table(Default::default()))
        .as_table_mut()
        .ok_or("Expected `patch` to be a table")?;

    let crates_io_table = patch_table
        .entry("crates-io")
        .or_insert(Value::Table(Default::default()))
        .as_table_mut()
        .ok_or("Expected `crates-io` to be a table")?;

    for target_dependency in target_framework.dependencies() {
        for dependency_name in target_dependency.dependency_names {
            let dependency_table = crates_io_table
                .entry(dependency_name)
                .or_insert(Value::Table(Default::default()))
                .as_table_mut()
                .ok_or("Expected dependency entry to be a table")?;

            dependency_table.insert(
                "git".to_string(),
                Value::String(target_dependency.git_url.clone()),
            );
        }
    }

    pb.println(format!(
        "{} dependencies",
        get_formatted_left_output("Patched", OutputColor::Green)
    ));
    pb.inc(1);

    // If the root temp dir contains a workspace `Cargo.toml`, cargo would fail to build unless there is a
    // `[workspace]` in this `Cargo.toml`
    if parsed_cargo_table.get("workspace").is_none() {
        pb.inc_length(1);
        pb.set_message(": add [workspace]");

        let mut toml_string = toml::to_string(&parsed_cargo)
            .map_err(|e| format!("Failed to serialize `Cargo.toml`: {}", e))?;

        let prepend_chars = "[workspace]\n";
        toml_string.insert_str(0, prepend_chars);

        // For some reason doing `parsed_cargo_table.insert("workspace".to_string(), Value::Table(Default::default()));` instead
        // doesn't work when running plonky2-examples
        fs::write(temp_dir.path().join("Cargo.toml"), toml_string)
            .map_err(|e| format!("Failed to add `[workspace]` to `Cargo.toml: {}", e))?;

        pb.println(format!(
            "{} empty `[workspace]` to {}",
            get_formatted_left_output("Added", OutputColor::Green),
            temp_dir
                .path()
                .join("Cargo.toml")
                .to_str()
                .ok_or("Failed to get `Cargo.toml`")?
        ));
        pb.inc(1);
    }

    let (subcommand, run_args) = match args.cargo_args.as_slice() {
        [first, second, rest @ ..] if first == "cargo" => (second.clone(), rest.to_vec()),
        _ => ("run".to_string(), args.cargo_args.clone()),
    };

    let default = "circuit".to_string();

    let circuit_name = if let Some(name) = &args.name {
        name
    } else if let Some(flag_index) = args
        .cargo_args
        .iter()
        .position(|arg| ["--example", "--bin"].contains(&arg.as_str()))
    {
        args.cargo_args.get(flag_index + 1).unwrap_or(&default)
    } else if let Some(flag_index) = args.cargo_args.iter().position(|arg| arg == "--package") {
        args.cargo_args.get(flag_index + 1).unwrap_or(&default)
    } else {
        "circuit"
    }
    .to_string();

    let output_dir_path = current_dir.join("zkcir_out");
    let output_cir_path_json = output_dir_path.join(&circuit_name).with_extension("json");
    let output_cir_path_source = output_dir_path.join(&circuit_name).with_extension("cir");

    fs::create_dir_all(&output_dir_path)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    if output_cir_path_json.exists() && args.json {
        if !args.allow_dirty {
            pb.abandon();
            return Err(format!("Output file ({}) already exists. Either remove it or rerun command with `--allow-dirty`", output_cir_path_json.display()));
        }

        fs::remove_file(&output_cir_path_json)
            .map_err(|e| format!("Failed to remove existing cir file: {}", e))?;
    }

    if output_cir_path_source.exists() && args.source {
        if !args.allow_dirty {
            pb.abandon();
            return Err(format!("Output file ({}) already exists. Either remove it or rerun command with `--allow-dirty`", output_cir_path_source.display()));
        }

        fs::remove_file(&output_cir_path_source)
            .map_err(|e| format!("Failed to remove existing cir file: {}", e))?;
    }

    pb.set_message(format!(": cargo {subcommand}"));
    let output = Command::new("cargo")
        .arg(&subcommand)
        .args(&run_args)
        .current_dir(temp_dir.path())
        .output()
        .map_err(|e| format!("Failed to execute `cargo {subcommand}`: {}", e))?;

    pb.println(format!(
        "{} cargo run {}",
        get_formatted_left_output("Executed", OutputColor::Green),
        &run_args.join(" ")
    ));
    pb.inc(1);

    if !output.status.success() {
        panic!(
            "cargo run failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    pb.set_message(": cir".to_string());

    let output_str = String::from_utf8(output.stdout)
        .map_err(|e| format!("Failed to convert output to string: {}", e))?;

    let start_byte_json = output_str
        .find(START_DISCRIMINATOR)
        .ok_or("Start discriminator of output not found")?
        + START_DISCRIMINATOR.len();

    let end_byte_json = output_str
        .find(END_DISCRIMINATOR)
        .ok_or("End discriminator of output not found")?;

    let json_string = &output_str[start_byte_json..end_byte_json]
        .trim()
        // Escape sequences are parsed as actual string data
        .replace("\\n", "\n")
        .replace("\\\"", "\"");

    let cir =
        Cir::from_json(json_string).map_err(|e| format!("Failed to parse json CIR: {}", e))?;

    pb.println(format!(
        "{} cir output",
        get_formatted_left_output("Parsed", OutputColor::Green)
    ));
    pb.inc(1);

    pb.set_message(": emit".to_string());

    if args.json {
        pb.inc(1);

        let mut file = File::options()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&output_cir_path_json)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        file.write_all(
            json_string
                // Escape sequences are parsed as actual string data
                .replace("\\n", "\n")
                .replace("\\\"", "\"")
                .as_bytes(),
        )
        .map_err(|e| format!("Failed to write cir to output file: {}", e))?;

        pb.println(format!(
            "{} json cir: {}",
            get_formatted_left_output("Emitted", OutputColor::Green),
            output_cir_path_json.display()
        ));
    }

    if args.source {
        pb.inc(1);

        let mut file = File::options()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&output_cir_path_source)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        file.write_all(cir.to_code_ir().as_bytes())
            .map_err(|e| format!("Failed to write cir to output file: {}", e))?;

        pb.println(format!(
            "{} source cir: {}",
            get_formatted_left_output("Emitted", OutputColor::Green),
            output_cir_path_source.display()
        ));
    }

    pb.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "{{msg}} in {}s",
                (start_time.elapsed().as_secs_f32() * 10.0).round() / 10.0
            ))
            .unwrap(),
    );
    pb.finish_with_message(get_formatted_left_output("Finished", OutputColor::Green));

    Ok(())
}

fn main() {
    let current_dir = env::current_dir().expect("Failed to get current directory");

    let pb = &create_new_pb(7, "Running");

    let _ = start(&current_dir, &Args::parse(), pb).map_err(|e| {
        pb.abandon();

        eprintln!(
            "{} {}",
            get_formatted_left_output("Error", OutputColor::Red),
            e
        );

        process::exit(1);
    });
}

fn copy_to(from: &Path, to: &Path) -> io::Result<u64> {
    let mut num_copied_files = 0;

    for entry in WalkDir::new(from) {
        let entry = entry?;

        if should_copy(&entry) {
            let path = entry.path();
            let relative_path = path.strip_prefix(from).unwrap();
            let target_path = to.join(relative_path);

            if path.is_dir() {
                fs::create_dir_all(&target_path)?;
            } else {
                fs::copy(path, &target_path)?;
                num_copied_files += 1;
            }
        }
    }

    Ok(num_copied_files)
}

fn should_copy(entry: &DirEntry) -> bool {
    !entry.path().to_string_lossy().contains("target")
}
