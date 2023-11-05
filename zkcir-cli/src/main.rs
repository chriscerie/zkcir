use core::fmt;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    env::{self},
    fs::{self},
    io,
    path::Path,
    process::Command,
};
use tempfile::tempdir;
use walkdir::{DirEntry, WalkDir};

use args::get_args;

mod args;

const START_DISCRIMINATOR: &str = "<ZKCIR_JSON_START>";
const END_DISCRIMINATOR: &str = "<ZKCIR_JSON_END>";

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
    pub fn replace_deps(&self, path: &Path, pb: &ProgressBar) {
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
                        .expect("Failed to execute `cargo add`");

                    pb.println(format!(
                        "{} dependency ({package})",
                        get_formatted_left_output_green("Replaced")
                    ));
                    pb.inc(1);
                }
            }
        }
    }
}

fn main() {
    let current_dir = env::current_dir().expect("Failed to get current directory");

    let args = get_args();

    let pb = get_new_pb(3_u64, "Running");

    pb.set_message(": cargo");
    let parsed_cargo = get_parsed_cargo();

    let dependencies = parsed_cargo
        .get("dependencies")
        .expect("No dependencies found in `Cargo.toml`");

    pb.println(format!(
        "{} {}",
        get_formatted_left_output_green("Found"),
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
        get_formatted_left_output_green("Found"),
        target_framework
    ));
    pb.inc(1);

    pb.set_message(": temp dir");
    let temp_dir = tempdir().expect("Failed to create temp dir");

    pb.println(format!(
        "{} temp ({})",
        get_formatted_left_output_green("Created"),
        temp_dir.path().display()
    ));
    pb.inc(1);

    pb.set_message(": copy contents");
    let num_copied_files =
        copy_to(&current_dir, temp_dir.path()).expect("Failed to copy contents to temp dir");

    pb.println(format!(
        "{} contents to temp ({num_copied_files} files)",
        get_formatted_left_output_green("Copied")
    ));
    pb.inc(1);

    // If the root temp dir contains a workspace `Cargo.toml`, cargo would fail to build unless there is a
    // `[workspace]` in this `Cargo.toml`
    if parsed_cargo.get("workspace").is_none() {
        pb.inc_length(1);
        pb.set_message(": add [workspace]");

        let mut toml_string =
            toml::to_string(&parsed_cargo).expect("Failed to serialize `Cargo.toml`");

        let prepend_chars = "[workspace]\n";
        toml_string.insert_str(0, prepend_chars);

        fs::write(temp_dir.path().join("Cargo.toml"), toml_string)
            .expect("Failed to add `[workspace]` to `Cargo.toml");

        pb.println(format!(
            "{} empty `[workspace]` ({})",
            get_formatted_left_output_green("Added"),
            temp_dir
                .path()
                .join("Cargo.toml")
                .to_str()
                .expect("Failed to get `Cargo.toml`")
        ));
        pb.inc(1);
    }

    target_framework.replace_deps(temp_dir.path(), &pb);

    let mut output = None;

    if let Some(example) = args.example {
        pb.inc_length(1);

        pb.set_message(": run".to_string());
        output = Some(
            Command::new("cargo")
                .arg("run")
                .args(["--example", &example])
                .current_dir(temp_dir.path())
                .output()
                .expect("Failed to execute `cargo run`"),
        );

        pb.println(format!(
            "{} cargo run with `{example}",
            get_formatted_left_output_green("Finished")
        ));
        pb.inc(1);
    }

    if let Some(output) = output {
        if !output.status.success() {
            panic!(
                "cargo run failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        pb.set_style(ProgressStyle::default_bar().template("{msg} ir").unwrap());
        pb.finish_with_message(get_formatted_left_output_green("Emitted"));
        let output_str =
            String::from_utf8(output.stdout).expect("Failed to convert output to string");

        let start_byte = output_str
            .find(START_DISCRIMINATOR)
            .expect("Start discriminator not found")
            + START_DISCRIMINATOR.len();

        let end_byte = output_str
            .find(END_DISCRIMINATOR)
            .expect("End discriminator not found");

        let json_string = &output_str[start_byte..end_byte].trim();

        println!("{}", json_string);
    } else {
        println!("Could not find circuit to run");
    }
}

fn get_parsed_cargo() -> toml::Value {
    let contents = fs::read_to_string("Cargo.toml").expect("Could not find `Cargo.toml`");

    contents
        .parse::<toml::Value>()
        .expect("Failed to parse `Cargo.toml`")
        .to_owned()
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

fn get_formatted_left_output_green(output: &str) -> String {
    let green_bold = "\x1b[1;32m";
    let reset = "\x1b[0m";

    format!("{green_bold}{:>12}{reset}", output)
}

fn get_formatted_left_output_blue(output: &str) -> String {
    let blue_bold = "\x1b[1;36m";
    let reset = "\x1b[0m";

    format!("{blue_bold}{:>12}{reset}", output)
}

fn get_new_pb(length: u64, progress_message_left: &str) -> ProgressBar {
    let pb = ProgressBar::new(length);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "{} [{{bar:40}}] {{pos}}/{{len}}{{msg}}",
                get_formatted_left_output_blue(progress_message_left)
            ))
            .unwrap()
            .progress_chars("=> "),
    );
    pb
}
