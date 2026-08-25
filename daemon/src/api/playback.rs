use std::sync::Arc;

use poem_openapi::{payload::Json, OpenApi};

use crate::{
    chrome::{tell, ChromeMessage},
    state::AppState,
};

use super::{
    auth::{authorize, Authorization},
    ApiError, ApiResult, MutationResult,
};

pub struct PlaybackApi {
    pub state: Arc<AppState>,
}

#[OpenApi]
impl PlaybackApi {
    /// Advance to the next tab in the current playlist.
    #[oai(path = "/playback/next", method = "post")]
    async fn next(&self, authorization: Authorization) -> ApiResult<Json<MutationResult>> {
        self.send(ChromeMessage::NextTab, &authorization).await
    }

    /// Step back to the previous tab.
    #[oai(path = "/playback/previous", method = "post")]
    async fn previous(&self, authorization: Authorization) -> ApiResult<Json<MutationResult>> {
        self.send(ChromeMessage::PreviousTab, &authorization).await
    }

    /// Stop rotating. The tab on screen stays there.
    #[oai(path = "/playback/pause", method = "post")]
    async fn pause(&self, authorization: Authorization) -> ApiResult<Json<MutationResult>> {
        self.send(ChromeMessage::Pause, &authorization).await
    }

    /// Resume rotating, clearing any hold left by a tab chosen by hand.
    #[oai(path = "/playback/resume", method = "post")]
    async fn resume(&self, authorization: Authorization) -> ApiResult<Json<MutationResult>> {
        self.send(ChromeMessage::Resume, &authorization).await
    }
}

impl PlaybackApi {
    async fn send(
        &self,
        message: ChromeMessage,
        authorization: &Authorization,
    ) -> ApiResult<Json<MutationResult>> {
        authorize(&self.state, authorization)?;
        tell(&self.state.chrome, message)
            .await
            .map_err(ApiError::internal)?;

        Ok(Json(MutationResult::applied()))
    }
}
