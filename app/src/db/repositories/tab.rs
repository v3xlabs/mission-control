use anyhow::Result;
use async_trait::async_trait;
use sqlx::{SqlitePool, Row};
use chrono::Utc;

use crate::db::models::*;
use super::TabRepository;

pub struct SqliteTabRepository {
    pool: SqlitePool,
}

impl SqliteTabRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TabRepository for SqliteTabRepository {
    async fn create(&self, request: CreateTabRequest) -> Result<Tab> {
        let now = Utc::now();
        let persist = request.persist.unwrap_or(true);
        
        sqlx::query(
            "INSERT INTO tabs (id, name, url, persist, viewport_width, viewport_height, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&request.id)
        .bind(&request.name)
        .bind(&request.url)
        .bind(persist)
        .bind(None::<i32>)  // viewport_width
        .bind(None::<i32>)  // viewport_height
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;
        
        Ok(Tab {
            id: request.id,
            name: request.name,
            url: request.url,
            persist,
            viewport_width: None,
            viewport_height: None,
            created_at: now,
            updated_at: now,
        })
    }
    
    async fn get_by_id(&self, id: &str) -> Result<Option<Tab>> {
        let row = sqlx::query(
            "SELECT id, name, url, persist, viewport_width, viewport_height, created_at, updated_at 
             FROM tabs WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            Ok(Some(Tab {
                id: row.get("id"),
                name: row.get("name"),
                url: row.get("url"),
                persist: row.get("persist"),
                viewport_width: row.get("viewport_width"),
                viewport_height: row.get("viewport_height"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }
    
    async fn get_all(&self) -> Result<Vec<Tab>> {
        let rows = sqlx::query(
            "SELECT id, name, url, persist, viewport_width, viewport_height, created_at, updated_at 
             FROM tabs ORDER BY created_at DESC"
        )
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
    
    async fn update(&self, id: &str, request: UpdateTabRequest) -> Result<Option<Tab>> {
        let now = Utc::now();
        
        // Build dynamic update query
        let mut query_builder = sqlx::QueryBuilder::new("UPDATE tabs SET ");
        
        let mut first = true;
        if let Some(name) = &request.name {
            if !first {
                query_builder.push(", ");
            }
            query_builder.push("name = ");
            query_builder.push_bind(name);
            first = false;
        }
        
        if let Some(url) = &request.url {
            if !first {
                query_builder.push(", ");
            }
            query_builder.push("url = ");
            query_builder.push_bind(url);
            first = false;
        }
        
        if let Some(persist) = request.persist {
            if !first {
                query_builder.push(", ");
            }
            query_builder.push("persist = ");
            query_builder.push_bind(persist);
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
        let result = sqlx::query("DELETE FROM tabs WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(result.rows_affected() > 0)
    }

    async fn update_url(&self, id: &str, url: &str) -> Result<()> {
        let now = Utc::now();
        
        sqlx::query("UPDATE tabs SET url = ?, updated_at = ? WHERE id = ?")
            .bind(url)
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }

    async fn update_viewport_dimensions(&self, id: &str, width: i32, height: i32) -> Result<()> {
        let now = Utc::now();
        
        sqlx::query("UPDATE tabs SET viewport_width = ?, viewport_height = ?, updated_at = ? WHERE id = ?")
            .bind(width)
            .bind(height)
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
} 