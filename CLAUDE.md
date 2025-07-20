# Arrow - Tauri Migration Plan

## Overview
Migration plan for converting the Arrow posture monitoring application from Iced (Rust GUI) to Tauri (web-based frontend with Rust backend).

## Current Architecture Analysis

### Iced Application Components
- **TCP subscription system**: Real-time server communication (`main.rs:310-369`)
- **Posture detection logic**: State management and posture classification
- **Database operations**: SQLite logging (`db_manager.rs`)
- **Desktop notifications**: `notify-rust` integration
- **GUI components**: SVG rendering, session history display

### Current Dependencies
```toml
iced = { version = "0.10", features = ["tokio", "svg"] }
tokio = { version = "1", features = ["full"] }
notify-rust = "4"
rusqlite = { version = "0.35.0", features = ["bundled"] }
dirs = "5.0.1"
```

## Target Tauri Architecture

### Backend (src-tauri/src/)
**Responsibilities:**
- TCP connection management with Python server
- Posture detection and classification logic
- SQLite database operations
- Desktop notifications
- Background services for real-time data processing

**Key Files to Create:**
- `tcp_client.rs` - Server connection management
- `posture_service.rs` - Core posture logic
- `db_manager.rs` - Database operations (migrated)
- `notifications.rs` - Desktop notification handling

### Frontend (Web Interface)
**Responsibilities:**
- Real-time posture display with SVG icons
- Session history visualization
- Connection status indicators
- User interface interactions

**Technologies:**
- HTML/CSS/JavaScript or modern framework (React, Vue, etc.)
- SVG rendering for posture icons
- Real-time updates via Tauri events

### Communication Layer
**Tauri Commands (Frontend → Backend):**
- `get_session_logs()` - Fetch posture history
- `get_connection_status()` - Check server connection state
- `initialize_tcp_connection()` - Start/restart server connection

**Tauri Events (Backend → Frontend):**
- `posture-update` - New posture data received
- `connection-status-changed` - TCP connection state changes
- `session-logs-updated` - Updated history data
- `notification-triggered` - Notification events

## Migration Steps

### Phase 1: Backend Foundation
1. **Setup Tauri project structure**
   - Configure `src-tauri/Cargo.toml` with required dependencies
   - Set up Tauri permissions for notifications and networking

2. **Migrate core modules**
   - Move `postures.rs` to `src-tauri/src/`
   - Adapt `db_manager.rs` for Tauri backend
   - Create TCP client service

3. **Implement Tauri commands**
   - Database query commands
   - Connection management commands

### Phase 2: Real-time Communication
1. **Background TCP service**
   - Convert Iced subscription to background tokio task
   - Implement event emission for posture updates
   - Handle connection failures and reconnection

2. **Event system setup**
   - Define event payloads and types
   - Implement event listeners in frontend

### Phase 3: Frontend Development
1. **Basic UI structure**
   - Posture display component
   - Connection status indicator
   - Layout and styling

2. **Asset migration**
   - Move SVG files to `public/` directory
   - Update asset references for web context

3. **Real-time updates**
   - Event listeners for posture changes
   - Dynamic UI updates based on backend events

### Phase 4: Feature Completion
1. **Session history**
   - History display component
   - Data fetching and rendering

2. **Notifications**
   - Desktop notification integration
   - Notification permission handling

3. **Error handling**
   - Connection error states
   - User feedback for failures

### Phase 5: Testing & Polish
1. **Integration testing**
   - End-to-end posture detection flow
   - Database operations validation
   - Notification system testing

2. **UI/UX improvements**
   - Responsive design
   - Performance optimization
   - Cross-platform compatibility

## Critical Migration Challenges

### Real-time Data Flow
**Current**: Iced subscription directly manages TCP connection
**Target**: Backend service maintains connection, emits events to frontend
**Solution**: Background tokio task with event emission

### State Management
**Current**: Single `Arrow` struct holds all application state
**Target**: Backend state + frontend state synchronization
**Solution**: Clear separation of concerns with event-driven updates

### Desktop Notifications
**Current**: Direct `notify-rust` integration in UI thread
**Target**: Backend triggers notifications, frontend shows status
**Solution**: Tauri notification API with proper permissions

## Assets & Configuration

### Files to Migrate
- `src/assets/bad_posture.svg` → `public/assets/`
- `src/assets/good_posture.svg` → `public/assets/`

### Dependencies Update
```toml
# src-tauri/Cargo.toml
[dependencies]
tauri = { version = "1.0", features = ["api-all"] }
tokio = { version = "1", features = ["full"] }
rusqlite = { version = "0.35.0", features = ["bundled"] }
dirs = "5.0.1"
serde = { version = "1.0", features = ["derive"] }
notify-rust = "4"
```

### Tauri Permissions
```json
// src-tauri/tauri.conf.json
{
  "tauri": {
    "allowlist": {
      "notification": {
        "all": true
      },
      "fs": {
        "all": true
      }
    }
  }
}
```

## Success Criteria
- [ ] Real-time posture monitoring functionality preserved
- [ ] Desktop notifications working across platforms
- [ ] Session history and database logging maintained
- [ ] TCP connection resilience and auto-reconnection
- [ ] Cross-platform compatibility (Windows, macOS, Linux)
- [ ] Performance comparable to or better than Iced version