use aws_sdk_s3::{presigning::PresigningConfig, primitives::ByteStream};
use axum::{
    debug_handler,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::{
    extract::Multipart,
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use common::{get_parsed_cargo, patch_dependencies, targets::TargetFramework};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::File,
    io::Read,
    process::Command,
    thread,
    time::{self, Duration},
};
use tempfile::{tempdir, TempDir};
use tokio::{io::AsyncWriteExt, time::timeout};
use utoipa::ToSchema;
use uuid::{NoContext, Timestamp, Uuid};
use zip::ZipArchive;

use crate::{app_error::AppError, jwt, state::AppState, zip::zip_path};

pub static CIRCUITS_BUCKET_NAME: Lazy<Option<String>> =
    Lazy::new(|| env::var("circuits_bucket").ok());

pub static COMPILE_LAMBDA_ARN: Lazy<Option<String>> =
    Lazy::new(|| env::var("compile_lambda_arn").ok());

#[derive(Deserialize, ToSchema)]
#[allow(unused)]
pub struct CompileToIrPayload {
    /// Zipped file. Requires `cargo.toml` and source files
    #[schema(format = Binary)]
    zip_file: Vec<u8>,

    /// Name of the example artifact to compile. For example, `square_root` for `cargo run --example square_root`
    example_artifact: Option<String>,

    /// Name of the repository to store the circuit in
    repo_name: String,
}

#[derive(Serialize, ToSchema, Clone)]
pub struct CompileToIrResponse {
    repo_name: String,
    circuit_version: String,
}

/// Compile source to IR
#[utoipa::path(post,
    tag="IR",
    path="/v1/ir",
    request_body(
        content = CompileToIrPayload,
        content_type = "multipart/form-data",
    ),
    responses(
        (status = 202, description = "Initiated compilation", body = CompileToIrResponse),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 400, description = "Bad request body", body = String),
        (status = 404, description = "Invalid circuit id", body = String),
    ),
    security(
        ("token" = [])
    )
)]
#[debug_handler]
pub async fn compile_to_ir(
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    State(app_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut example_artifact = None;
    let mut repo_name = None;
    let mut description = None;

    let zip_dir = tempdir()?;
    let mut zip_file_path = None;

    let timestamp_now = Timestamp::now(NoContext);
    let uuid_now = Uuid::new_v7(timestamp_now).simple().to_string();
    // Need this because the original string will get moved to coroutine
    let uuid_now_clone = uuid_now.clone();

    let token = bearer
        .token()
        .to_string()
        .trim_start_matches("Bearer ")
        .to_string();
    let user_data = jwt::get_user_claims(&token)?;

    while let Some(field) = multipart.next_field().await? {
        let name = field
            .name()
            .ok_or(anyhow::anyhow!("Failed to get field name"))?;

        match name {
            "zip_file" => {
                let file_name = field
                    .file_name()
                    .ok_or(anyhow::anyhow!("Failed to get file name"))?;
                let path = zip_dir.path().join(file_name);
                let data = field.bytes().await?;
                let mut file = tokio::fs::File::create(&path).await?;
                file.write_all(&data).await?;
                zip_file_path = Some(path);
            }
            "example_artifact" => {
                let data = field.text().await?;
                example_artifact = Some(data);
            }
            "repo_name" => {
                let data = field.text().await?;

                let circuits_bucket_name = CIRCUITS_BUCKET_NAME.as_ref().ok_or_else(|| {
                    AppError::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Could not get circuits bucket name".to_string(),
                    )
                })?;

                repo_name = Some(data);
            }
            "description" => {
                let data = field.text().await?;
                description = Some(data);
            }
            _ => {
                return Err(AppError::new(
                    StatusCode::BAD_REQUEST,
                    "Unknown field or wrong content type".to_string(),
                ));
            }
        }
    }

    let repo_name = repo_name.ok_or_else(|| {
        AppError::new(
            StatusCode::BAD_REQUEST,
            "No repo name found in the request".to_string(),
        )
    })?;

    // Need this because the original string will get moved to coroutine
    let repo_name_clone = repo_name.clone();

    let zipped_path = zip_file_path.ok_or_else(|| {
        AppError::new(
            StatusCode::BAD_REQUEST,
            "No zip file found in the request".to_string(),
        )
    })?;

    if !zipped_path.is_file() {
        return Err(AppError::new(
            StatusCode::BAD_REQUEST,
            "Uploaded file is not found or not a file".to_string(),
        ));
    }

    let mut retries = 0;
    while !is_file_ready(&zipped_path)? {
        if retries > 5 {
            return Err(AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to process the file".to_string(),
            ));
        }
        retries += 1;
    }

    let zip_file = std::fs::File::open(&zipped_path).map_err(|e| {
        tracing::error!("Failed to open zip file: {e}");
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to open zip file".to_string(),
        )
    })?;

    let mut archive = ZipArchive::new(zip_file).map_err(|e| {
        tracing::error!("Failed to read zip archive: {e}");
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to read zip archive".to_string(),
        )
    })?;

    let unzipped_dir = tempdir()?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let out_path = unzipped_dir
            .path()
            .join(file.enclosed_name().ok_or_else(|| {
                AppError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Unsafe path encountered in zip file".to_string(),
                )
            })?);

        if file.is_dir() {
            std::fs::create_dir_all(&out_path)?;
        } else {
            let file_parent = out_path.parent().ok_or_else(|| {
                AppError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Invalid parent directory encountered when unpacking zip".to_string(),
                )
            })?;

            std::fs::create_dir_all(file_parent)?;

            let mut outfile = std::fs::File::create(&out_path)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    let circuits_bucket_name = CIRCUITS_BUCKET_NAME.as_ref().ok_or_else(|| {
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Circuit bucket name not found".to_string(),
        )
    })?;

    let zipped_source = zip_path(unzipped_dir.path()).map_err(|e| {
        tracing::error!("Failed to zip source: {e}");
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal error".to_string(),
        )
    })?;

    let mut zipped_source = File::open(&zipped_source.1).map_err(|e| {
        tracing::error!("Failed to open zip file: {e}");
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal error".to_string(),
        )
    })?;

    let mut source_buffer = Vec::new();

    zipped_source.read_to_end(&mut source_buffer).map_err(|e| {
        tracing::error!("Failed to read zip file: {e}");
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal error".to_string(),
        )
    })?;

    app_state
        .get_s3_client()
        .put_object()
        .bucket(circuits_bucket_name)
        .key(format!(
            "{}/{repo_name}/{uuid_now}/source.zip",
            user_data.claims.sub
        ))
        .body(source_buffer.into())
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to upload to S3: {e}");
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error".to_string(),
            )
        })?;

    app_state
        .get_s3_client()
        .put_object()
        .bucket(circuits_bucket_name)
        .key(format!(
            "{}/{repo_name}/{uuid_now}/description.txt",
            user_data.claims.sub
        ))
        .body(ByteStream::from(
            description.unwrap_or_default().into_bytes(),
        ))
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to upload to S3: {e}");
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error".to_string(),
            )
        })?;

    tracing::info!("Stored source in S3.");

    tokio::spawn(async move {
        let result = timeout(
            Duration::from_secs(2 * 60),
            compile_and_upload(
                app_state,
                unzipped_dir,
                bearer.token().to_string(),
                repo_name,
                uuid_now.clone(),
                example_artifact,
            ),
        )
        .await;

        match result {
            Ok(Ok(())) => tracing::info!("Build and upload completed successfully."),
            Ok(Err(e)) => tracing::error!("Background build process failed: {:#?}", e),
            Err(_) => tracing::error!("Background build process timed out."),
        }
    });

    Response::builder()
        .status(StatusCode::CREATED)
        .body(
            serde_json::to_string(&CompileToIrResponse {
                repo_name: repo_name_clone,
                circuit_version: uuid_now_clone,
            })
            .map_err(AppError::from)?,
        )
        .map_err(AppError::from)
}

async fn compile_and_upload(
    app_state: AppState,
    unzipped_dir: TempDir,
    bearer_token: String,
    repo_name: String,
    circuit_version: String,
    example_artifact: Option<String>,
) -> Result<(), String> {
    let unzipped_dir_path = unzipped_dir.path();

    let circuits_bucket_name = CIRCUITS_BUCKET_NAME
        .as_ref()
        .ok_or_else(|| "Circuits bucket name not found".to_string())?;

    let token = bearer_token.trim_start_matches("Bearer ").to_string();
    let data =
        jwt::get_user_claims(&token).map_err(|e| format!("Failed to get user claims: {e}"))?;

    let mut parsed_cargo = get_parsed_cargo(&unzipped_dir_path.join("Cargo.toml"))?;

    let dependencies = parsed_cargo
        .get("dependencies")
        .ok_or("No dependencies found in `Cargo.toml`")?;

    let target_framework = if dependencies.get("plonky2").is_some() {
        TargetFramework::Plonky2
    } else {
        return Err("No supported target framework found".into());
    };

    tracing::info!("Found target framework: {target_framework}");

    let parsed_cargo_table = parsed_cargo
        .as_table_mut()
        .ok_or("Root of Cargo.toml is not a table")?;

    patch_dependencies(
        &unzipped_dir_path.join("Cargo.toml"),
        &target_framework,
        parsed_cargo_table,
    )?;

    let output = Command::new("cargo")
        .arg("build")
        .args(
            example_artifact
                .as_ref()
                .map(|name| vec!["--example".to_string(), name.to_string()])
                .unwrap_or_default(),
        )
        .current_dir(unzipped_dir_path)
        .output()
        .map_err(|e| format!("Failed to build project: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "Failed to build project: {}",
            String::from_utf8(output.stderr)
                .unwrap_or_else(|_| "Failed to convert stderr to string".to_string())
        ));
    }

    tracing::info!("Project built successfully");

    let debug_folder = unzipped_dir_path.join("target/debug");

    let executable_path = if let Some(name) = example_artifact {
        debug_folder.join("examples").join(name)
    } else {
        // STUB: this only supports examples for now
        debug_folder
    };

    let mut executable =
        File::open(&executable_path).map_err(|e| format!("Failed to open executable file: {e}"))?;

    let mut executable_buffer = Vec::new();
    executable
        .read_to_end(&mut executable_buffer)
        .map_err(|e| format!("Failed to read executable file: {e}"))?;

    app_state
        .get_s3_client()
        .put_object()
        .bucket(circuits_bucket_name)
        .key(format!(
            "{}/{repo_name}/{circuit_version}/executable",
            data.claims.sub
        ))
        .body(executable_buffer.into())
        .send()
        .await
        .map_err(|e| format!("Failed to upload to S3: {e}"))?;

    tracing::info!("Stored executable in S3.");

    // Give lambda presigned url as it doesn't have direct access to S3 due to executing arbitrary code
    let executable_presigned_url = app_state
        .get_s3_client()
        .get_object()
        .bucket(circuits_bucket_name)
        .key(format!(
            "{}/{repo_name}/{circuit_version}/executable",
            data.claims.sub
        ))
        .presigned(
            PresigningConfig::expires_in(Duration::from_secs(60 * 5))
                .map_err(|e| format!("Failed to create pre-signed URL config: {e}"))?,
        )
        .await
        .map_err(|e| format!("Failed to generate pre-signed URL: {e}"))?;

    let compile_lambda_arn = COMPILE_LAMBDA_ARN
        .as_ref()
        .ok_or_else(|| "Compile lambda ARN not found".to_string())?;

    let payload = common::CompileLambdaPayload {
        presigned_executable_url: executable_presigned_url.uri().to_string(),
    };

    let res = app_state
        .get_lambda_client()
        .invoke()
        .function_name(compile_lambda_arn)
        .payload(aws_sdk_lambda::primitives::Blob::new(
            serde_json::to_string(&payload)
                .map_err(|e| format!("Failed to serialize lambda payload to string: {e}"))?
                .into_bytes(),
        ))
        .send()
        .await
        .map_err(|e| format!("Failed to invoke Lambda function: {e:#?}"))?;

    if res.status_code() != 200 {
        return Err(format!(
            "Failed to invoke Lambda function: {:#?}",
            res.payload.as_ref().map_or_else(
                || "No payload".to_string().into(),
                |p| String::from_utf8_lossy(p.as_ref())
            )
        ));
    }

    let json_ir_bytes = res
        .payload
        .ok_or_else(|| "No payload".to_string())?
        .as_ref()
        .to_vec();

    app_state
        .get_s3_client()
        .put_object()
        .bucket(circuits_bucket_name)
        .key(format!(
            "{}/{repo_name}/{circuit_version}/ir.json",
            data.claims.sub
        ))
        .body(ByteStream::from(json_ir_bytes))
        .content_type("application/json")
        .send()
        .await
        .map_err(|e| format!("Failed to upload to S3: {e}"))?;

    Ok(())
}

fn is_file_ready(zip_path: &std::path::Path) -> std::io::Result<bool> {
    let initial_size = std::fs::metadata(zip_path)?.len();

    thread::sleep(time::Duration::from_secs(1));

    let final_size = std::fs::metadata(zip_path)?.len();
    Ok(initial_size == final_size)
}

#[derive(Serialize, ToSchema, Clone)]
pub struct GetIrResponse {
    /// IR as JSON
    ir: String,
}

/// Get IR as JSON
#[utoipa::path(get,
    tag="IR",
    path="/v1/ir/{repo_name}/{circuit_version}",
    responses(
        (status = 200, description = "Got IR", body = GetIrResponse),
        (status = 202, description = "Compilation still in progress", body = String),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 404, description = "Invalid circuit id", body = String),
    ),
    security(
        ("token" = [])
    )
)]
#[debug_handler]
pub async fn get_ir(
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    State(app_state): State<AppState>,
    Path((repo_name, circuit_version)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let token = bearer
        .token()
        .to_string()
        .trim_start_matches("Bearer ")
        .to_string();
    let user_data = jwt::get_user_claims(&token)?;

    let circuits_bucket_name = CIRCUITS_BUCKET_NAME.as_ref().ok_or_else(|| {
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not get circuits bucket name".to_string(),
        )
    })?;

    let ir_result = app_state
        .get_s3_client()
        .get_object()
        .bucket(circuits_bucket_name)
        .key(format!(
            "{}/{repo_name}/{circuit_version}/ir.json",
            user_data.claims.sub
        ))
        .send()
        .await;

    if let Ok(res) = ir_result {
        let ir_bytes = res.body.collect().await.map_err(|_| {
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read IR from S3".to_string(),
            )
        })?;

        let ir_string = String::from_utf8(ir_bytes.to_vec()).map_err(|_| {
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to convert IR bytes to string".to_string(),
            )
        })?;

        return Response::builder()
            .status(StatusCode::OK)
            .body(serde_json::to_string(&GetIrResponse { ir: ir_string }).map_err(AppError::from)?)
            .map_err(AppError::from);
    }

    Response::builder()
        .status(StatusCode::ACCEPTED)
        .body("Compilation still in progress".to_string())
        .map_err(AppError::from)
}

#[derive(Serialize, ToSchema, Clone)]
pub struct IrMetadata {
    repo_name: String,
    circuit_version: String,
    name: String,
    description: String,
}

#[derive(Serialize, ToSchema, Clone)]
pub struct ListIrsMetadataResponse {
    irs: Vec<IrMetadata>,
}

/// List IRs
#[utoipa::path(get,
    tag="IR",
    path="/v1/ir/metadata/list",
    responses(
        (status = 200, description = "Got IRs", body = String),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
    ),
    security(
        ("token" = [])
    )
)]
#[debug_handler]
pub async fn list_irs_metadata(
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let token = bearer
        .token()
        .to_string()
        .trim_start_matches("Bearer ")
        .to_string();
    let user_data = jwt::get_user_claims(&token)?;

    let circuits_bucket_name = CIRCUITS_BUCKET_NAME.as_ref().ok_or_else(|| {
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not get circuits bucket name".to_string(),
        )
    })?;

    let list_objects_res = app_state
        .get_s3_client()
        .list_objects_v2()
        .bucket(circuits_bucket_name)
        .prefix(format!("{}/", user_data.claims.sub))
        .send()
        .await
        .map_err(|_| {
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to list objects in S3".to_string(),
            )
        })?;

    let directories = list_objects_res
        .common_prefixes
        .unwrap_or_default()
        .into_iter()
        .filter_map(|prefix| prefix.prefix)
        .collect::<Vec<_>>();

    let mut irs = Vec::new();

    for dir in directories {
        let list_objects_res = app_state
            .get_s3_client()
            .list_objects_v2()
            .bucket(circuits_bucket_name)
            .prefix(format!("{dir}/"))
            .send()
            .await
            .map_err(|_| {
                AppError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to list objects in S3".to_string(),
                )
            })?;

        let mut ir_metadata = IrMetadata {
            repo_name: dir.split('/').last().unwrap().to_string(),
            circuit_version: String::new(),
            name: String::new(),
            description: String::new(),
        };

        for object in list_objects_res.contents.unwrap_or_default() {
            let key = object.key.unwrap();

            if key.ends_with("ir.json") {
                ir_metadata.circuit_version = key.split('/').nth(1).unwrap().to_string();
            } else if key.ends_with("description.txt") {
                let description = app_state
                    .get_s3_client()
                    .get_object()
                    .bucket(circuits_bucket_name)
                    .key(key)
                    .send()
                    .await
                    .map_err(|_| {
                        AppError::new(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Failed to get object from S3".to_string(),
                        )
                    })?;

                let description = description.body.collect().await.map_err(|_| {
                    AppError::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to read object from S3".to_string(),
                    )
                })?;

                ir_metadata.description =
                    String::from_utf8(description.to_vec()).map_err(|_| {
                        AppError::new(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Failed to convert description to string".to_string(),
                        )
                    })?;
            }
        }

        irs.push(ir_metadata);
    }

    Response::builder()
        .status(StatusCode::OK)
        .body(serde_json::to_string(&irs).map_err(AppError::from)?)
        .map_err(AppError::from)
}
