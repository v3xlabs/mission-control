use anyhow::Result;
use sqlx::{SqlitePool, migrate::MigrateDatabase, Sqlite};
use tracing::info;

pub mod models;
pub mod repositories;

/// Initialize the database connection and run migrations
pub async fn init_database() -> Result<SqlitePool> {
    // Create database file relative to the binary location
    let db_path = "./sqlite.db";
    let db_url = format!("sqlite://{}", db_path);
    
    info!("Initializing database at: {}", db_path);
    
    // Create database if it doesn't exist
    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        info!("Database not found, creating new database");
        Sqlite::create_database(&db_url).await?;
    }
    
    // Connect to database
    let pool = SqlitePool::connect(&db_url).await?;
    
    // Run migrations
    info!("Running database migrations");
    run_migrations(&pool).await?;
    
    // Seed initial data if needed
    seed_initial_data(&pool).await?;
    
    Ok(pool)
}

/// Run database migrations
async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    // Create tables
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS playlists (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            interval_seconds INTEGER NOT NULL DEFAULT 30,
            is_active BOOLEAN NOT NULL DEFAULT FALSE,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#
    ).execute(pool).await?;
    
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tabs (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            persist BOOLEAN NOT NULL DEFAULT TRUE,
            viewport_width INTEGER,
            viewport_height INTEGER,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#
    ).execute(pool).await?;
    
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS playlist_tabs (
            playlist_id TEXT NOT NULL,
            tab_id TEXT NOT NULL,
            order_index INTEGER NOT NULL,
            duration_seconds INTEGER,
            PRIMARY KEY (playlist_id, tab_id),
            FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
            FOREIGN KEY (tab_id) REFERENCES tabs(id) ON DELETE CASCADE
        )
        "#
    ).execute(pool).await?;
    
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#
    ).execute(pool).await?;
    
    // Create indexes for performance
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_playlist_tabs_playlist_id ON playlist_tabs(playlist_id)")
        .execute(pool).await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_playlist_tabs_order ON playlist_tabs(playlist_id, order_index)")
        .execute(pool).await?;
    
    // Add viewport dimensions columns if they don't exist (for existing databases)
    let _ = sqlx::query("ALTER TABLE tabs ADD COLUMN viewport_width INTEGER")
        .execute(pool).await; // Ignore errors if column already exists
    
    let _ = sqlx::query("ALTER TABLE tabs ADD COLUMN viewport_height INTEGER")
        .execute(pool).await; // Ignore errors if column already exists
    
    // Add new fields to playlist_tabs table
    let _ = sqlx::query("ALTER TABLE playlist_tabs ADD COLUMN enabled BOOLEAN NOT NULL DEFAULT TRUE")
        .execute(pool).await; // Ignore errors if column already exists
        
    let _ = sqlx::query("ALTER TABLE playlist_tabs ADD COLUMN last_manual_activation DATETIME")
        .execute(pool).await; // Ignore errors if column already exists
    
    Ok(())
}

/// Seed initial data if the database is empty
async fn seed_initial_data(pool: &SqlitePool) -> Result<()> {
    // Check if we have any playlists
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM playlists")
        .fetch_one(pool)
        .await?;

    if count == 0 {
        info!("Database is empty, seeding initial data");
        
        // Create hello world playlist
        sqlx::query(
            "INSERT INTO playlists (id, name, interval_seconds, is_active) VALUES (?, ?, ?, ?)"
        )
        .bind("hello_world")
        .bind("Hello World")
        .bind(30)
        .bind(true)
        .execute(pool)
        .await?;
        
        // Create welcome tab
        sqlx::query(
            "INSERT INTO tabs (id, name, url, persist) VALUES (?, ?, ?, ?)"
        )
        .bind("welcome")
        .bind("Welcome Page")
        .bind("https://example.com/welcome")
        .bind(true)
        .execute(pool)
        .await?;
        
        // Create dashboard tab
        sqlx::query(
            "INSERT INTO tabs (id, name, url, persist) VALUES (?, ?, ?, ?)"
        )
        .bind("dashboard")
        .bind("Dashboard")
        .bind("https://example.com/dashboard")
        .bind(true)
        .execute(pool)
        .await?;
        
        // Add tabs to playlist
        sqlx::query(
            "INSERT INTO playlist_tabs (playlist_id, tab_id, order_index) VALUES (?, ?, ?)"
        )
        .bind("hello_world")
        .bind("welcome")
        .bind(0)
        .execute(pool)
        .await?;
        
        sqlx::query(
            "INSERT INTO playlist_tabs (playlist_id, tab_id, order_index) VALUES (?, ?, ?)"
        )
        .bind("hello_world")
        .bind("dashboard")
        .bind(1)
        .execute(pool)
        .await?;
        
        info!("Initial data seeded successfully");
    }
    
    Ok(())
}

/// Import data from existing TOML config if needed
pub async fn import_config_data(pool: &SqlitePool, chromium_config: &crate::config::ChromiumConfig) -> Result<()> {
    info!("Importing data from config file");
    
    // Import tabs
    if let Some(tabs) = &chromium_config.tabs {
        for (tab_id, tab_config) in tabs {
            // Insert tab if it doesn't exist
            sqlx::query(
                "INSERT OR IGNORE INTO tabs (id, name, url, persist) VALUES (?, ?, ?, ?)"
            )
            .bind(tab_id)
            .bind(tab_id) // Use ID as name for now
            .bind(&tab_config.url)
            .bind(tab_config.persist)
            .execute(pool)
            .await?;
        }
    }
    
    // Import playlists
    if let Some(playlists) = &chromium_config.playlists {
        for (playlist_id, playlist_config) in playlists {
            // Insert playlist if it doesn't exist
            sqlx::query(
                "INSERT OR IGNORE INTO playlists (id, name, interval_seconds, is_active) VALUES (?, ?, ?, ?)"
            )
            .bind(playlist_id)
            .bind(playlist_id) // Use ID as name for now
            .bind(playlist_config.interval as i64)
            .bind(false) // Set to inactive by default
            .execute(pool)
            .await?;
            
            // Insert playlist-tab relationships
            for (index, tab_id) in playlist_config.tabs.iter().enumerate() {
                sqlx::query(
                    "INSERT OR IGNORE INTO playlist_tabs (playlist_id, tab_id, order_index) VALUES (?, ?, ?)"
                )
                .bind(playlist_id)
                .bind(tab_id)
                .bind(index as i64)
                .execute(pool)
                .await?;
            }
        }
    }
    
    info!("Config data imported successfully");
    Ok(())
} 