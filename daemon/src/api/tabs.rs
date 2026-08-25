use std::sync::Arc;

use poem_openapi::{param::Path, payload::Json, OpenApi};

use crate::{
    chrome::{tell, ChromeMessage},
    config::Document,
    state::AppState,
};

use super::{
    auth::{authorize, Authorization},
    ApiError, ApiResult, MutationResult, TabInfo, UpsertTabRequest,
};

pub struct TabApi {
    pub state: Arc<AppState>,
}

#[OpenApi]
impl TabApi {
    /// Every configured tab, whether or not a playlist uses it.
    #[oai(path = "/tabs", method = "get")]
    async fn list(&self) -> ApiResult<Json<Vec<TabInfo>>> {
        let tabs = self.state.config.tabs().await;
        let mut out = Vec::with_capacity(tabs.len());

        for (index, tab) in tabs.iter().enumerate() {
            out.push(TabInfo::new(tab, index, true, &self.state.chrome).await);
        }

        Ok(Json(out))
    }

    /// Create a tab, or replace one with the same id.
    #[oai(path = "/tabs/:tab_id", method = "put")]
    async fn upsert(
        &self,
        tab_id: Path<String>,
        request: Json<UpsertTabRequest>,
        authorization: Authorization,
    ) -> ApiResult<Json<MutationResult>> {
        authorize(&self.state, &authorization)?;

        let url = request.0.url.clone();
        let tab = request.0.into_tab(tab_id.0.clone());

        let persisted = self
            .state
            .config
            .mutate(Document::Tabs, |documents| {
                match documents
                    .tabs
                    .tabs
                    .iter_mut()
                    .find(|existing| existing.tab_id == tab_id.0)
                {
                    Some(existing) => *existing = tab.clone(),
                    None => documents.tabs.tabs.push(tab.clone()),
                }
            })
            .await
            .map_err(ApiError::internal)?;

        let _ = self.state.chrome.update_url(&tab_id.0, &url).await;

        Ok(Json(persisted.into()))
    }

    /// Remove a tab, and every playlist reference to it.
    #[oai(path = "/tabs/:tab_id", method = "delete")]
    async fn delete(
        &self,
        tab_id: Path<String>,
        authorization: Authorization,
    ) -> ApiResult<Json<MutationResult>> {
        authorize(&self.state, &authorization)?;

        self.state
            .config
            .tab(&tab_id.0)
            .await
            .ok_or_else(|| ApiError::not_found(&tab_id.0))?;

        let persisted = self
            .state
            .config
            .mutate(Document::Tabs, |documents| {
                documents.tabs.tabs.retain(|tab| tab.tab_id != tab_id.0);
            })
            .await
            .map_err(ApiError::internal)?;

        self.state
            .config
            .mutate(Document::Playlists, |documents| {
                for playlist in &mut documents.playlists.playlists {
                    playlist.tabs.retain(|id| id != &tab_id.0);
                    playlist.disabled_tabs.retain(|id| id != &tab_id.0);
                }
            })
            .await
            .map_err(ApiError::internal)?;

        let _ = tell(
            &self.state.chrome,
            ChromeMessage::CloseTab {
                tab_id: tab_id.0.clone(),
            },
        )
        .await;

        Ok(Json(persisted.into()))
    }

    /// Reload a tab's page.
    #[oai(path = "/tabs/:tab_id/refresh", method = "post")]
    async fn refresh(
        &self,
        tab_id: Path<String>,
        authorization: Authorization,
    ) -> ApiResult<Json<MutationResult>> {
        authorize(&self.state, &authorization)?;
        tell(
            &self.state.chrome,
            ChromeMessage::RefreshTab { tab_id: tab_id.0 },
        )
        .await
        .map_err(ApiError::internal)?;

        Ok(Json(MutationResult::applied()))
    }

    /// Close and reopen a tab's page.
    #[oai(path = "/tabs/:tab_id/recreate", method = "post")]
    async fn recreate(
        &self,
        tab_id: Path<String>,
        authorization: Authorization,
    ) -> ApiResult<Json<MutationResult>> {
        authorize(&self.state, &authorization)?;
        tell(
            &self.state.chrome,
            ChromeMessage::RecreateTab { tab_id: tab_id.0 },
        )
        .await
        .map_err(ApiError::internal)?;

        Ok(Json(MutationResult::applied()))
    }
}
