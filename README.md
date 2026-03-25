# Clipboard Watcher

A small Rust daemon that monitors your system clipboard and persists unique clipboard entries to a local SQLite history. It records both text and images, encrypts text at rest, and avoids storing duplicate text content.

**Features**
- Watches the clipboard on a fixed interval.
- Persists clipboard history to a local SQLite database.
- Encrypts text entries before storing them.
- Stores image entries in the database and can optionally export PNG files to disk.
- Supports unix watcher control commands for pause, resume, and status.
- Uses a canonical per-user app-data directory for its default database, config, image export, and socket paths.
- Lightweight, single binary.

**How It Works**
- Polls the clipboard every `INTERVAL_MS` (see `src/constants.rs`).
- Text entries are deduplicated by content before being appended and encrypted before being stored.
- Image entries are hashed and stored as PNG blobs in SQLite.
- Optional image export to disk is controlled by `.clipboard-watcher.toml` in the app-data directory.

**Default Storage Location**
- Base directory: the per-user app-data directory for `clipboard-manager`
- History database: `.clipboard_history.db`
- Config file: `.clipboard-watcher.toml`
- Unix control socket: `.clipboard-watcher.sock`
- Image export directory: `clipboard_images/`

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

**Project Layout**
- `src/main.rs` main loop and clipboard polling.
- `src/history.rs` SQLite-backed history persistence, encryption, and timestamps.
- `src/image_store.rs` image hashing and PNG persistence.
- `src/text.rs` clipboard text classification.
- `src/ipc.rs` unix control socket handling.
- `src/paths.rs` canonical app-data path resolution.
- `src/constants.rs` shared constants and SQL statements.
