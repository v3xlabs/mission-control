use std::sync::Arc;

use poem_openapi::{param::Path, payload::Json, OpenApi};

use crate::{
    chrome::{tell, ChromeMessage},
    display,
    state::AppState,
};

use super::{
    auth::{authorize, Authorization},
    ApiError, ApiResult, MutationResult,
};

pub struct SystemApi {
    pub state: Arc<AppState>,
}

#[OpenApi]
impl SystemApi {
    /// Power the machine off, reboot, or suspend it, through logind.
    #[oai(path = "/system/:action", method = "post")]
    async fn power(
        &self,
        action: Path<String>,
        authorization: Authorization,
    ) -> ApiResult<Json<MutationResult>> {
        authorize(&self.state, &authorization)?;

        let method = match action.0.as_str() {
            "poweroff" => "PowerOff",
            "reboot" => "Reboot",
            "suspend" => "Suspend",
            other => {
                return Err(ApiError::bad_request(format!(
                    "unknown action {other}, expected poweroff, reboot or suspend"
                )))
            }
        };

        let _ = tell(&self.state.chrome, ChromeMessage::Shutdown).await;

        display::logind(method).await.map_err(ApiError::internal)?;

        Ok(Json(MutationResult::applied()))
    }
}
