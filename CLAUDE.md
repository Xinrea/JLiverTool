# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

JLiverTool is a Bilibili live streaming tool that displays danmaku (chat messages), gifts, and superchats. The project is being refactored from Electron/TypeScript to native Rust using the GPUI framework.

## Build Commands

```bash
# Build all crates
cargo build

# Build release version
cargo build --release

# Run the application
cargo run -p jlivertool

# Run with logging
RUST_LOG=jlivertool=info cargo run -p jlivertool

# Run tests
cargo test

# Run tests for a specific crate
cargo test -p jlivertool-core
```

## Architecture

### Crate Structure

The project uses a Cargo workspace with three crates:

- **jlivertool** (`crates/jlivertool/`) - Main binary, orchestrates backend and UI
- **jlivertool-core** (`crates/jlivertool-core/`) - Core library with Bilibili API, WebSocket, config, database
- **jlivertool-ui** (`crates/jlivertool-ui/`) - GPUI-based UI components and views

### Communication Flow

```
┌─────────────────┐     mpsc::channel<Event>      ┌─────────────────┐
│                 │ ─────────────────────────────▶│                 │
│  Backend        │                               │  UI (GPUI)      │
│  (Tokio async)  │                               │  (Main thread)  │
│                 │ ◀─────────────────────────────│                 │
└─────────────────┘     mpsc::channel<UiCommand>  └─────────────────┘
```

- **Events** (`jlivertool-core/src/events.rs`): Backend → UI communication (danmu, gifts, connection status, etc.)
- **UiCommand** (`jlivertool-ui/src/app.rs`): UI → Backend commands (login, change room, send danmu, etc.)

### Key Components

**jlivertool-core:**
- `bilibili/api.rs` - REST API client for Bilibili (login, room info, danmu info)
- `bilibili/ws.rs` - WebSocket client for real-time danmaku stream
- `bilibili/wbi.rs` - WBI signature for authenticated API requests
- `config.rs` - JSON config persistence with change notifications
- `database.rs` - SQLite storage for danmus, gifts, guards, superchats
- `messages.rs` - Message types parsed from WebSocket (DanmuMessage, GiftMessage, etc.)

**jlivertool-ui:**
- `app.rs` - GPUI application entry point
- `views/main_view.rs` - Main window with danmu list
- `views/setting_view.rs` - Settings window with login, room config
- `views/gift_view.rs` - Gift records window
- `views/superchat_view.rs` - Superchat records window
- `platform.rs` - Platform-specific APIs (macOS window level for always-on-top)
- `theme.rs` - Theme management (dark/light)

### WebSocket Protocol

Bilibili danmaku WebSocket uses a binary protocol:
- Packet: `[packetLen(4)][headerLen(2)][ver(2)][op(4)][seq(4)][body]`
- Ver 0: Normal JSON, Ver 2: Deflate compressed, Ver 3: Brotli compressed
- Operations: Auth (7), AuthReply (8), KeepAlive (2), KeepAliveReply (3), SendMsgReply (5)

### GPUI Patterns

- Views implement `Render` trait and use `cx.new()` to create entities
- Use `cx.notify()` to trigger re-renders
- Use `cx.listener()` for event handlers that need mutable access to view
- Wrap views with `gpui_component::Root` for input component support
- Timer-based polling for event processing (100ms interval in MainView)

## Dependencies

Key dependencies:
- **gpui 0.2.2** / **gpui-component 0.5.0** - UI framework
- **tokio** - Async runtime for backend
- **reqwest** - HTTP client
- **tokio-tungstenite** - WebSocket client
- **rusqlite** (bundled) - SQLite database
- **objc/cocoa** - macOS native APIs (for always-on-top)

## Special Commands

The danmu input supports special commands:
- `/title <new title>` - Update room title
- `/bye` - Stop live streaming
