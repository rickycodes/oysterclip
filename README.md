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
- Automatically resolves history database location (`--db` flag → env var → canonical default)

**Theming**
- Dark and light mode, toggled from the help panel (?)
- Preference persisted to `config.toml` between sessions (desktop builds)
- `--theme` flag overrides for a single session without touching the saved preference

## Running

```bash
cargo run [-- [OPTIONS]]
```

**Options:**

| Flag | Description |
|------|-------------|
| `--db <PATH>` | Path to the clipboard history database |
| `--theme <dark\|light>` | Override theme for this session (does not persist) |

The database is resolved in this order:
1. `--db <PATH>` flag
2. `CLIPBOARD_HISTORY_DB` environment variable
3. Canonical per-user app-data path (default)

**Examples:**
```bash
cargo run
cargo run -- --db /path/to/.clipboard_history.db
cargo run -- --theme light
CLIPBOARD_HISTORY_DB=/custom/path/.clipboard_history.db cargo run
```

Raw JSON can be passed to `--db` for read-only inspection:
```bash
cargo run -- --db '[{"id":1,...}]'
```

## Configuration

On desktop, persistent preferences are stored in a TOML config file:

| Platform | Path |
|----------|------|
| Linux | `~/.config/clipboard-manager/config.toml` |
| macOS | `~/Library/Application Support/clipboard-manager/config.toml` |
| Windows | `%APPDATA%\clipboard-manager\config.toml` |

**Example `config.toml`:**
```toml
[theme]
mode = "light"   # "dark" or "light" — toggling in the app updates this automatically
```

CLI flags take priority over the config file but do not overwrite it.

## Hot Reload (Development)

Install Dioxus CLI for fast iterative development:
```bash
cargo install dioxus-cli --version 0.7.3 --locked
dx serve --platform desktop
```

## Architecture

| File | Size | Purpose |
|------|------|---------|
| `src/components.rs` | 25K | UI rendering (DetailPane, Sidebar, entry display) |
| `src/app.rs` | 13K | Main app structure, layout, keyboard routing |
| `src/history.rs` | 9.5K | SQLite history reading & decryption |
| `src/app_actions.rs` | 9.5K | Copy, delete, clear, watcher control actions |
| `src/help_modal.rs` | 7.5K | Keyboard shortcuts UI |
| `src/app_state.rs` | 7.4K | AppState signal management, filtering, search |
| `src/auth.rs` | 5.4K | Local auth flow for password reveal |
| `src/format.rs` | 4.9K | Text classification & URL parsing |
| `src/link_preview.rs` | 4.6K | Open Graph metadata fetching |
| `src/watcher_control.rs` | 4.0K | Unix socket IPC for pause/resume |
| `src/source.rs` | 3.1K | Database path resolution |
| `src/theme.rs` | 1.8K | UI theme enum, load/save with config fallback |
| `src/config.rs` | 1.1K | `AppConfig` struct, TOML read/write |
| `src/cli.rs` | 754B | CLI argument definitions (`--db`, `--theme`) |
| `src/paths.rs` | 652B | Platform-specific config/data directory resolution |
| `src/entry.rs` | 909B | Clipboard entry struct |
| `src/main.rs` | 621B | App entry point |
