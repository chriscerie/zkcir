use core::fmt;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    env::{self},
    fs::{self, File},
    io::{self, Write},
    path::Path,
    process::{self, Command},
};
use tempfile::tempdir;
use walkdir::{DirEntry, WalkDir};
use zkcir::{END_DISCRIMINATOR, START_DISCRIMINATOR};

use args::{get_args, Args};

mod args;

#[derive(Debug)]
enum TargetFramework {
    Plonky2,
}

impl fmt::Display for TargetFramework {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TargetFramework::Plonky2 => write!(f, "plonky2"),
        }
    }
}

impl TargetFramework {
    pub fn replace_deps(&self, path: &Path, pb: &ProgressBar) -> Result<(), String> {
        match self {
            TargetFramework::Plonky2 => {
                let git_url = "https://github.com/chriscerie/plonky2.git".to_string();
                let target_packages = [
                    "plonky2",
                    "plonky2_evm",
                    "plonky2_field",
                    "plonky2_maybe_rayon",
                    "starky",
                    "util",
                ];

                pb.inc_length(target_packages.len() as u64);

                for package in target_packages {
                    pb.set_message(format!(": {package}"));
                    Command::new("cargo")
                        .args(["add", package])
                        .args(["--git", &git_url])
                        .current_dir(path)
                        .output()
                        .map_err(|e| format!("Failed to execute `cargo add`: {}", e))?;

                    pb.println(format!(
                        "{} dependency ({package})",
                        get_formatted_left_output("Replaced", OutputColor::Green)
                    ));
                    pb.inc(1);
                }

                Ok(())
            }
        }
    }
}

fn start(current_dir: &Path, args: &Args, pb: &ProgressBar) -> Result<(), String> {
    pb.set_message(": cargo");
    let parsed_cargo = get_parsed_cargo()?;

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
        "{} contents to temp ({num_copied_files} files)",
        get_formatted_left_output("Copied", OutputColor::Green)
    ));
    pb.inc(1);

    // If the root temp dir contains a workspace `Cargo.toml`, cargo would fail to build unless there is a
    // `[workspace]` in this `Cargo.toml`
    if parsed_cargo.get("workspace").is_none() {
        pb.inc_length(1);
        pb.set_message(": add [workspace]");

        let mut toml_string = toml::to_string(&parsed_cargo)
            .map_err(|e| format!("Failed to serialize `Cargo.toml`: {}", e))?;

        let prepend_chars = "[workspace]\n";
        toml_string.insert_str(0, prepend_chars);

        fs::write(temp_dir.path().join("Cargo.toml"), toml_string)
            .map_err(|e| format!("Failed to add `[workspace]` to `Cargo.toml: {}", e))?;

        pb.println(format!(
            "{} empty `[workspace]` ({})",
            get_formatted_left_output("Added", OutputColor::Green),
            temp_dir
                .path()
                .join("Cargo.toml")
                .to_str()
                .ok_or("Failed to get `Cargo.toml`")?
        ));
        pb.inc(1);
    }

    target_framework.replace_deps(temp_dir.path(), pb)?;

    let mut output = None;

    let mut circuit_name: String = "circuit".to_string();

    if let Some(example) = &args.example {
        pb.inc_length(1);

        circuit_name = example.clone();

        pb.set_message(": run".to_string());
        output = Some(
            Command::new("cargo")
                .arg("run")
                .args(["--example", &example])
                .current_dir(temp_dir.path())
                .output()
                .map_err(|e| format!("Failed to execute `cargo run`: {}", e))?,
        );

        pb.println(format!(
            "{} cargo run with `{example}`",
            get_formatted_left_output("Finished", OutputColor::Green)
        ));
        pb.inc(1);
    }

    if let Some(output) = output {
        pb.inc_length(1);

        if !output.status.success() {
            panic!(
                "cargo run failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        pb.set_message(": cir".to_string());

        let output_str = String::from_utf8(output.stdout)
            .map_err(|e| format!("Failed to convert output to string: {}", e))?;

        let start_byte = output_str
            .find(START_DISCRIMINATOR)
            .ok_or("Start discriminator of output not found")?
            + START_DISCRIMINATOR.len();

        let end_byte = output_str
            .find(END_DISCRIMINATOR)
            .ok_or("End discriminator of output not found")?;

        let json_string = &output_str[start_byte..end_byte].trim();

        pb.println(format!(
            "{} cir output",
            get_formatted_left_output("Parsed", OutputColor::Green)
        ));
        pb.inc(1);

        pb.set_message(": emit".to_string());

        let output_dir_path = current_dir.join("zkcir_out");
        let output_cir_path = output_dir_path.join(circuit_name);

        fs::create_dir_all(&output_dir_path)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;

        if output_cir_path.exists() {
            if !args.allow_dirty {
                pb.abandon();
                return Err(format!("Output file ({}) already exists. Either remove it or rerun command with `--allow-dirty`", output_cir_path.display()));
            }

            fs::remove_file(&output_cir_path)
                .map_err(|e| format!("Failed to remove existing cir file: {}", e))?;
        }

        let mut file = File::options()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&output_cir_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        file.write_all(
            json_string
                // Escape sequences are parsed as actual string data
                .replace("\\n", "\n")
                .replace("\\\"", "\"")
                .as_bytes(),
        )
        .map_err(|e| format!("Failed to write cir to output file: {}", e))?;

        pb.set_style(
            ProgressStyle::default_bar()
                .template(&format!("{{msg}} {}", output_cir_path.display()))
                .unwrap(),
        );
        pb.finish_with_message(get_formatted_left_output("Emitted", OutputColor::Green));
    } else {
        return Err("Could not find circuit to run".to_string());
    }

    Ok(())
}

fn main() {
    let current_dir = env::current_dir().expect("Failed to get current directory");

    let pb = &get_new_pb(4_u64, "Running");

    let _ = start(&current_dir, &get_args(), pb).map_err(|e| {
        pb.abandon();

        eprintln!(
            "{} {}",
            get_formatted_left_output("Error", OutputColor::Red),
            e
        );

        process::exit(1);
    });
}

fn get_parsed_cargo() -> Result<toml::Value, String> {
    let contents =
        fs::read_to_string("Cargo.toml").map_err(|_| "Failed to find required `Cargo.toml`")?;

    Ok(contents
        .parse::<toml::Value>()
        .map_err(|e| format!("Failed to parse `Cargo.toml`: {}", e))?
        .to_owned())
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

enum OutputColor {
    Green,
    Blue,
    Red,
}

fn get_formatted_left_output(output: &str, color: OutputColor) -> String {
    let reset = "\x1b[0m";

    format!(
        "{}{:>12}{reset}",
        match color {
            OutputColor::Green => "\x1b[1;32m",
            OutputColor::Blue => "\x1b[1;36m",
            OutputColor::Red => "\x1b[1;31m",
        },
        output
    )
}

fn get_new_pb(length: u64, progress_message_left: &str) -> ProgressBar {
    let pb = ProgressBar::new(length);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "{} [{{bar:40}}] {{pos}}/{{len}}{{msg}}",
                get_formatted_left_output(progress_message_left, OutputColor::Blue)
            ))
            .unwrap()
            .progress_chars("=> "),
    );
    pb
}
