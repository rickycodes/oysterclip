# Clipboard Watcher

A lightweight Rust daemon that monitors your system clipboard and persists unique clipboard entries to a local SQLite database. It captures both text and images, encrypts text at rest with XChaCha20Poly1305, and automatically deduplicates text content.

**Core Features**
- Clipboard polling on fixed 500ms interval
- Text encryption at rest (XChaCha20Poly1305)
- Image storage as PNG blobs in SQLite
- Automatic text deduplication by content hash
- Configurable max-entry retention (default 500)
- Unix socket control API (pause/resume/status)
- Optional PNG image export to disk
- OS keychain integration for encryption key storage
- Canonical per-user app-data directory for all storage
- Lightweight single binary, no external dependencies

**How It Works**
- Polls the clipboard every `INTERVAL_MS` (see `src/constants.rs`).
- Text entries are deduplicated by content before being appended and encrypted before being stored.
- Image entries are hashed and stored as PNG blobs in SQLite.
- Optional image export to disk is controlled by `.clipboard-watcher.toml` in the app-data directory.

## Storage Layout

All files stored in canonical per-user app-data directory for `clipboard-manager`:

| File | Purpose |
|------|---------|
| `.clipboard_history.db` | SQLite history database (text encrypted, images as PNG blobs) |
| `.clipboard-watcher.toml` | Config file (retention settings, image export options) |
| `.clipboard-watcher.sock` | Unix socket for control commands (pause/resume/status) |
| `clipboard_images/` | Optional PNG image export directory |

**Database Schema** (`entries` table):
- `id`, `created_at`, `entry_type`
- `text_kind` (plain, url, json, multiline)
- `text_ciphertext`, `text_nonce` (encrypted text entries)
- `image_path`, `image_png`, `image_hash` (image storage)
- `content_hash` (for deduplication)

**Build**
```bash
cargo build
```

**Run**
```bash
cargo run
```

**CLI**
```bash
cargo run -- --help
cargo run -- --version
cargo run -- control status
cargo run -- control pause
cargo run -- control resume
```

The watcher creates and loads its text encryption key from the OS keychain.

**Test**
```bash
cargo test
```

## Architecture & Design

### Module Organization

The codebase is organized into domain-focused modules:

```
src/
├── main.rs              (20 lines)  Entry point: CLI parsing → orchestrator
├── app/
│   ├── mod.rs           (115 lines) Orchestrator/Facade pattern
│   └── error.rs         (43 lines)  Custom AppError with type-safe variants
├── config/
│   ├── mod.rs           Module exports
│   ├── cli.rs           CLI argument parsing (clap)
│   ├── settings.rs      TOML config loading & WatcherConfig
│   ├── constants.rs     SQL schema, keyring IDs, defaults
│   └── paths.rs         Canonical app-data path resolution
├── data/
│   ├── mod.rs           Module exports
│   ├── entry.rs         ClipboardEntry struct definitions
│   ├── text.rs          Text classification (plain, url, json, multiline)
│   └── image_store.rs   Image hashing & PNG blob storage
├── history/
│   ├── mod.rs           Module exports
│   ├── store.rs         SQLite operations (insert, dedupe, retention)
│   └── crypto.rs        XChaCha20Poly1305 encryption/decryption & keychain
├── ipc/
│   ├── mod.rs           Shared types & public API (ControlResponse, etc.)
│   ├── server.rs        Unix socket control server (pause/resume/status)
│   └── client.rs        Control command client
└── watcher/
    └── mod.rs           (133 lines) Clipboard polling loop & signal handling
```

### Design Patterns

**Facade/Orchestrator Pattern (app/mod.rs)**
- Centralizes startup, subsystem coordination, and error handling
- Simplifies main.rs to minimal entry point (20 lines)
- Decouples CLI parsing from domain logic
- Improves testability by collecting all initialization logic in one place
- Routes commands (watch, control pause/resume) to appropriate subsystems

**Module Inception**
- Each major domain (config, data, history, ipc, watcher) is its own module with a `mod.rs` declaring submodules
- Provides clear public API boundaries via re-exports
- Enables internal reorganization without affecting external imports

**Custom Error Type**
- `AppError` enum with semantic variants: `HistoryDbFailed`, `ControlSocketFailed`, `ConfigNotFound`, etc.
- Implements `Display`, `std::error::Error`, and `From<io::Error>`
- Type-safe error matching instead of string parsing
- Better error context for debugging

**Signal Handling (Unix-only)**
- SIGTERM/SIGINT caught in background thread via signal-hook crate
- Atomic flag shared with watch loop prevents race conditions
- Graceful shutdown implemented with #[cfg(unix)] gating
- On non-Unix platforms, watcher runs normally but shutdown flag is unused

### Key Design Decisions

**Encryption at Rest**
- Uses XChaCha20Poly1305 (20-byte nonce, 256-bit key)
- Encryption key stored in OS keychain (secure_storage crate)
- Text stored as (ciphertext, nonce) pairs; images are not encrypted (space/perf)
- Decryption happens on-demand when entries are read

**Deduplication Strategy**
- Text entries deduplicated by SHA256 content hash before insert
- Prevents duplicate entries on repeated clipboard activity
- Images deduplicated by SHA256 as well
- Hash computed once at capture time, reused for queries

**IPC for Control Commands**
- Unix socket-based control server (Linux/macOS only)
- Commands: `status` (returns paused state), `pause`, `resume`
- Socket path: `~/.config/clipboard-manager/.clipboard-watcher.sock`
- Cross-platform Windows support planned (item 2.9 in roadmap)

**Polling Architecture**
- Fixed 500ms interval polling keeps CPU usage minimal
- More reliable than event-based clipboard access across platforms
- Atomic pause flag enables responsive pause/resume without loop restart

### Project Statistics

| Layer | Modules | Total LOC |
|-------|---------|-----------|
| Entry point | main.rs | 20 |
| Orchestration | app/ | 158 (mod, error) |
| Config | config/ | ~300 (cli, settings, constants, paths) |
| Data models | data/ | ~200 (entry, text, image_store) |
| History | history/ | ~280 (store, crypto) |
| IPC | ipc/ | ~300 (server, client) |
| Watcher | watcher/ | 133 |
| **Total** | **9 modules** | **~1,400** |
