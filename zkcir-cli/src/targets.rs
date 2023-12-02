use core::fmt;
use indicatif::ProgressBar;
use std::{collections::HashSet, path::Path, process::Command};
use terminal::{get_formatted_left_output, OutputColor};
use toml::Value;

use crate::terminal;

#[derive(Debug)]
pub enum TargetFramework {
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
    pub fn replace_deps(
        &self,
        path: &Path,
        pb: &ProgressBar,
        current_deps: &Value,
    ) -> Result<(), String> {
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
                ]
                .iter()
                // This does not account for cases where the package is installed in dev but not in dependencies
                .filter(|package| current_deps.get(package).is_some())
                .collect::<HashSet<_>>();

                pb.inc_length(target_packages.len() as u64);

                for package in target_packages {
                    pb.set_message(format!(": {package}"));

                    // Must remove first because if the package is installed in both `dependencies` and `dev-dependencies`,
                    // `cargo add` will fail since each dependency must have a single canonical source
                    Command::new("cargo")
                        .args(["remove", package])
                        .arg("--dev")
                        .current_dir(path)
                        .output()
                        .map_err(|e| format!("Failed to execute `cargo remove --dev`: {}", e))?;

                    Command::new("cargo")
                        .args(["add", package])
                        .args(["--git", &git_url])
                        .current_dir(path)
                        .output()
                        .map_err(|e| format!("Failed to execute `cargo add`: {}", e))?;

                    Command::new("cargo")
                        .args(["add", package])
                        .args(["--git", &git_url])
                        .arg("--dev")
                        .current_dir(path)
                        .output()
                        .map_err(|e| format!("Failed to execute `cargo add --dev`: {}", e))?;

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
