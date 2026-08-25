use std::sync::Arc;

use poem_openapi::{param::Path, payload::Json, OpenApi};

use crate::state::AppState;

use super::{
    auth::{authorize, Authorization},
    ApiError, ApiResult, MutationResult, SetBrightnessRequest,
};

pub struct DisplayApi {
    pub state: Arc<AppState>,
}

#[OpenApi]
impl DisplayApi {
    /// Turn the screen on or off.
    #[oai(path = "/display/power/:on", method = "post")]
    async fn set_power(
        &self,
        on: Path<bool>,
        authorization: Authorization,
    ) -> ApiResult<Json<MutationResult>> {
        authorize(&self.state, &authorization)?;

        let display = self.state.config.read().await.display;

        self.state
            .display
            .set_power(&display, on.0)
            .await
            .map_err(ApiError::internal)?;
        self.state.hass.publish_backlight(on.0).await;
        self.state.events.publish(&self.state).await;

        Ok(Json(MutationResult::applied()))
    }

    /// Set panel brightness over DDC.
    #[oai(path = "/display/brightness", method = "put")]
    async fn set_brightness(
        &self,
        request: Json<SetBrightnessRequest>,
        authorization: Authorization,
    ) -> ApiResult<Json<MutationResult>> {
        authorize(&self.state, &authorization)?;

        let display = self.state.config.read().await.display;

        self.state
            .display
            .set_brightness(&display, request.0.percent)
            .await
            .map_err(ApiError::internal)?;

        Ok(Json(MutationResult::applied()))
    }
}
