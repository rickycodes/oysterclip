# Clipboard Manager

A lightweight, secure, dual-component clipboard history system for Unix/Linux and macOS.

**Watcher** captures clipboard entries to a local encrypted database. **Viewer** provides a searchable UI for browsing history.

## Components

### 📝 Watcher (`packages/watcher`)

A daemon that monitors your system clipboard and persists entries to SQLite with encryption.

**Features:**
- Clipboard polling on 500ms interval
- XChaCha20Poly1305 encryption at rest
- Image storage as PNG blobs
- Automatic text deduplication
- Unix socket control API (pause/resume/status)
- OS keychain integration for secure key storage

```bash
cargo run -p watcher
cargo run -p watcher -- control status
```

See [`packages/watcher/README.md`](packages/watcher/README.md) for details.

### 🖥️ Viewer (`packages/viewer`)

A Dioxus Desktop app for browsing, searching, and managing clipboard history.

**Features:**
- Keyboard-driven navigation (↑↓ hjkl, Home/End)
- Structured search: `type:image`, `kind:url`, `since:1h`
- Copy/delete/clear history
- Dark/light theme
- Password masking with auth caching
- URL previews and clickable links
- Multi-select bulk operations

```bash
cargo run -p viewer
cargo run -p viewer -- --theme light
```

See [`packages/viewer/README.md`](packages/viewer/README.md) for details.

## Quick Start

**Requirements:** Rust 1.70+, GTK 3.24+ (Linux), macOS 10.13+ (macOS)

```bash
# Start the watcher daemon
cargo run -p watcher &

# Launch the viewer UI
cargo run -p viewer
```

## Storage

All files live in the canonical per-user app-data directory (`~/.config/clipboard-manager` on Linux):

| File | Purpose |
|------|---------|
| `.clipboard_history.db` | SQLite history (encrypted text, image blobs) |
| `.clipboard-watcher.toml` | Watcher config (retention, image export) |
| `.clipboard-watcher.sock` | Unix socket for control commands |
| `clipboard_images/` | Optional PNG image export directory |

## Architecture

```
┌─────────────────────────────────────┐
│     System Clipboard                │
└────────────┬────────────────────────┘
             │
     ┌───────▼──────────┐
     │   Watcher (CLI)  │ ← Monitors every 500ms
     │   Daemon/Service │
     └───────┬──────────┘
             │
     ┌───────▼──────────────────────┐
     │  SQLite History Database     │
     │ • Encrypted text entries     │
     │ • Image PNG blobs            │
     │ • Deduplication by hash      │
     └───────┬──────────────────────┘
             │
     ┌───────▼──────────┐
     │ Viewer (UI)      │ ← Search, browse, copy
     │ Dioxus Desktop   │
     └──────────────────┘
```

## Roadmap

See [`ROADMAP.md`](ROADMAP.md) for planned features and architecture improvements.

## Development

**Build all packages:**
```bash
cargo build --workspace
```

**Run tests:**
```bash
cargo test --workspace
```

**Hot reload (development):**
```bash
cargo install dioxus-cli --version 0.7.3 --locked
dx serve --platform desktop
```

## License

See [`LICENSE`](LICENSE) for details.
