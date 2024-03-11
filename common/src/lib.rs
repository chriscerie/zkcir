#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{fs, path::Path};
use targets::TargetFramework;
use toml::map::Map;
use toml::Value;

pub mod targets;

#[derive(Serialize, Deserialize)]
pub struct CompileLambdaPayload {
    pub presigned_executable_url: String,
}

/// # Errors
/// This function will return an error if `path` does not already exist
pub fn get_parsed_cargo(path: &Path) -> Result<toml::Value, String> {
    let contents = fs::read_to_string(path).map_err(|_| "Failed to find required `Cargo.toml`")?;

    Ok(contents
        .parse::<toml::Value>()
        .map_err(|e| format!("Failed to parse `Cargo.toml`: {e}"))?
        .clone())
}

/// # Errors
pub fn patch_dependencies(
    cargo_toml_path: &PathBuf,
    target_framework: &TargetFramework,
    parsed_cargo_table: &mut Map<String, Value>,
) -> Result<(), String> {
    let patch_table = parsed_cargo_table
        .entry("patch")
        .or_insert(toml::Value::Table(Map::default()))
        .as_table_mut()
        .ok_or("Expected `patch` to be a table")?;

    let crates_io_table = patch_table
        .entry("crates-io")
        .or_insert(toml::Value::Table(Map::default()))
        .as_table_mut()
        .ok_or("Expected `crates-io` to be a table")?;

    for target_dependency in target_framework.dependencies() {
        for dependency_name in target_dependency.dependency_names {
            let dependency_table = crates_io_table
                .entry(dependency_name)
                .or_insert(toml::Value::Table(Map::default()))
                .as_table_mut()
                .ok_or("Expected dependency entry to be a table")?;

            dependency_table.insert(
                "git".to_string(),
                toml::Value::String(target_dependency.git_url.clone()),
            );
        }
    }

    fs::write(
        cargo_toml_path,
        toml::to_string(&parsed_cargo_table)
            .map_err(|e| format!("Failed to serialize `Cargo.toml`: {e}"))?,
    )
    .map_err(|e| format!("Failed to add `[workspace]` to `Cargo.toml: {e}"))?;

    tracing::info!("Patched dependencies");

    Ok(())
}
