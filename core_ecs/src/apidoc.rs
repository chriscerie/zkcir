use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

use crate::{auth, ir, profile, UnauthorizedResponse};

#[derive(OpenApi)]
#[openapi(
    paths(
        auth::auth_google,
        profile::get_profile,
        ir::compile_to_ir,
        ir::get_ir,
        ir::list_irs_metadata,
        ir::list_ir_versions,
    ),
    modifiers(&SecurityAddon),
    components(schemas(
        UnauthorizedResponse,
        profile::GetProfileResponse,
        ir::CompileToIrPayload,
        ir::CompileToIrResponse,
        ir::GetIrResponse,
        ir::IrMetadata,
        ir::ListIrsMetadataResponse,
        ir::ListIrVersionsResponse,
    )),
    tags(
        (name = "zkcir", description = "Zero Proof Knowledge Circuits IR")
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "token",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some("Obtained through `/v1/auth`"))
                        .build(),
                ),
            );
        }
    }
}
