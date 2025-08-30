# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Mission Control is a Rust application for managing digital signage displays with a React-based web interface. It controls Chrome browser instances through playlists and tabs, integrates with Home Assistant via MQTT, and provides a REST API for programmatic control.

## Architecture

### Backend (Rust)
- **Framework**: `poem` HTTP server with `async-std` runtime
- **Database**: SQLite with `sqlx` for data persistence
- **Chrome Control**: `chromiumoxide` for browser automation
- **MQTT**: `rumqttc` for Home Assistant integration
- **Configuration**: `figment` with TOML configuration files

### Frontend (React TypeScript)
- **Build Tool**: Vite with React plugin
- **Styling**: Tailwind CSS with PostCSS
- **API**: TanStack Query for data fetching
- **Type Safety**: OpenAPI-generated TypeScript schemas

### Key Components
- **Chrome Controller** (`app/src/chrome/controller.rs`): Message-driven browser control system
- **Database Layer** (`app/src/db/`): SQLite repositories for playlists, tabs, and relationships
- **API Layer** (`app/src/api/`): OpenAPI-documented REST endpoints
- **Configuration** (`config.toml`): Device settings, MQTT credentials, and display configuration

## Development Commands

### Backend Development
```bash
cd app
cargo run                    # Start backend server (port 3000)
cargo test                   # Run Rust tests
cargo clippy                 # Lint Rust code
```

### Frontend Development
```bash
cd web
pnpm install                 # Install dependencies
pnpm dev                     # Start dev server (port 5173, proxies to backend)
pnpm build                   # Build for production
pnpm typecheck              # Type check TypeScript
pnpm api-schema             # Regenerate API types from backend OpenAPI spec
```

### Database Management
The application automatically creates and migrates the SQLite database on startup. Database file is located at `./app/sqlite.db`.

## Configuration

### Main Configuration
Configuration is read from `./config.toml` or `~/.config/v3x-mission-control/config.toml`:

```toml
[device]
name = "Display Name"
id = "unique_device_id"

[homeassistant]
mqtt_url = "mqtt://localhost:1883"
mqtt_username = "username"
mqtt_password = "password"

[chromium]
enabled = true
binary_path = "/usr/bin/chromium"  # optional
```

### Data Storage
- **Static Config**: Device settings, MQTT credentials remain in TOML
- **Dynamic Data**: Playlists and tabs are stored in SQLite database
- **Migration**: Existing TOML playlist/tab config is automatically imported to database on first run

## Database Schema

### Core Tables
- `playlists`: Playlist metadata (id, name, interval_seconds, is_active)
- `tabs`: Tab definitions (id, name, url, persist)
- `playlist_tabs`: Many-to-many relationship with ordering (playlist_id, tab_id, order_index)

### Repository Pattern
Located in `app/src/db/repositories/`:
- `PlaylistRepository`: CRUD operations for playlists
- `TabRepository`: CRUD operations for tabs
- `PlaylistTabRepository`: Managing playlist-tab relationships

## Chrome Control System

The Chrome controller uses a message-driven architecture:
- **Messages** (`ChromeMessage`): Commands for playlist activation, tab switching
- **State Management**: Current playlist/tab state persisted in database
- **Screen Capture**: Real-time tab screenshots for web UI previews
- **Responsive Control**: Immediate tab switching without restart

## API Structure

### REST Endpoints
- `GET /api/playlists` - List all playlists
- `GET /api/tabs` - List all tabs  
- `POST /api/playlists/:id/activate` - Activate a playlist
- `POST /api/tabs/:id/activate` - Switch to specific tab
- `GET /api/status` - Current system status

### OpenAPI Documentation
- Interactive docs available at `http://localhost:3000/docs`
- TypeScript types auto-generated in `web/src/api/schema.gen.ts`

## Key Files to Understand

- `app/src/main.rs:17-59` - Application initialization and async task spawning
- `app/src/state.rs` - Shared application state management
- `app/src/chrome/controller.rs:29-41` - Chrome controller structure
- `app/src/db/mod.rs:9-33` - Database initialization and migration logic
- `web/src/main.tsx` - Frontend application entry point
- `web/vite.config.ts:6-15` - Development proxy configuration

## Migration Status

The codebase is currently migrating from static TOML configuration to SQLite-based data storage (see `MIGRATION_PLAN.md`). The migration automatically imports existing playlist/tab configurations from TOML into the database while preserving backward compatibility.