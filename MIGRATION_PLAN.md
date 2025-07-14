# Mission Control SQLite Migration Plan

## Overview

This document outlines the migration from the current TOML configuration file-based system to a SQLite database for storing Chrome playlists and tabs. The migration will enable dynamic playlist management through the web interface and improve the responsiveness of the Chrome browser control system.

## Current System Analysis

### Current Architecture
- **Backend**: Rust application using async-std, poem HTTP framework, chromiumoxide for Chrome control
- **Frontend**: React TypeScript with Vite, TanStack Query, Tailwind CSS
- **Data Storage**: Static TOML configuration file (`config.toml`)
- **Chrome Control**: Loop-based tab rotation with interrupt mechanism
- **API**: Basic endpoints for reading playlists/tabs and activating them

### Current Data Structure
```toml
[chromium.tabs.tab_id]
url = "https://example.com"
persist = true

[chromium.playlists.playlist_id]
tabs = ["tab1", "tab2"]
interval = 30
```

### Limitations of Current System
1. **Static Configuration**: No runtime modification of playlists/tabs
2. **No Persistence**: Changes require application restart
3. **Limited UI Control**: Frontend can only read and activate existing items
4. **Inflexible Chrome Control**: Loop-based system with basic interrupt mechanism
5. **No Tab Ordering**: Cannot reorder tabs within playlists
6. **No Metadata**: Limited information about playlists and tabs

## Migration Goals

### Primary Objectives
1. **Dynamic Management**: Create, edit, delete playlists and tabs through the web interface
2. **Real-time Updates**: Immediate browser changes without restart
3. **Enhanced Chrome Control**: Responsive message-based Chrome control system
4. **Improved Data Model**: Better structure for playlists, tabs, and metadata
5. **Seeding**: Automatic hello world playlist creation on first setup

### Secondary Objectives
1. **Better API**: RESTful endpoints for full CRUD operations
2. **Validation**: Proper input validation and error handling
3. **Migration Path**: Seamless transition from config file to database
4. **Performance**: Efficient database queries and caching
5. **Extensibility**: Foundation for future features

## Database Design

### Schema

#### Tables

##### `playlists`
```sql
CREATE TABLE playlists (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    interval_seconds INTEGER NOT NULL DEFAULT 30,
    is_active BOOLEAN NOT NULL DEFAULT FALSE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

##### `tabs`
```sql
CREATE TABLE tabs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    persist BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

##### `playlist_tabs`
```sql
CREATE TABLE playlist_tabs (
    playlist_id TEXT NOT NULL,
    tab_id TEXT NOT NULL,
    order_index INTEGER NOT NULL,
    duration_seconds INTEGER, -- Optional per-tab duration override
    PRIMARY KEY (playlist_id, tab_id),
    FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
    FOREIGN KEY (tab_id) REFERENCES tabs(id) ON DELETE CASCADE
);
```

##### `settings`
```sql
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

### Initial Data Seeding

#### Hello World Playlist
```sql
INSERT INTO playlists (id, name, interval_seconds, is_active) VALUES
    ('hello_world', 'Hello World', 30, TRUE);

INSERT INTO tabs (id, name, url, persist) VALUES
    ('welcome', 'Welcome Page', 'https://example.com/welcome', TRUE),
    ('dashboard', 'Dashboard', 'https://example.com/dashboard', TRUE);

INSERT INTO playlist_tabs (playlist_id, tab_id, order_index) VALUES
    ('hello_world', 'welcome', 0),
    ('hello_world', 'dashboard', 1);
```

## Implementation Plan

### Phase 1: Database Infrastructure (Week 1)

#### Backend Changes
1. **Add SQLx Dependency**
   - Add `sqlx` with SQLite feature to `Cargo.toml`
   - Add `sqlx-cli` for migrations

2. **Database Module**
   - Create `app/src/db/mod.rs` with connection management
   - Create `app/src/db/models.rs` with SQLx model structs
   - Create `app/src/db/migrations/` directory

3. **Migration System**
   - Create initial migration files for table creation
   - Add seeding logic for hello world playlist
   - Add config migration logic to import existing TOML data

4. **Database Connection**
   - Initialize SQLite connection in `main.rs`
   - Add database handle to `AppState`
   - Create connection pool for concurrent access

#### Files to Create/Modify
- `app/src/db/mod.rs` - Database connection and utilities
- `app/src/db/models.rs` - SQLx model structs
- `app/src/db/migrations/001_initial.sql` - Initial schema
- `app/src/db/migrations/002_seed_data.sql` - Initial data
- `app/src/state.rs` - Add database pool to AppState
- `app/src/main.rs` - Initialize database connection
- `app/Cargo.toml` - Add SQLx dependencies

### Phase 2: Data Layer (Week 2)

#### Repository Pattern
1. **Create Repository Traits**
   - `PlaylistRepository` for playlist CRUD operations
   - `TabRepository` for tab CRUD operations
   - `PlaylistTabRepository` for playlist-tab relationships

2. **Implement SQLite Repositories**
   - Handle all database operations
   - Include proper error handling
   - Add transaction support for complex operations

3. **Replace Config Loading**
   - Remove dependency on TOML config for playlists/tabs
   - Keep device/display/homeassistant config in TOML
   - Add migration utility to import existing config

#### Files to Create/Modify
- `app/src/db/repositories/mod.rs` - Repository traits
- `app/src/db/repositories/playlist.rs` - Playlist repository
- `app/src/db/repositories/tab.rs` - Tab repository
- `app/src/db/repositories/playlist_tab.rs` - Playlist-tab repository
- `app/src/config.rs` - Separate chromium config from device config

### Phase 3: Enhanced Chrome Control (Week 3)

#### Message-Based Chrome Control
1. **Chrome Message System**
   - Replace loop-based system with message-driven architecture
   - Create message types for playlist/tab operations
   - Implement message queue for Chrome commands

2. **Responsive Tab Switching**
   - Immediate tab activation on API calls
   - Dynamic interval changes
   - Real-time playlist updates

3. **Chrome State Management**
   - Track current playlist/tab state in database
   - Persist Chrome state across restarts
   - Handle Chrome instance failures gracefully

#### Files to Create/Modify
- `app/src/chrome/messages.rs` - Chrome message types
- `app/src/chrome/controller.rs` - Refactored Chrome controller
- `app/src/chrome/mod.rs` - Chrome module reorganization
- `app/src/chrome.rs` - Update to use new architecture

### Phase 4: Enhanced API (Week 4)

#### CRUD API Endpoints
1. **Playlist Management**
   - `POST /playlists` - Create playlist
   - `PUT /playlists/:id` - Update playlist
   - `DELETE /playlists/:id` - Delete playlist
   - `PUT /playlists/:id/reorder` - Reorder tabs

2. **Tab Management**
   - `POST /tabs` - Create tab
   - `PUT /tabs/:id` - Update tab
   - `DELETE /tabs/:id` - Delete tab
   - `POST /playlists/:id/tabs` - Add tab to playlist
   - `DELETE /playlists/:id/tabs/:tab_id` - Remove tab from playlist

3. **Enhanced Status**
   - Real-time status updates
   - WebSocket support for live updates
   - Better error reporting

#### Files to Create/Modify
- `app/src/api/playlists.rs` - Playlist CRUD endpoints
- `app/src/api/tabs.rs` - Tab CRUD endpoints
- `app/src/api/models.rs` - Enhanced API models
- `app/src/api/mod.rs` - Updated API registration

### Phase 5: Frontend Enhancements (Week 5)

#### Enhanced UI Components
1. **Playlist Management**
   - Create/edit/delete playlist forms
   - Drag-and-drop tab reordering
   - Real-time interval adjustment
   - Playlist duplication

2. **Tab Management**
   - Create/edit/delete tab forms
   - URL validation
   - Tab preview improvements
   - Bulk operations

3. **Real-time Updates**
   - WebSocket integration for live updates
   - Optimistic UI updates
   - Better error handling and user feedback

#### Files to Create/Modify
- `web/src/components/PlaylistForm.tsx` - Playlist creation/editing
- `web/src/components/TabForm.tsx` - Tab creation/editing
- `web/src/components/DragDropList.tsx` - Drag-and-drop reordering
- `web/src/hooks/usePlaylistMutations.ts` - Playlist CRUD hooks
- `web/src/hooks/useTabMutations.ts` - Tab CRUD hooks
- `web/src/api/playlists.ts` - Enhanced playlist API calls
- `web/src/api/tabs.ts` - Enhanced tab API calls

### Phase 6: Testing and Polish (Week 6)

#### Testing
1. **Unit Tests**
   - Database repository tests
   - API endpoint tests
   - Chrome controller tests

2. **Integration Tests**
   - End-to-end API tests
   - Database migration tests
   - Chrome integration tests

3. **Frontend Tests**
   - Component tests
   - API integration tests
   - User interaction tests

#### Polish
1. **Error Handling**
   - Comprehensive error messages
   - Graceful fallbacks
   - User-friendly error reporting

2. **Performance**
   - Database query optimization
   - Frontend rendering optimization
   - Memory usage improvements

3. **Documentation**
   - API documentation updates
   - User guide for new features
   - Developer documentation

## Technical Considerations

### Database Performance
- Use connection pooling for concurrent access
- Add indexes for frequently queried columns
- Consider using prepared statements for repeated queries
- Implement proper transaction handling

### Chrome Control Improvements
- Replace polling with event-driven architecture
- Add retry mechanisms for failed operations
- Implement proper cleanup on shutdown
- Add logging for debugging Chrome issues

### API Design
- Use proper HTTP status codes
- Implement request validation
- Add rate limiting for API endpoints
- Use consistent error response format

### Frontend Architecture
- Implement proper state management
- Add loading states for all operations
- Use optimistic updates where appropriate
- Add proper error boundaries

## Migration Strategy

### Backward Compatibility
1. **Config File Support**
   - Keep existing config file loading during transition
   - Add migration utility to import existing data
   - Provide fallback to config file if database fails

2. **Gradual Migration**
   - Phase 1: Database infrastructure with config fallback
   - Phase 2: Database-first with config backup
   - Phase 3: Full database migration with config removal

### Data Migration
1. **Import Existing Data**
   - Parse existing TOML configuration
   - Create corresponding database entries
   - Preserve playlist/tab relationships and settings

2. **Validation**
   - Validate all imported data
   - Handle duplicate IDs gracefully
   - Provide migration report

## Risk Assessment

### High Risk
- **Chrome Integration**: Complex interaction with chromiumoxide
- **Database Transactions**: Ensuring data consistency
- **Migration Complexity**: Seamless transition from config file

### Medium Risk
- **Performance**: Database queries under load
- **Error Handling**: Graceful degradation
- **Frontend Complexity**: Drag-and-drop implementation

### Low Risk
- **API Implementation**: Standard CRUD operations
- **UI Components**: Well-established patterns
- **Testing**: Standard testing approaches

## Success Metrics

### Functional Goals
- [ ] Create playlists through web interface
- [ ] Edit playlist names and intervals
- [ ] Reorder tabs within playlists
- [ ] Create and edit tabs
- [ ] Delete playlists and tabs
- [ ] Immediate browser updates
- [ ] Hello world playlist auto-creation

### Technical Goals
- [ ] Database migration from config file
- [ ] Responsive Chrome control system
- [ ] Comprehensive API coverage
- [ ] Real-time frontend updates
- [ ] Proper error handling
- [ ] Performance benchmarks met

### User Experience Goals
- [ ] Intuitive playlist management
- [ ] Drag-and-drop tab reordering
- [ ] Real-time browser updates
- [ ] Clear error messages
- [ ] Fast response times
- [ ] Reliable operation

## Conclusion

This migration plan provides a comprehensive roadmap for transitioning from a static configuration file system to a dynamic SQLite-based system. The phased approach ensures minimal disruption while delivering significant improvements in functionality and user experience.

The key benefits of this migration include:
- Dynamic playlist and tab management
- Real-time browser control
- Enhanced user interface
- Better data persistence
- Improved system architecture
- Foundation for future features

The estimated timeline of 6 weeks allows for thorough implementation, testing, and polish while maintaining system stability throughout the transition. 