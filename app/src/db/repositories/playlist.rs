use anyhow::Result;
use async_trait::async_trait;
use sqlx::{SqlitePool, Row};
use chrono::Utc;

use crate::db::models::*;
use super::PlaylistRepository;

pub struct SqlitePlaylistRepository {
    pool: SqlitePool,
}

impl SqlitePlaylistRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PlaylistRepository for SqlitePlaylistRepository {
    async fn create(&self, request: CreatePlaylistRequest) -> Result<Playlist> {
        let now = Utc::now();
        
        sqlx::query(
            "INSERT INTO playlists (id, name, interval_seconds, is_active, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&request.id)
        .bind(&request.name)
        .bind(request.interval_seconds)
        .bind(false)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;
        
        Ok(Playlist {
            id: request.id,
            name: request.name,
            interval_seconds: request.interval_seconds,
            is_active: false,
            created_at: now,
            updated_at: now,
        })
    }
    
    async fn get_by_id(&self, id: &str) -> Result<Option<Playlist>> {
        let row = sqlx::query(
            "SELECT id, name, interval_seconds, is_active, created_at, updated_at 
             FROM playlists WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            Ok(Some(Playlist {
                id: row.get("id"),
                name: row.get("name"),
                interval_seconds: row.get("interval_seconds"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }
    
    async fn get_all(&self) -> Result<Vec<Playlist>> {
        let rows = sqlx::query(
            "SELECT id, name, interval_seconds, is_active, created_at, updated_at 
             FROM playlists ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut playlists = Vec::new();
        for row in rows {
            playlists.push(Playlist {
                id: row.get("id"),
                name: row.get("name"),
                interval_seconds: row.get("interval_seconds"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        
        Ok(playlists)
    }
    
    async fn get_with_tabs(&self, id: &str) -> Result<Option<PlaylistWithTabs>> {
        let playlist = self.get_by_id(id).await?;
        
        if let Some(playlist) = playlist {
            let rows = sqlx::query(
                "SELECT t.id, t.name, t.url, t.persist, t.viewport_width, t.viewport_height, 
                        pt.order_index, pt.duration_seconds, pt.enabled, pt.last_manual_activation,
                        t.created_at, t.updated_at
                 FROM tabs t
                 JOIN playlist_tabs pt ON t.id = pt.tab_id
                 WHERE pt.playlist_id = ?
                 ORDER BY pt.order_index"
            )
            .bind(id)
            .fetch_all(&self.pool)
            .await?;
            
            let mut tabs = Vec::new();
            for row in rows {
                tabs.push(TabWithOrder {
                    id: row.get("id"),
                    name: row.get("name"),
                    url: row.get("url"),
                    persist: row.get("persist"),
                    viewport_width: row.get("viewport_width"),
                    viewport_height: row.get("viewport_height"),
                    order_index: row.get("order_index"),
                    duration_seconds: row.get("duration_seconds"),
                    enabled: row.get("enabled"),
                    last_manual_activation: row.get("last_manual_activation"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                });
            }
            
            Ok(Some(PlaylistWithTabs {
                id: playlist.id,
                name: playlist.name,
                interval_seconds: playlist.interval_seconds,
                is_active: playlist.is_active,
                created_at: playlist.created_at,
                updated_at: playlist.updated_at,
                tabs,
            }))
        } else {
            Ok(None)
        }
    }
    
    async fn get_all_with_tabs(&self) -> Result<Vec<PlaylistWithTabs>> {
        let playlists = self.get_all().await?;
        let mut result = Vec::new();
        
        for playlist in playlists {
            if let Some(playlist_with_tabs) = self.get_with_tabs(&playlist.id).await? {
                result.push(playlist_with_tabs);
            }
        }
        
        Ok(result)
    }
    
    async fn update(&self, id: &str, request: UpdatePlaylistRequest) -> Result<Option<Playlist>> {
        let now = Utc::now();
        
        // Build dynamic update query
        let mut query_builder = sqlx::QueryBuilder::new("UPDATE playlists SET ");
        
        let mut first = true;
        if let Some(name) = &request.name {
            if !first {
                query_builder.push(", ");
            }
            query_builder.push("name = ");
            query_builder.push_bind(name);
            first = false;
        }
        
        if let Some(interval_seconds) = request.interval_seconds {
            if !first {
                query_builder.push(", ");
            }
            query_builder.push("interval_seconds = ");
            query_builder.push_bind(interval_seconds);
            first = false;
        }
        
        if let Some(is_active) = request.is_active {
            if !first {
                query_builder.push(", ");
            }
            query_builder.push("is_active = ");
            query_builder.push_bind(is_active);
            first = false;
        }
        
        if first {
            return self.get_by_id(id).await;
        }
        
        query_builder.push(", updated_at = ");
        query_builder.push_bind(now);
        query_builder.push(" WHERE id = ");
        query_builder.push_bind(id);
        
        let result = query_builder.build().execute(&self.pool).await?;
        
        if result.rows_affected() > 0 {
            self.get_by_id(id).await
        } else {
            Ok(None)
        }
    }
    
    async fn delete(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM playlists WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    async fn set_active(&self, id: &str, is_active: bool) -> Result<()> {
        // First, set all playlists to inactive
        sqlx::query("UPDATE playlists SET is_active = false")
            .execute(&self.pool)
            .await?;
        
        // Then set the specified playlist to active if requested
        if is_active {
            sqlx::query("UPDATE playlists SET is_active = true WHERE id = ?")
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        
        Ok(())
    }
    
        async fn get_active(&self) -> Result<Option<Playlist>> {
        let row = sqlx::query(
            "SELECT id, name, interval_seconds, is_active, created_at, updated_at 
             FROM playlists WHERE is_active = true LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Playlist {
                id: row.get("id"),
                name: row.get("name"),
                interval_seconds: row.get("interval_seconds"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_tabs(&self, playlist_id: &str) -> Result<Vec<Tab>> {
        let rows = sqlx::query(
            "SELECT t.id, t.name, t.url, t.persist, t.viewport_width, t.viewport_height, t.created_at, t.updated_at
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
            tabs.push(Tab {
                id: row.get("id"),
                name: row.get("name"),
                url: row.get("url"),
                persist: row.get("persist"),
                viewport_width: row.get("viewport_width"),
                viewport_height: row.get("viewport_height"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(tabs)
    }

    async fn update_interval(&self, playlist_id: &str, interval_seconds: i64) -> Result<()> {
        let now = Utc::now();
        
        sqlx::query(
            "UPDATE playlists SET interval_seconds = ?, updated_at = ? WHERE id = ?"
        )
        .bind(interval_seconds)
        .bind(now)
        .bind(playlist_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
} 