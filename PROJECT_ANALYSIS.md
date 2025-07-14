# Mission Control - Project Analysis & Roadmap

## Current State Overview

Mission Control is a Rust-based digital signage solution that controls displays via Chromium browser automation. The application is functional and deployed in production.

### Architecture Summary

**Core Components:**
- **Main Application** (`main.rs`): Orchestrates all components with async-std runtime
- **Chrome Controller** (`chrome.rs`): Manages Chromium browser instances, tabs, and playlists
- **HTTP Server** (`http/mod.rs`): Provides basic web UI with preview endpoints
- **Home Assistant Integration** (`models/hass/`): MQTT-based device control and status reporting
- **Configuration Management** (`config.rs`): TOML-based configuration with figment

### Current Features ✅

1. **Browser Control**
   - Chromium automation via chromiumoxide
   - Tab management with persistent tabs
   - Playlist rotation with configurable intervals
   - Screen capture via Chrome DevTools Protocol
   - Kiosk mode operation

2. **Home Assistant Integration** 
   - MQTT connectivity with authentication
   - Device entities (backlight, brightness, playlist, tab, URL)
   - Auto-discovery via MQTT discovery protocol
   - Remote playlist switching
   - Display power management (xset/xrandr commands)

3. **Web Interface**
   - Basic HTTP server on port 3000
   - Static image preview: `/preview/:tab_id`
   - Live MJPEG stream: `/preview_live/:tab_id`
   - Base64 encoded JPEG frames at ~4 FPS

4. **Configuration**
   - TOML configuration with schema validation
   - Support for multiple tabs and playlists
   - Device identification and naming
   - MQTT connection settings

### Technical Stack

- **Runtime**: async-std with tokio compatibility
- **HTTP**: poem web framework
- **Browser**: chromiumoxide for Chrome DevTools Protocol
- **MQTT**: rumqttc client
- **Config**: figment with TOML support
- **Logging**: tracing with subscriber

## Current Limitations & Issues

### Architecture Issues
1. **State Management**: Configuration and runtime state are mixed
2. **No Persistence**: All state is in-memory, lost on restart
3. **Single Display**: No multi-display support despite hardware capability
4. **Basic Web UI**: Minimal preview-only interface

### Code Quality Issues
1. **Error Handling**: Inconsistent error propagation
2. **Threading**: Manual async task spawning, no structured concurrency
3. **Configuration**: No runtime config updates
4. **Testing**: No test coverage

### Missing Features
1. **Database**: No persistent storage for playlists, configs, tabs
2. **Multi-Display**: Cannot control multiple monitors per device
3. **Advanced Web UI**: No management interface
4. **API**: No structured API for external control

## Planned Improvements Roadmap

### Phase 1: Foundation (Database & API)

#### 1.1 Database Integration
- **Technology**: SQLite with sqlx for async/compile-time verified queries
- **Schema Design**:
  ```sql
  -- Devices table for multi-display support
  CREATE TABLE devices (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    display_count INTEGER DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
  );

  -- Tabs table for URL management
  CREATE TABLE tabs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    persist BOOLEAN DEFAULT false,
    device_id TEXT REFERENCES devices(id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
  );

  -- Playlists for rotation control
  CREATE TABLE playlists (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    interval_seconds INTEGER NOT NULL,
    device_id TEXT REFERENCES devices(id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
  );

  -- Playlist membership
  CREATE TABLE playlist_tabs (
    playlist_id TEXT REFERENCES playlists(id),
    tab_id TEXT REFERENCES tabs(id),
    order_index INTEGER NOT NULL,
    PRIMARY KEY (playlist_id, tab_id)
  );

  -- Display configurations
  CREATE TABLE displays (
    id INTEGER PRIMARY KEY,
    device_id TEXT REFERENCES devices(id),
    name TEXT NOT NULL,
    xrandr_identifier TEXT,
    active_playlist_id TEXT REFERENCES playlists(id),
    backlight_enabled BOOLEAN DEFAULT true
  );
  ```

#### 1.2 API Framework Migration
- **Replace**: Basic poem endpoints with poem-openapi
- **Benefits**: 
  - Auto-generated OpenAPI specs
  - Type-safe request/response handling
  - Automatic TypeScript type generation
  - Built-in validation and documentation

#### 1.3 State Management Refactor
- **Separate**: Configuration from runtime state
- **Implement**: Database-backed state persistence
- **Add**: Configuration hot-reloading

### Phase 2: Multi-Display Support

#### 2.1 Display Detection & Management
- **Implement**: xrandr integration for display enumeration
- **Add**: Per-display Chrome instances
- **Support**: Independent playlist control per display

#### 2.2 Enhanced Chrome Controller
- **Refactor**: Single browser → multiple browser instances
- **Add**: Display-aware tab management
- **Implement**: Cross-display state synchronization

### Phase 3: Advanced Web UI

#### 3.1 Frontend Framework Setup
- **Technology**: React with TypeScript
- **Build Process**: Bundle frontend into binary during compilation
- **API Integration**: Auto-generated TypeScript clients from OpenAPI specs

#### 3.2 Management Interface Features
- **Device Management**: Add/edit devices and displays
- **Tab Management**: CRUD operations for tabs with URL validation
- **Playlist Builder**: Drag-and-drop playlist creation
- **Live Preview**: Real-time display monitoring
- **Configuration**: Runtime configuration updates

#### 3.3 Advanced Features
- **Scheduling**: Time-based playlist switching
- **Monitoring**: Display health and status dashboard
- **Bulk Operations**: Multi-device management
- **Import/Export**: Configuration backup/restore

### Phase 4: Production Enhancements

#### 4.1 Reliability & Monitoring
- **Health Checks**: HTTP endpoints for monitoring
- **Metrics**: Prometheus-compatible metrics
- **Logging**: Structured logging with correlation IDs
- **Recovery**: Automatic restart on browser crashes

#### 4.2 Security & Authentication
- **API Security**: JWT-based authentication
- **HTTPS**: TLS termination for web interface
- **RBAC**: Role-based access control
- **Audit**: Action logging and audit trails

#### 4.3 Deployment & Operations
- **Docker**: Containerized deployment option
- **Configuration**: Environment-based configuration
- **Updates**: In-place update mechanism
- **Backup**: Automated database backups

## Technical Implementation Notes

### Database Migration Strategy
```rust
// migrations/001_initial.sql
// migrations/002_multi_display.sql
// Use sqlx-cli for migration management
```

### API Structure with poem-openapi
```rust
#[derive(ApiResponse)]
enum TabResponse {
    #[oai(status = 200)]
    Ok(Json<Tab>),
    #[oai(status = 404)]
    NotFound,
}

#[OpenApi]
impl TabApi {
    #[oai(path = "/api/tabs", method = "get")]
    async fn list_tabs(&self) -> TabListResponse { ... }
    
    #[oai(path = "/api/tabs", method = "post")]
    async fn create_tab(&self, req: Json<CreateTabRequest>) -> TabResponse { ... }
}
```

### Frontend Build Integration
```rust
// During compilation, embed frontend assets
include!(concat!(env!("OUT_DIR"), "/frontend_assets.rs"));

// Serve from binary
app.at("/app/*").serve_embedded(Frontend::get);
```

### Multi-Display Chrome Management
```rust
pub struct DisplayManager {
    displays: HashMap<String, DisplayController>,
    browsers: HashMap<String, Browser>,
}

impl DisplayController {
    pub async fn switch_playlist(&self, playlist_id: &str) -> Result<()> { ... }
    pub async fn get_screenshot(&self) -> Result<Vec<u8>> { ... }
}
```

## Migration Path

### Immediate Actions (Week 1-2)
1. Add sqlx dependency and basic database setup
2. Create initial migration files
3. Implement basic CRUD operations for tabs/playlists

### Short Term (Month 1)
1. Migrate to poem-openapi
2. Implement database-backed state management
3. Basic frontend scaffolding

### Medium Term (Month 2-3)
1. Multi-display support implementation
2. Complete web UI with management features
3. Enhanced error handling and logging

### Long Term (Month 4+)
1. Advanced features (scheduling, monitoring)
2. Security enhancements
3. Production deployment optimizations

## Current Working Features to Preserve

⚠️ **Critical**: The following must remain functional during migration:
- MQTT Home Assistant integration
- Live preview streams (`/preview_live/:tab_id`)
- Static preview images (`/preview/:tab_id`)
- Basic playlist rotation
- Display power management (backlight control)

## Success Metrics

- [ ] Database persistence replaces in-memory state
- [ ] OpenAPI specification auto-generates TypeScript types
- [ ] Multi-display support works with 2+ monitors
- [ ] Web UI provides complete device management
- [ ] All existing MQTT functionality preserved
- [ ] Frontend bundles into single binary
- [ ] Zero-downtime configuration updates 