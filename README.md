# Clipboard Watcher

A small Rust daemon that monitors your system clipboard and persists unique clipboard entries to disk. It records both text and images, and avoids storing duplicate text content.

**Features**
- Watches the clipboard on a fixed interval.
- Persists unique text entries to a local history file.
- Saves image clipboard entries as PNG files.
- Lightweight, single binary.

**How It Works**
- Polls the clipboard every `INTERVAL_MS` (see `src/common.rs`).
- Text entries are deduplicated by content before being appended.
- Image entries are hashed and saved as PNGs under a local image directory.

**Files Created**
- `.clipboard_history.json.gpg` in the working directory.
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

To enable encrypted history writes, set `CLIPBOARD_WATCHER_GPG_RECIPIENT` or create
`.clipboard-watcher.toml` in the project directory with:

```toml
gpg_recipient = "your-key-id-or-email"
```

**Test**
```bash
cargo test
```

**Project Layout**
- `src/main.rs` main loop and clipboard polling.
- `src/config.rs` config resolution for the GPG recipient.
- `src/history.rs` encrypted history persistence and timestamps.
- `src/image_store.rs` image hashing and PNG persistence.
- `src/text.rs` clipboard text classification.
- `src/common.rs` shared constants and types.
