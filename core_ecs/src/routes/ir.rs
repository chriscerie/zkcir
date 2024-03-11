use aws_sdk_s3::{presigning::PresigningConfig, primitives::ByteStream, types::ObjectIdentifier};
use axum::{
    debug_handler,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use common::{get_parsed_cargo, patch_dependencies, targets::TargetFramework};
use derive_more::Display;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{env, fs::File, io::Read, process::Command, str::FromStr, time::Duration};
use tempfile::TempDir;
use tokio::time::timeout;
use utoipa::ToSchema;
use zkcir::ir::Cir;

use crate::{
    app_error::AppError,
    codecommit::get_http_clone_url,
    git::clone_repo,
    jwt::{self},
    state::AppState,
};

pub static CIRCUITS_BUCKET_NAME: Lazy<Option<String>> =
    Lazy::new(|| env::var("circuits_bucket").ok());

pub static COMPILE_LAMBDA_ARN: Lazy<Option<String>> =
    Lazy::new(|| env::var("compile_lambda_arn").ok());

#[derive(Serialize, Deserialize, ToSchema, Display, Clone)]
pub enum CompilationProgress {
    #[display(fmt = "CloningRepository")]
    NotStarted,

    #[display(fmt = "CloningRepository")]
    CloningRepository,

    #[display(fmt = "Compiling")]
    Compiling,

    #[display(fmt = "Completed")]
    Completed,
}

impl FromStr for CompilationProgress {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NotStarted" => Ok(CompilationProgress::NotStarted),
            "CloningRepository" => Ok(CompilationProgress::CloningRepository),
            "Compiling" => Ok(CompilationProgress::Compiling),
            "Completed" => Ok(CompilationProgress::Completed),
            _ => Err(()),
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct CompileToIrPayload {
    /// Name of the example artifact to compile. For example, `square_root` for `cargo run --example square_root`
    example_artifact: Option<String>,
}

/// Initiate compilation to IR
#[utoipa::path(put,
    tag="IR",
    path="/v1/ir/{owner}/{repo_name}/{commit_id}",
    request_body(
        content = CompileToIrPayload
    ),
    responses(
        (status = 202, description = "Initiated compilation", body = String),
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
    Path((owner, repo_name, commit_id)): Path<(String, String, String)>,
    Json(payload): Json<CompileToIrPayload>,
) -> Result<impl IntoResponse, AppError> {
    let token = bearer
        .token()
        .to_string()
        .trim_start_matches("Bearer ")
        .to_string();
    let user_data = jwt::get_user_claims(&token)?;

    if user_data.claims.sub != owner {
        return Err(AppError::new(
            StatusCode::UNAUTHORIZED,
            "Unauthorized".to_string(),
        ));
    }

    let circuits_bucket_name = CIRCUITS_BUCKET_NAME.as_ref().ok_or_else(|| {
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Circuit bucket name not found".to_string(),
        )
    })?;

    app_state
        .get_s3_client()
        .delete_objects()
        .bucket(circuits_bucket_name)
        .delete(
            aws_sdk_s3::types::Delete::builder()
                .objects(
                    ObjectIdentifier::builder()
                        .key(format!(
                            "{}/{repo_name}/{commit_id}/ir.json",
                            user_data.claims.sub
                        ))
                        .build()?,
                )
                .objects(
                    ObjectIdentifier::builder()
                        .key(format!(
                            "{}/{repo_name}/{commit_id}/ir.cir",
                            user_data.claims.sub
                        ))
                        .build()?,
                )
                .objects(
                    ObjectIdentifier::builder()
                        .key(format!(
                            "{}/{repo_name}/{commit_id}/status.txt",
                            user_data.claims.sub
                        ))
                        .build()?,
                )
                .objects(
                    ObjectIdentifier::builder()
                        .key(format!(
                            "{}/{repo_name}/{commit_id}/executable",
                            user_data.claims.sub
                        ))
                        .build()?,
                )
                .build()?,
        )
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete previous IR from S3: {e}");
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
            "{}/{repo_name}/{commit_id}/status.txt",
            user_data.claims.sub
        ))
        .body(ByteStream::from(
            CompilationProgress::CloningRepository
                .to_string()
                .into_bytes(),
        ))
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to upload status to S3: {e}");
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error".to_string(),
            )
        })?;

    let repo_full_name = format!("{owner}.{repo_name}");

    let codecommit_client = app_state.get_codecommit_client();

    let clone_url = get_http_clone_url(codecommit_client, &repo_full_name).await?;

    let unzipped_dir = clone_repo(&clone_url)?;

    app_state
        .get_s3_client()
        .put_object()
        .bucket(circuits_bucket_name)
        .key(format!(
            "{}/{repo_name}/{commit_id}/status.txt",
            user_data.claims.sub
        ))
        .body(ByteStream::from(
            CompilationProgress::Compiling.to_string().into_bytes(),
        ))
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to upload status to S3: {e}");
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error".to_string(),
            )
        })?;

    tokio::spawn(async move {
        let result = timeout(
            Duration::from_secs(2 * 60),
            compile_and_upload(
                app_state,
                unzipped_dir,
                owner,
                repo_name,
                commit_id,
                payload.example_artifact,
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
        .body("Started compilation".to_string())
        .map_err(AppError::from)
}

async fn compile_and_upload(
    app_state: AppState,
    unzipped_dir: TempDir,
    owner: String,
    repo_name: String,
    commit_id: String,
    example_artifact: Option<String>,
) -> Result<(), String> {
    let unzipped_dir_path = unzipped_dir.path().to_path_buf();

    let circuits_bucket_name = CIRCUITS_BUCKET_NAME
        .as_ref()
        .ok_or_else(|| "Circuits bucket name not found".to_string())?;

    let mut parsed_cargo = get_parsed_cargo(&unzipped_dir_path.join("Cargo.toml"))?;

    let dependencies = parsed_cargo
        .get("dependencies")
        .ok_or("No dependencies found in `Cargo.toml`")?;

    let target_framework = if dependencies.get("plonky2").is_some() {
        TargetFramework::Plonky2
    } else if dependencies.get("halo2").is_some() {
        TargetFramework::Halo2
    } else {
        panic!("No supported target framework found");
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

    let example_artifact_clone = example_artifact.clone();
    let output = tokio::task::spawn_blocking(move || {
        Command::new("cargo")
            .arg("override")
            .arg("set")
            .arg(target_framework.rust_version())
            .current_dir(&unzipped_dir_path)
            .output()?;

        Command::new("cargo")
            .arg("build")
            .args(
                example_artifact_clone
                    .as_ref()
                    .map(|name| vec!["--example".to_string(), name.to_string()])
                    .unwrap_or_default(),
            )
            .current_dir(unzipped_dir_path)
            .output()
    })
    .await
    .map_err(|e| format!("Failed to join thread: {e}"))?
    .map_err(|e| format!("Failed to build project: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "Failed to build project: {}",
            String::from_utf8(output.stderr)
                .unwrap_or_else(|_| "Failed to convert stderr to string".to_string())
        ));
    }

    tracing::info!("Project built successfully");

    let debug_folder = unzipped_dir.path().join("target/debug");

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
        .key(format!("{owner}/{repo_name}/{commit_id}/executable"))
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
        .key(format!("{owner}/{repo_name}/{commit_id}/executable"))
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

    let json_ir_string = String::from_utf8(json_ir_bytes.clone())
        .map_err(|_| format!("Failed to convert IR bytes to string: {json_ir_bytes:#?}"))?
        // Otherwise it is a string of a string
        .trim_start_matches('"')
        .trim_end_matches('"')
        .replace("\\n", "\n")
        .replace("\\\"", "\"");

    let cir =
        Cir::from_json(&json_ir_string).map_err(|e| format!("Failed to parse json CIR: {e}"))?;

    let source_ir_string = cir.to_code_ir();

    app_state
        .get_s3_client()
        .put_object()
        .bucket(circuits_bucket_name)
        .key(format!("{owner}/{repo_name}/{commit_id}/ir.json"))
        .body(ByteStream::from(json_ir_string.into_bytes()))
        .content_type("application/json")
        .send()
        .await
        .map_err(|e| format!("Failed to upload to S3: {e}"))?;

    app_state
        .get_s3_client()
        .put_object()
        .bucket(circuits_bucket_name)
        .key(format!("{owner}/{repo_name}/{commit_id}/ir.cir"))
        .body(ByteStream::from(source_ir_string.into_bytes()))
        .content_type("text/plain")
        .send()
        .await
        .map_err(|e| format!("Failed to upload to S3: {e}"))?;

    app_state
        .get_s3_client()
        .delete_object()
        .bucket(circuits_bucket_name)
        .key(format!("{owner}/{repo_name}/{commit_id}/executable"))
        .send()
        .await
        .map_err(|e| format!("Failed to delete executable from S3: {e}"))?;

    app_state
        .get_s3_client()
        .put_object()
        .bucket(circuits_bucket_name)
        .key(format!("{owner}/{repo_name}/{commit_id}/status.txt"))
        .body(ByteStream::from(
            CompilationProgress::Completed.to_string().into_bytes(),
        ))
        .send()
        .await
        .map_err(|e| format!("Failed to upload status to S3: {e}"))?;

    Ok(())
}

#[derive(Serialize, ToSchema, Clone)]
pub struct GetIrResponse {
    /// IR as JSON if compiled
    json: Option<String>,

    /// IR as CIR if compiled
    cir: Option<String>,

    /// StatusCode
    status: CompilationProgress,
}

/// Get IR
#[utoipa::path(get,
    tag="IR",
    path="/v1/ir/{owner}/{repo_name}/{commit_id}",
    responses(
        (status = 200, description = "Got IR", body = GetIrResponse),
        (status = 202, description = "Compilation still in progress", body = String),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 404, description = "DId not find IR", body = String),
    ),
    security(
        ("token" = [])
    )
)]
#[debug_handler]
pub async fn get_ir(
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    State(app_state): State<AppState>,
    Path((owner, repo_name, commit_id)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let token = bearer
        .token()
        .to_string()
        .trim_start_matches("Bearer ")
        .to_string();
    let user_data = jwt::get_user_claims(&token)?;

    if user_data.claims.sub != owner {
        return Err(AppError::new(
            StatusCode::UNAUTHORIZED,
            "Unauthorized".to_string(),
        ));
    }

    let circuits_bucket_name = CIRCUITS_BUCKET_NAME.as_ref().ok_or_else(|| {
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not get circuits bucket name".to_string(),
        )
    })?;

    let ir_json_result = app_state
        .get_s3_client()
        .get_object()
        .bucket(circuits_bucket_name)
        .key(format!("{owner}/{repo_name}/{commit_id}/ir.json"))
        .send()
        .await;

    let ir_cir_result = app_state
        .get_s3_client()
        .get_object()
        .bucket(circuits_bucket_name)
        .key(format!("{owner}/{repo_name}/{commit_id}/ir.cir"))
        .send()
        .await;

    if let (Ok(ir_json_res), Ok(ir_cir_res)) = (ir_json_result, ir_cir_result) {
        let ir_json_bytes = ir_json_res.body.collect().await.map_err(|_| {
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read IR from S3".to_string(),
            )
        })?;

        let ir_cir_bytes = ir_cir_res.body.collect().await.map_err(|_| {
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read IR from S3".to_string(),
            )
        })?;

        let ir_json_string = String::from_utf8(ir_json_bytes.to_vec()).map_err(|_| {
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to convert IR bytes to string".to_string(),
            )
        })?;

        let ir_cir_string = String::from_utf8(ir_cir_bytes.to_vec()).map_err(|_| {
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to convert IR bytes to string".to_string(),
            )
        })?;

        return Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(
                serde_json::to_string(&GetIrResponse {
                    json: Some(ir_json_string),
                    cir: Some(ir_cir_string),
                    status: CompilationProgress::Completed,
                })
                .map_err(AppError::from)?,
            )
            .map_err(AppError::from);
    }

    let status = app_state
        .get_s3_client()
        .get_object()
        .bucket(circuits_bucket_name)
        .key(format!("{owner}/{repo_name}/{commit_id}/status.txt"))
        .send()
        .await;

    if let Ok(status) = status {
        let status_bytes = status.body.collect().await.map_err(|_| {
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read status from S3".to_string(),
            )
        })?;

        let status_string = String::from_utf8(status_bytes.to_vec()).map_err(|_| {
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to convert status to string".to_string(),
            )
        })?;

        return Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(
                serde_json::to_string(&GetIrResponse {
                    json: None,
                    cir: None,
                    status: status_string.parse().map_err(|()| {
                        AppError::new(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Failed to parse status".to_string(),
                        )
                    })?,
                })
                .map_err(AppError::from)?,
            )
            .map_err(AppError::from);
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(
            serde_json::to_string(&GetIrResponse {
                json: None,
                cir: None,
                status: CompilationProgress::NotStarted,
            })
            .map_err(AppError::from)?,
        )
        .map_err(AppError::from)
}

#[derive(Serialize, ToSchema, Clone)]
pub struct IrMetadata {
    repo_name: String,
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
        (status = 200, description = "Got IRs", body = ListIrsMetadataResponse),
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

    let mut irs = Vec::new();

    for object in list_objects_res.contents() {
        let Some(key) = object.key() else {
            continue;
        };

        // Only one `description.txt` exists per repo, so we use it to get reference to all unique repos
        if key.ends_with("description.txt") {
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

            let description_string = String::from_utf8(description.to_vec()).map_err(|_| {
                AppError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to convert description to string".to_string(),
                )
            })?;

            irs.push(IrMetadata {
                repo_name: key.split('/').nth(1).unwrap().to_string(),
                description: description_string,
            });
        }
    }

    Ok(Json(ListIrsMetadataResponse { irs }))
}

#[derive(Serialize, ToSchema, Clone)]
pub struct ListIrVersionsResponse {
    /// uuid v7 of versions sorted by time from newest to oldest
    versions: Vec<String>,
}

/// List IR versions of a specific repo
#[utoipa::path(get,
    tag="IR",
    path="/v1/ir/versions/{repo_name}",
    responses(
        (status = 200, description = "Got versions", body = ListIrsMetadataResponse),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
    ),
    security(
        ("token" = [])
    )
)]
#[debug_handler]
pub async fn list_ir_versions(
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    State(app_state): State<AppState>,
    Path(repo_name): Path<String>,
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
        .prefix(format!("{}/{}/", user_data.claims.sub, repo_name))
        .send()
        .await
        .map_err(|_| {
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to list objects in S3".to_string(),
            )
        })?;

    let mut versions = Vec::new();

    for object in list_objects_res.contents() {
        let Some(key) = object.key() else {
            continue;
        };

        // Exactly one `source.zip` exists per version, so we use it to get reference to all unique versions
        if key.ends_with("source.zip") {
            versions.push(key.split('/').nth(2).unwrap().to_string());
        }
    }

    versions.sort();
    versions.reverse();

    Ok(Json(ListIrVersionsResponse { versions }))
}
