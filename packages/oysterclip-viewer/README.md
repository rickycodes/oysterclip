# Clipboard Viewer

A Dioxus Desktop app for browsing, searching, and managing clipboard history captured by `clipboard-watcher`.

## Features

**Navigation & Search**
- Keyboard shortcuts: ↑↓ (hjkl), Home/End for navigation
- Structured search filters: `type:image`, `type:text`, `type:password`, `kind:url`, `kind:json`, `kind:path`
- Date/time filters: `since:1h`, `since:2d`, `since:today`, `since:yesterday`
- Free-text search on content, entry type, or image path (case-insensitive)
- Combine filters with free-text search (e.g., `type:image my folder`, `kind:url github`, `kind:path since:today`)
- Escape to clear search or close overlays

**Clipboard Actions**
- Copy selected text entry back to clipboard (Enter/y)
- Delete single entries or clear entire history (Delete/Backspace/d)
- Multi-select with Space / Shift+↑↓, bulk delete selected entries
- Pause/resume watcher daemon from the UI (Unix only)

**Content Display**
- Per-type colored left-border accents in both sidebar cards and detail pane: JSON (amber), path (teal), URL (blue), password (orange), image (purple)
- Image preview rendering from PNG blobs with fallback to local paths
- Password-like text masking with Show/Hide button + short-lived local auth cache
- Clickable URLs that open in your default browser
- File path entries rendered as clickable links (opens with `xdg-open` / `open`)
- Open Graph link previews (fetches title, description, image)
- JSON entries pretty-printed in the detail pane
- Text classification labels: Text, Link, JSON, Path, Pass, Image

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
cargo run -- --db /path/to/.oysterclip.db
cargo run -- --theme light
CLIPBOARD_HISTORY_DB=/custom/path/.oysterclip.db cargo run
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

### Design Philosophy

The viewer follows a **reactive signal-based architecture** using Dioxus:
- Central `AppState` signal holds filtered/searched history and UI state
- Components reactively subscribe to state changes without manual re-renders
- Actions (copy, delete, pause) dispatch to AppState and optionally trigger side effects
- Clear separation: UI layer (components) → state management (app_state) → business logic (actions)

### Module Organization

```
src/
├── main.rs                Main Dioxus app initialization
├── app/
│   ├── root.rs           (App component, main layout, keyboard routing)
│   ├── state.rs          (AppState signal: history, search filters, selection)
│   ├── actions.rs        (Copy, delete, clear, watcher control)
│   ├── keyboard_shortcuts.rs (Keyboard event handling with parameter grouping)
│   ├── selection.rs      (Multi-select state management)
│   └── query.rs          (Search filter parsing and application)
├── config/
│   ├── cli.rs            (clap argument definitions)
│   ├── source.rs         (Database path resolution: CLI → env → default)
│   ├── paths.rs          (Platform-specific config/data directories)
│   ├── settings.rs       (AppConfig TOML struct and persistence)
│   └── help.rs           (Help modal content and keyboard reference)
├── data/
│   ├── entry.rs          (ClipboardEntry struct, shared with watcher)
│   ├── history.rs        (SQLite reads, filtering, decryption)
│   ├── link_preview.rs   (Open Graph metadata fetching)
│   └── format/
│       ├── classification.rs (Text/URL/JSON/Path/Password detection)
│       ├── text_type.rs     (Text classification, multiline detection)
│       ├── url.rs           (URL parsing and validation)
│       ├── image.rs         (Image type detection, PNG handling)
│       └── timestamp.rs     (Relative timestamp formatting)
├── system/
│   └── watcher_control.rs (Unix socket IPC: pause/resume/status)
└── ui/
    ├── root_view.rs      (RSX macro for main view layout)
    ├── sidebar.rs        (Entry list cards with colored accents)
    ├── detail/
    │   ├── mod.rs        (DetailPane component with state)
    │   ├── text.rs       (Text rendering with links and masking)
    │   ├── image.rs      (Image display: blobs or file paths)
    │   └── empty.rs      (Empty state view)
    ├── search_bar.rs     (Filter input with live updates)
    ├── help_modal.rs     (Help overlay with shortcuts)
    ├── theme.rs          (Dark/light mode toggle with persistence)
    ├── linkable_text.rs  (URL-aware text component)
    ├── image_overlay.rs  (Fullscreen image viewer)
    └── icon.rs           (Entry type icons)
```

### Key Design Decisions

**Reactive State Management**
- Uses Dioxus signals (`use_signal`) instead of Redux-like state containers
- AppState holds: history entries, filtered results, search query, selected index, watcher status
- Components subscribe to specific signals, only re-render on changes
- Eliminates boilerplate compared to action/reducer patterns

**Filtering & Search**
- Search filters parsed into structured types: `FilterType::Text`, `FilterType::Image`, `FilterKind::Url`, `DateFilter`
- Filters AND-combined; free-text search applied to all fields
- Efficient in-memory filtering (database queried once on startup + polling)

**Password Reveal Flow**
- Shows masked text by default for entries classified as passwords
- Local auth caching: user enters password once, cached for 5 min (or configurable)
- Cache cleared on app focus loss (defensive)
- No server round-trip; all auth happens locally

**URL Previews**
- Open Graph fetching: title, description, image from remote URLs
- Fails gracefully with fallback to URL text
- Cached per URL to avoid repeated fetches
- Clickable URLs open in default browser

**Watcher Control**
- Unix socket client to watcher daemon
- Commands: `pause`, `resume`, `status` (checks if paused)
- Status polled every 1000ms; UI shows pause state
- Cross-platform: fails gracefully on Windows (buttons disabled)

**Configuration Resolution**
- CLI flags > environment variables > config file > defaults
- Config persisted to TOML but CLI flags don't overwrite it
- Separate runtime config (theme override) vs. persistent config

**Database Access**
- Reads only; no modifications except via clipboard-watcher
- Supports both SQLite and raw JSON input (for testing)
- Decryption happens on-demand using watcher's keychain key

### Performance Considerations

- **Initial load**: Full history read from database (~500 entries in <100ms)
- **Polling**: History polled every 500ms, status every 1000ms
- **Filtering**: In-memory filter + search (O(n) but fast for <1000 entries)
- **Images**: Rendered from PNG blobs; large images may stall UI (deferred rendering considered)
- **Link previews**: Fetched asynchronously; timeout after 5s

### Component Hierarchy

```
App (main layout, keyboard handler)
├── HelpModal (toggleable, overlays content)
├── Sidebar (entry list cards, clickable)
│   └── EntryCard (colored border, text preview)
├── DetailPane (selected entry details)
│   ├── PasswordReveal (if classified as password)
│   ├── LinkPreview (if URL)
│   ├── ImageDisplay (PNG or file preview)
│   └── JsonPrettyPrint (if JSON)
└── SearchBar (filter input, live update)
