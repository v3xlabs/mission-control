use anyhow::Result;
use async_trait::async_trait;
use sqlx::{SqlitePool, Row};

use crate::db::models::*;
use super::PlaylistTabRepository;

pub struct SqlitePlaylistTabRepository {
    pool: SqlitePool,
}

impl SqlitePlaylistTabRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PlaylistTabRepository for SqlitePlaylistTabRepository {
    async fn add_tab_to_playlist(&self, playlist_id: &str, request: AddTabToPlaylistRequest) -> Result<()> {
        // First check if the tab exists
        let tab_count: i64 = sqlx::query("SELECT COUNT(*) FROM tabs WHERE id = ?")
            .bind(&request.tab_id)
            .fetch_one(&self.pool)
            .await?
            .get(0);
        
        if tab_count == 0 {
            return Err(anyhow::anyhow!("Tab with id '{}' does not exist", request.tab_id));
        }
        
        // Check if the playlist exists
        let playlist_count: i64 = sqlx::query("SELECT COUNT(*) FROM playlists WHERE id = ?")
            .bind(playlist_id)
            .fetch_one(&self.pool)
            .await?
            .get(0);
        
        if playlist_count == 0 {
            return Err(anyhow::anyhow!("Playlist with id '{}' does not exist", playlist_id));
        }
        
        // Insert or update the playlist-tab relationship
        sqlx::query(
            "INSERT OR REPLACE INTO playlist_tabs (playlist_id, tab_id, order_index, duration_seconds) 
             VALUES (?, ?, ?, ?)"
        )
        .bind(playlist_id)
        .bind(&request.tab_id)
        .bind(request.order_index)
        .bind(request.duration_seconds)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn remove_tab_from_playlist(&self, playlist_id: &str, tab_id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM playlist_tabs WHERE playlist_id = ? AND tab_id = ?")
            .bind(playlist_id)
            .bind(tab_id)
            .execute(&self.pool)
            .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    async fn get_playlist_tabs(&self, playlist_id: &str) -> Result<Vec<TabWithOrder>> {
        let rows = sqlx::query(
            "SELECT t.id, t.name, t.url, t.persist, pt.order_index, pt.duration_seconds, t.created_at, t.updated_at
             FROM tabs t
             JOIN playlist_tabs pt ON t.id = pt.tab_id
             WHERE pt.playlist_id = ?
             ORDER BY pt.order_index"
        )
        .bind(playlist_id)
        .fetch_all(&self.pool)
        .await?;
        
        let mut tabs = Vec::new();
        for row in rows {
            tabs.push(TabWithOrder {
                id: row.get("id"),
                name: row.get("name"),
                url: row.get("url"),
                persist: row.get("persist"),
                order_index: row.get("order_index"),
                duration_seconds: row.get("duration_seconds"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        
        Ok(tabs)
    }
    
    async fn reorder_tabs(&self, playlist_id: &str, request: ReorderTabsRequest) -> Result<()> {
        // Start a transaction
        let mut tx = self.pool.begin().await?;
        
        // Update the order for each tab
        for tab_order in request.tab_orders {
            sqlx::query("UPDATE playlist_tabs SET order_index = ? WHERE playlist_id = ? AND tab_id = ?")
                .bind(tab_order.order_index)
                .bind(playlist_id)
                .bind(tab_order.tab_id)
                .execute(&mut *tx)
                .await?;
        }
        
        // Commit the transaction
        tx.commit().await?;
        
        Ok(())
    }
} 