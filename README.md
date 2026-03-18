# Clipboard Watcher

A small Rust daemon that monitors your system clipboard and persists unique clipboard entries to disk. It records both text and images, and avoids storing duplicate text content.

**Features**
- Watches the clipboard on a fixed interval.
- Persists clipboard history to a local SQLite database.
- Saves image clipboard entries as PNG files.
- Lightweight, single binary.

**How It Works**
- Polls the clipboard every `INTERVAL_MS` (see `src/common.rs`).
- Text entries are deduplicated by content before being appended and encrypted before being stored.
- Image entries are hashed and saved as PNGs under a local image directory.

**Files Created**
- `.clipboard_history.db` in the working directory.
- `clipboard_images/` in the working directory, containing `img_<hash>.png`.

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
- `src/common.rs` shared constants and types.
