use std::{sync::Arc, time::Duration};

use poem_openapi::{param::Path, payload::Json, OpenApi};
use tracing::warn;

use crate::{
    chrome::{tell, ChromeMessage},
    config::NotificationMode,
    notifications::Notification,
    state::AppState,
};

use super::{
    auth::{authorize, Authorization},
    ApiError, ApiResult, MutationResult, NotifyRequest, SidebarState, StingerInfo,
};

pub struct NotificationApi {
    pub state: Arc<AppState>,
}

#[OpenApi]
impl NotificationApi {
    /// Raise an alert.
    #[oai(path = "/notify", method = "post")]
    async fn notify(
        &self,
        request: Json<NotifyRequest>,
        authorization: Authorization,
    ) -> ApiResult<Json<MutationResult>> {
        authorize(&self.state, &authorization)?;

        let defaults = self.state.config.read().await.notifications;
        let (notification, duration) = request.0.into_notification(&defaults)?;
        let mode = notification.mode;
        let tab_id = notification.tab_id.clone();
        let stinger = notification.stinger.clone();

        self.state
            .notifications
            .push(notification, duration.into())
            .await;

        // The sidebar and the toast are windows the reconciler owns. Pushing the notification is
        // already what opens them, so there is nothing to do here for either.
        if mode == NotificationMode::Takeover {
            // A transition can take seconds, and the caller is a doorbell or an automation that
            // should not be held open for it. The takeover runs on its own and ends itself, so
            // nothing has to come back and clear it.
            let state = self.state.clone();

            tokio::spawn(async move {
                if let Err(error) = tell(
                    &state.chrome,
                    ChromeMessage::Takeover {
                        tab_id,
                        stinger,
                        seconds: Some(duration.seconds()),
                    },
                )
                .await
                {
                    warn!("takeover failed: {error}");

                    return;
                }

                tokio::time::sleep(Duration::from(duration)).await;

                // Only a live takeover keeps this one on screen. Asking whether anything at all is
                // live leaves the display on a camera because an agenda entry has not ended yet.
                if state
                    .notifications
                    .current_in(NotificationMode::Takeover)
                    .await
                    .is_none()
                {
                    let _ = tell(&state.chrome, ChromeMessage::EndTakeover).await;
                }
            });
        }

        Ok(Json(MutationResult::applied()))
    }

    /// The configured clips, so the transition page can resolve a name to a file.
    #[oai(path = "/stingers", method = "get")]
    async fn stingers(&self) -> ApiResult<Json<Vec<StingerInfo>>> {
        let stingers = self.state.config.read().await.notifications.stingers;

        Ok(Json(
            stingers
                .into_iter()
                .map(|(name, stinger)| StingerInfo {
                    name,
                    file: stinger.file,
                })
                .collect(),
        ))
    }

    /// Everything currently showing. The alert pages read this.
    #[oai(path = "/notifications", method = "get")]
    async fn list(&self) -> ApiResult<Json<Vec<Notification>>> {
        Ok(Json(self.state.notifications.active().await))
    }

    /// Clear one early.
    #[oai(path = "/notifications/:notification_id", method = "delete")]
    async fn dismiss(
        &self,
        notification_id: Path<u64>,
        authorization: Authorization,
    ) -> ApiResult<Json<MutationResult>> {
        authorize(&self.state, &authorization)?;

        if !self.state.notifications.dismiss(notification_id.0).await {
            return Err(ApiError::not_found(notification_id.0));
        }

        if self
            .state
            .notifications
            .current_in(NotificationMode::Takeover)
            .await
            .is_none()
        {
            let _ = tell(&self.state.chrome, ChromeMessage::EndTakeover).await;
        }

        Ok(Json(MutationResult::applied()))
    }

    /// Open the rail if it is closed, close it if it is open.
    ///
    /// One call, no state to keep at the other end, which is what a button on a launchpad needs.
    /// A closed rail stays closed until something new arrives for it.
    #[oai(path = "/sidebar/toggle", method = "post")]
    async fn toggle_sidebar(
        &self,
        authorization: Authorization,
    ) -> ApiResult<Json<SidebarState>> {
        authorize(&self.state, &authorization)?;

        let open = self.state.surfaces.toggle_sidebar(&self.state).await;

        Ok(Json(SidebarState { open }))
    }

    /// Whether the rail is up.
    #[oai(path = "/sidebar", method = "get")]
    async fn sidebar(&self) -> ApiResult<Json<SidebarState>> {
        Ok(Json(SidebarState {
            open: self.state.surfaces.sidebar.is_open().await,
        }))
    }
}
