# Clipboard Viewer

A Dioxus Desktop app for browsing, searching, and managing clipboard history captured by `clipboard-watcher`.

## Features

**Navigation & Search**
- Keyboard shortcuts: ↑↓ (hjkl), Home/End for navigation
- Structured search filters: `type:image`, `type:text`, `type:password`, `kind:url`, `kind:json`
- Free-text search on content, entry type, or image path (case-insensitive)
- Combine filters with free-text search (e.g., `type:image my folder`, `kind:url github`)
- Escape to clear search or close overlays

**Clipboard Actions**
- Copy selected text entry back to clipboard (Enter/y)
- Delete single entries or clear entire history (Delete/Backspace/d)
- Pause/resume watcher daemon from the UI (Unix only)

**Content Display**
- Image preview rendering from PNG blobs with fallback to local paths
- Password-like text masking with Show/Hide button + short-lived local auth cache
- Clickable URLs that open in your default browser
- Open Graph link previews (fetches title, description, image)
- Text classification labels (plain text, URL, JSON, multiline)

**History Management**
- Polls clipboard history every 500ms for live updates
- Polls watcher status every 1000ms
- Automatically resolves history database location (argument → env var → canonical default)

## Running

The viewer automatically discovers your clipboard history database in this order:
1. First positional argument (path to `.clipboard_history.db`)
2. `CLIPBOARD_HISTORY_DB` environment variable
3. Canonical per-user app-data path for `clipboard-manager` (default)

**Examples:**
```bash
cargo run
cargo run -- /path/to/.clipboard_history.db
CLIPBOARD_HISTORY_DB=/custom/path/.clipboard_history.db cargo run
```

Raw JSON input is also supported as the first argument for read-only inspection:
```bash
cargo run -- '/path/to/export.json'
```

## Hot Reload (Development)

Install Dioxus CLI for fast iterative development:
```bash
cargo install dioxus-cli --version 0.7.3 --locked
dx serve --platform desktop
```

## Architecture

| File | LOC | Purpose |
|------|-----|---------|
| `src/components.rs` | 24K | UI rendering (DetailPane, Sidebar, entry display) |
| `src/app.rs` | 12K | Main app structure, layout, keyboard routing |
| `src/history.rs` | 9.5K | SQLite history reading & decryption |
| `src/app_state.rs` | 7.4K | AppState signal management, filtering, search |
| `src/app_actions.rs` | 7.9K | Copy, delete, clear, watcher control actions |
| `src/link_preview.rs` | 4.6K | Open Graph metadata fetching |
| `src/auth.rs` | 5.4K | Local auth flow for password reveal |
| `src/watcher_control.rs` | 4.0K | Unix socket IPC for pause/resume |
| `src/help_modal.rs` | 4.4K | Keyboard shortcuts UI |
| `src/format.rs` | 3.7K | Text classification & URL parsing |
| `src/source.rs` | 3.2K | Database path resolution |
| `src/theme.rs`, `src/entry.rs`, `src/main.rs` | 2.8K | UI theme, entry struct, entry point |
