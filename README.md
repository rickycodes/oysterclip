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

**Project Layout**

| File | LOC | Purpose |
|------|-----|---------|
| `src/main.rs` | 7.6K | Main clipboard polling loop |
| `src/history.rs` | 11K | SQLite operations (insert, dedupe, encrypt, retention) |
| `src/ipc.rs` | 7.4K | Unix socket control server |
| `src/config.rs` | 3.7K | TOML config loading |
| `src/image_store.rs` | 2.3K | Image hashing & PNG storage |
| `src/text.rs` | 1.6K | Text classification (plain, url, json, multiline) |
| `src/paths.rs` | 951B | Canonical app-data path resolution |
| `src/constants.rs` | 2.7K | SQL schema & keyring IDs |
| `src/cli.rs` | 3.2K | Argument parsing |
| `src/entry.rs` | 350B | Entry struct definitions |
