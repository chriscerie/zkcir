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

fn main() {
    let current_dir = env::current_dir().expect("Failed to get current directory");

    let args = get_args();

    let pb = get_new_pb(3_u64, "Running", ": cargo");

    let temp_dir = tempdir().expect("Failed to create temp dir");

    pb.println(format!(
        "{} temp ({})",
        get_formatted_left_output_green("Created"),
        temp_dir.path().display()
    ));
    pb.inc(1);

    let num_copied_files =
        copy_to(&current_dir, temp_dir.path()).expect("Failed to copy contents to temp dir");

    pb.println(format!(
        "{} contents to temp ({num_copied_files} files)",
        get_formatted_left_output_green("Copied")
    ));
    pb.inc(1);

    if let Some(example) = args.example {
        let output = Command::new("cargo")
            .arg("run")
            .arg("--example")
            .arg(&example)
            .current_dir(current_dir) // Set the current directory for the command
            .output()
            .expect("Failed to execute `cargo run`");

        pb.println(format!(
            "{} cargo run",
            get_formatted_left_output_green("Finished")
        ));
        pb.inc(1);

        if output.status.success() {
            pb.set_style(
                ProgressStyle::default_bar()
                    .template(&format!("{{msg}} ir with `{example}`"))
                    .unwrap(),
            );
            pb.finish_with_message(get_formatted_left_output_green("Emitted"));
        } else {
            println!("cargo run failed");
        }
    }
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

fn get_new_pb(
    length: u64,
    progress_message_left: &str,
    progress_message_right: &str,
) -> ProgressBar {
    let pb = ProgressBar::new(length);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "{} [{{bar:40}}] {{pos}}/{{len}}{progress_message_right}",
                get_formatted_left_output_blue(progress_message_left)
            ))
            .unwrap()
            .progress_chars("=> "),
    );
    pb
}
