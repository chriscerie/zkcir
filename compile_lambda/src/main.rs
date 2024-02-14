#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]

use common::CompileLambdaPayload;
use lambda_runtime::{service_fn, Error, LambdaEvent};
#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{fs::File, io::Write, process::Command};
use tempfile::tempdir;
use zkcir::{END_DISCRIMINATOR, START_DISCRIMINATOR};

async fn compile(event: LambdaEvent<CompileLambdaPayload>) -> Result<String, Error> {
    tracing::info!("Lambda compiling");

    let zip_response = reqwest::get(event.payload.presigned_executable_url).await?;
    let bytes = zip_response.bytes().await?.to_vec();

    let unzipped_dir = tempdir()?;
    let executable_path = unzipped_dir.path().join("executable");

    let mut executable = File::create(&executable_path)?;
    executable.write_all(&bytes)?;
    executable.flush()?;
    drop(executable);

    // Gives the executable permission to run. `permissions.set_mode` is only available on Unix, but it's fine
    // because the lambda runtime is guaranteed Linux
    #[cfg(unix)]
    {
        let metadata = fs::metadata(&executable_path)?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&executable_path, permissions)?;
    }

    let output = Command::new(executable_path)
        // Enables cargo to write to the directory. Otherwise it will fail with a permission error
        .env("CARGO_HOME", "/tmp/.cargo")
        .output()
        .map_err(|e| format!("Failed to run executable: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "Failed to run executable: {}",
            String::from_utf8(output.stderr)
                .unwrap_or_else(|_| "Failed to convert stderr to string".to_string())
        )
        .into());
    }

    let output_str = String::from_utf8(output.stdout)
        .map_err(|e| format!("Failed to convert output to string: {e}"))?;

    let start_byte_json = output_str
        .find(START_DISCRIMINATOR)
        .ok_or("Start discriminator of output not found")?
        + START_DISCRIMINATOR.len();

    let end_byte_json = output_str
        .find(END_DISCRIMINATOR)
        .ok_or("End discriminator of output not found")?;

    let json_ir = &output_str[start_byte_json..end_byte_json]
        .trim()
        // Escape sequences are parsed as actual string data
        .replace("\\n", "\n")
        .replace("\\\"", "\"");

    Ok(json_ir.to_string())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().with_ansi(false).init();

    lambda_runtime::run(service_fn(compile)).await
}
