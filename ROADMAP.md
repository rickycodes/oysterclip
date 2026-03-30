# Clipboard Viewer + Watcher — Roadmap

The watcher/viewer pair is stable and feature-complete for core use. This roadmap covers the next
round of work: stabilizing the shared storage contract, cleaning up data lifecycle issues, and
adding high-value UX improvements.

---

## Architecture

![Architecture diagram](docs/architecture.png)

---

## Tier 1 — Foundation

*These make later changes safer and remove known correctness problems.*

### 1.0 Workspace consolidation ✅ COMPLETE

**Goal:** Consolidate separate repos into a single Rust workspace to enable code sharing, unified versioning, and easier coordination.

**Status:** ✅ COMPLETE - All work done
- ✅ Moved watcher code to `packages/watcher/` (preserving git move history)
- ✅ Merged viewer with full commit history to `packages/viewer/` 
- ✅ Created `packages/common/` with shared modules:
  - `paths.rs` - Canonical app directory resolution (used by both watcher and viewer)
  - `constants.rs` - Shared constants, DB schema, keyring IDs, socket file names
  - `crypto.rs` - XChaCha20Poly1305 encryption/decryption, keychain integration
  - `ipc.rs` - Control protocol types and socket paths
- ✅ Updated watcher to use `common::paths`
- ✅ Updated viewer to use `common::paths` and `common::constants`
- ✅ Root workspace manages unified versioning (0.1.0) and release profiles
- ✅ Both packages build successfully with no clippy warnings

**Final Structure:**
```
clipboard-manager/
├── Cargo.toml                 (workspace: common, watcher, viewer)
├── Cargo.lock
├── README.md
├── ROADMAP.md
├── docs/
│   └── architecture.png
│
└── packages/
    ├── common/                ⭐ Shared crate
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── paths.rs       (app paths, config dir resolution)
    │       ├── constants.rs   (HISTORY_FILE, CONFIG_FILE, SOCKET_FILE, DB schema)
    │       ├── crypto.rs      (XChaCha20-Poly1305, keychain setup)
    │       └── ipc.rs         (ControlCommand, ControlResponse enums)
    │
    ├── watcher/               (CLI daemon, ~1.4K LOC)
    │   ├── Cargo.toml         (depends on: common)
    │   └── src/
    │
    └── viewer/                (Dioxus UI, ~0.7K LOC)
        ├── Cargo.toml         (depends on: common)
        └── src/
```

**Work Completed:**
1. Consolidated both repos with full history (git subtree)
2. Extracted common paths/constants/crypto/ipc into shared crate
3. Both packages updated to import from common
4. All clippy warnings resolved
5. Workspace builds cleanly

**Next:** 1.1 - Share data types (ClipboardEntry) and improve test coverage

### 1.1 Shared data types and error handling

**Goal:** Extract common data structures and error types into `packages/common/`.

**Design: Three-layer separation**

Layer 1: **StorageEntry** (DB schema exactly)
```rust
pub struct StorageEntry {
    pub id: i64,
    pub created_at: u64,
    pub entry_type: EntryType,
    pub text_kind: Option<String>,
    pub text_ciphertext: Option<Vec<u8>>,
    pub text_nonce: Option<Vec<u8>>,
    pub image_path: Option<String>,
    pub image_png: Option<Vec<u8>>,
    pub image_hash: Option<u64>,
    pub content_hash: Option<String>,
}
```

Layer 2: **CommonEntry** (minimal, shared across apps)
```rust
pub enum CommonEntry {
    Text {
        id: i64,
        timestamp: u64,
        content: String,
        kind: Option<String>,
    },
    Image {
        id: i64,
        timestamp: u64,
        path: Option<String>,
        hash: u64,
    },
}
```

Layer 3: **App-specific** (in each package)
- `watcher/src/data/entry.rs` — PasteEntry (working model during capture)
- `viewer/src/data/entry.rs` — ClipboardEntry (UI model with data_url, source)

**What goes into packages/common:**
- `storage.rs` — StorageEntry, DB schema, path resolution, migrations
- `entry.rs` — CommonEntry enum, entry type definitions
- `crypto.rs` — encrypt/decrypt, keychain integration
- `ipc.rs` — IPC protocol types, socket paths
- `constants.rs` — SQL schema, keyring IDs, defaults
- `errors.rs` — AppError enum, Result type alias
- Conversion functions: StorageEntry ↔ CommonEntry

**Rationale:**
- Each app evolves independently (watcher doesn't bloat with UI concerns)
- DB schema stays in sync (StorageEntry is source of truth)
- Clean conversion boundaries between layers
- Watcher works with PasteEntry (no UI baggage), viewer works with ClipboardEntry (UI-enriched)

**Status:** ✅ COMPLETE - Option B design implemented
- ✅ Created packages/common/ crate with entry.rs, storage.rs, crypto.rs, ipc.rs, constants.rs, errors.rs
- ✅ Implemented StorageEntry matching DB schema exactly
- ✅ Created CommonEntry as minimal shared enum
- ✅ Updated watcher/src/data/entry.rs to use PasteEntry (app-specific)
- ✅ Updated viewer/src/data/entry.rs to use ClipboardEntry (app-specific)
- ✅ Added conversion utilities between layers
- ✅ Updated both packages to depend on common crate
- ✅ All imports updated throughout both packages

### 1.2 Image lifecycle cleanup

Deleting an entry or pruning old rows leaves orphaned PNG files on disk forever.
- Watcher: when retention prunes rows with `image_path`, delete the file
- Viewer: when user deletes an image entry with `image_path`, delete the file
- Safeguard: only delete files inside the known `clipboard_images/` directory

**Status:** 🔄 Not started

### 1.3 Contract-level integration tests

After the shared crate exists, add tests that exercise the full round-trip:
- Encryption/decryption round-trip across both crates
- Path resolution consistency (argument → env → default)
- Pause/status/resume IPC socket protocol compatibility

**Status:** 🔄 Not started

---

## Tier 2 — Bug Fixes & Correctness

### ~~2.1 Password preview leaks in sidebar~~ *(not an issue)*
Already correctly masked via `preview_text()` — no action needed.

### 2.2 Watcher socket auto-recovery
If the viewer starts before the watcher, or the watcher restarts, the socket state goes
`unavailable` and never recovers without restarting the viewer.
- Treat `unavailable` as retriable on each poll cycle
- Reconnect automatically when the socket becomes available again

### 2.3 Link preview retry
Failed link previews show a permanent "Failed" badge with no way to retry.
- Re-attempt on next navigation to the entry
- Optionally: add a small retry button in the detail pane

### 2.4 URL detection edge cases
The regex used for link detection and clickable URL rendering doesn't handle several common patterns.
- Strip trailing punctuation (`.`, `,`, `)`) that gets absorbed into matched URLs
- Handle bare `www.` URLs without a scheme (render as `https://www....`)
- Affects sidebar link label, detail pane hyperlinking, and link preview trigger

### ~~2.5 Watcher graceful shutdown~~ ✅ *COMPLETED*
Signal handling for SIGTERM/SIGINT implemented with signal-hook crate.
- ✅ Capture SIGTERM and SIGINT signals (background thread with atomic flag)
- ✅ Stop the watch loop gracefully 
- ✅ Print shutdown message before exit
- ✅ Gated with #[cfg(unix)] for cross-platform compatibility

### 2.6 Watcher logging framework
Replace `println!` / `eprintln!` with structured logging for better observability.
- Integrate `tracing` or `log` crate
- Support verbosity levels: error, warn, info, debug, trace
- Control via `--verbose` flag or env var (`CLIPBOARD_WATCHER_LOG_LEVEL`)
- Include timestamps and structured context in logs
- Easier for monitoring, debugging, and production support

### 2.7 Watcher configuration validation
Configuration values are not validated at load time.
- Validate database path is writable and parent directory exists
- Validate image export directory is writable
- Validate history retention count is > 0
- Early errors on startup rather than silent failures during operation

### 2.8 Watcher environment variable config overrides
Support environment variables to override config file values for containerization.
- `CLIPBOARD_WATCHER_DB_PATH` overrides database path
- `CLIPBOARD_WATCHER_IMAGE_DIR` overrides image export directory
- `CLIPBOARD_WATCHER_MAX_HISTORY` overrides retention limit
- Follow XDG Base Directory spec for default paths

### 2.9 Cross-platform watcher control (IPC)
Currently IPC is Unix-only (Unix sockets). Windows users can capture but not pause/resume.
Implement cross-platform control mechanism for pause/resume/status.
- Windows: Named pipes or HTTP control server on localhost
- Unix: Keep existing Unix socket approach (or unify with HTTP)
- Single CLI interface: `clipboard-watcher control --pause/--resume/--status`
- Enables full feature parity across platforms

### 2.10 Watcher code modularization ✅ *COMPLETED*
Monolithic codebase refactored into domain-focused modules.
- ✅ Split ipc.rs into ipc/{mod.rs, server.rs, client.rs}
- ✅ Split history.rs into history/{mod.rs, crypto.rs, store.rs}
- ✅ Reorganized top-level files into config/ and data/ modules
- ✅ Created app/mod.rs orchestrator/facade layer (main.rs → 20 lines)
- ✅ Extracted watcher loop into watcher/mod.rs
- ✅ Reduced cyclomatic complexity, improved testability and maintainability

### 2.11 Watcher type-safe error handling ✅ *COMPLETED*
Replaced generic io::Error with semantic error types.
- ✅ Created app/error.rs with AppError enum (8 semantic variants)
- ✅ Type-safe error matching instead of string parsing
- ✅ Better error context for debugging and logging

---

## Tier 3 — UX Quick Wins

*Low-effort, high-impact polish.*

### ✅ 3.1 Relative timestamps in sidebar *(done)*
- Shows `just now`, `5m ago`, `2h ago`, `yesterday`, day name, or `Mar 20` in sidebar rows
- Full timestamp kept in the detail pane

### ✅ 3.2 Keyboard shortcut to focus search *(done)*
- `/` or `Ctrl+F` to focus the search input
- `Escape` to clear search and return focus to the list
- Documented in help modal

### ✅ 3.3 JSON pretty-printing in detail pane *(done)*
- Parses and re-renders `kind:json` entries with `serde_json::to_string_pretty`
- Falls back to raw content if parsing fails
- Rendered with an amber left-border accent to distinguish from plain text
- Detail label shows "JSON" instead of "Text"

### ✅ 3.4 Theme persistence on desktop *(done)*
The dark/light theme resets to Dark on every app restart (localStorage only works in WASM builds).
- On native desktop, theme preference is persisted to `config.toml` in the app config directory
- CLI `--theme <dark|light>` overrides for a single session without writing back to the config
- CLI `--db <path>` replaces the old positional argument for specifying the history database
- Use shared storage crate path resolution once available (1.1)

### ✅ 3.5 Keyboard shortcut for watcher toggle
`p` toggles pause/resume from anywhere in the list. Shows a status flash
("Watcher paused" / "Watcher resumed") consistent with other action feedback.
Documented in the help modal Actions section; a `p` badge also appears inline
next to the Pause/Resume button in the Controls panel.

### ✅ 3.6 Per-entry copy feedback *(done)*
The "Copied!" status flash was tied to a single global signal — navigating quickly after
copying could show the indicator on the wrong entry.
- `copy_status` now stores `Option<(i64, String)>` — the entry ID paired with the message
- DetailPane only renders the flash when the current entry's ID matches
- Manual `copy_status.set(None)` clears removed from all navigation handlers
- Flash message is type-aware: "Copied JSON", "Copied Link", "Copied Path", etc.,
  using the existing `entry_label` system for both button clicks and keyboard shortcuts

### 3.7 Extended `since:` filter syntax
`since:` currently accepts `Nh` (hours) and `Nd` (days), but common durations are missing.
- Add `since:Nm` for minutes (e.g. `since:30m`)
- Add `since:Nw` for weeks (e.g. `since:2w`)
- Document new variants in the help modal filter reference

### 3.8 Duplicate entry collapsing
Consecutive identical clipboard entries (common with apps that write to the clipboard
repeatedly on each keystroke) clutter the list.
- Collapse runs of identical content into a single row with a repeat count badge
- Expanding the group shows individual timestamps

### 3.9 Search query history
- `↑/↓` in the search box cycles through recent queries
- Last N queries persisted to `config.toml`; cleared on `Escape` or explicit clear shortcut
- Documented in the help modal

### 3.10 Filtered entry count badge
When a filter is active there is no indication of how many entries are hidden.
- Show "X of Y" below the search box when results are filtered
- Disappears when the search is empty

### ✅ 3.11 `kind:secret` / `kind:pass` filter support *(done)*
Password entries already use `"password"` as their kind string internally.
`kind:pass`, `kind:password` all work via substring matching — no additional
implementation needed. Document in the help modal filter reference.

### ✅ 3.13 Help modal filter reference update *(done)*
The help modal documents `kind:url` and `kind:json` but is missing `kind:path` (already
implemented) and `since:` date filter variants. Bring the filter reference up to date.
- Add `kind:path` example
- Add `since:today`, `since:yesterday` examples alongside `since:1h`
- Add `type:text` example (currently only `type:image` and `type:password` are listed)

### 3.12 Clickable image entries
Image entries currently have no keyboard action other than viewing the thumbnail.
- `Enter` / `y` on a selected image entry opens the PNG in the system image viewer
  (`xdg-open` on Linux, `open` on macOS)
- Show feedback consistent with the copy flash

---

## Tier 4 — New Capabilities

### 4.0 TUI viewer (clipboard-tui)
Complement the desktop GUI with a terminal-based viewer for SSH sessions, headless servers,
and terminal-first workflows. Lives as a separate crate in the workspace.
- Uses ratatui for TUI rendering, crossterm for terminal interaction
- Depends on `clipboard-common` for history reading, entry types, formatting
- Feature parity with core viewing: search/filter, entry inspection, copying
- Keybindings and UX tailored for keyboard-only terminal environment
- Reads from the same clipboard-watcher database as the desktop viewer

### ✅ 4.1 Date/time range filtering *(done)*
- `since:1h`, `since:Nh` (N hours), `since:Nd` (N days), `since:today`, `since:yesterday`
- Combinable with other filters: `kind:url since:today github`
- Documented in help modal

### 4.2 Favorites / pinning
Let users pin entries so they persist through retention culling and appear at the top of the list.
- Store pinned IDs in a local JSON file (viewer-side, no watcher changes needed)
- `f` to toggle pin on selected entry; pinned entries show a pin icon
- Pinned entries sort to the top or live in a separate section

### ✅ 4.3 Multi-select bulk delete *(done)*
`Space` toggles selection; `Shift+↑/↓` extends it; `Delete`/`d` bulk-deletes
all selected entries with a count-aware confirmation; `Escape` clears selection.
Selected entries show a blue left-accent stripe; a toolbar in the sidebar shows
the count with Delete and ✕ clear buttons.

### ✅ 4.13 Per-type color accents *(done)*
Each content type gets a distinct colored left-border accent in both the sidebar
entry cards and the detail pane, making type immediately scannable at a glance.
- JSON → amber
- Path → teal
- URL/Link → blue
- Password → orange
- Image → purple
- CSS variables defined for both dark and light themes

### 4.4 `kind:code` text classification (watcher)
The watcher classifies text as `plain`, `url`, `json`, or `multiline`. Add a `code` kind.
- Heuristics: indentation patterns, bracket/brace density, language keywords (`fn`, `def`,
  `class`, `import`, `=>`, `;`, etc.)
- Viewer: `kind:code` filter support and a "Code" label/icon

### 4.5 App source tracking (watcher)
Capture the name/title of the foreground app at the time of each clipboard capture.
Enables `app:` filter syntax in the viewer.
- Linux: `xdotool getactivewindow getwindowname`
- macOS: `NSWorkspace.shared.frontmostApplication`
- Windows: `GetForegroundWindow` + `GetWindowText`
- Store in a new nullable `source_app` column (additive schema change)
- Viewer: display in detail pane, add `app:` filter

### 4.6 Export entry to file
No way to save a captured item to disk without copying to clipboard and pasting elsewhere.
- `e` on a selected text entry opens a save dialog (via `rfd`, already a dependency) to write
  the content as a `.txt` file
- `e` on an image entry saves the PNG to a chosen path
- Show success/error feedback consistent with copy flash

### 4.7 Tunable preferences in config.toml
Several values are hardcoded that users may reasonably want to change. Now that `config.toml`
infrastructure exists, expose them without requiring a recompile.
- Password detection character length (currently hardcoded to exactly 25)
- Auth cache TTL in minutes (currently hardcoded to 5)
- History poll interval in milliseconds (currently hardcoded to 500)
- Watcher status poll interval in milliseconds (currently hardcoded to 1000)
- All values have sensible defaults so the config section is optional

### 4.8 Watcher retention policy in config.toml
Max entry count and max age before pruning are hardcoded in the watcher. Expose them as
viewer-readable and watcher-readable config so users can tune without recompiling.
- `retention.max_entries` — maximum number of rows to keep
- `retention.max_age_days` — prune rows older than this
- Complements 4.7; ideally lives in the shared storage crate (1.1)

### 4.9 Hex / colour detection
- Classify entries matching `#rrggbb`, `#rgb`, and `rgb(r, g, b)` patterns as `kind:color`
- Detail pane shows a colour swatch alongside the hex/rgb value
- `kind:color` filter support
- CSS variables for the accent color are already defined (`--image-border` purple is a
  placeholder; assign a dedicated variable once the feature lands)

### 4.14 Sidebar entry density option
The sidebar currently uses a fixed card layout. Users with large histories may prefer a
compact list mode.
- Toggle between "comfortable" (current) and "compact" (single-line, smaller font) layouts
- Persist preference in `config.toml`
- Keyboard shortcut (e.g. `v`) to cycle layouts

### 4.15 Copy as… format variants
Some entries would be more useful if copied in a transformed format without leaving the app.
- JSON entries: "Copy minified" and "Copy pretty" alongside the default Copy
- Path entries: "Copy as file:// URL"
- URL entries: "Copy as Markdown link" (`[title](url)` using the fetched OG title if available)
- Color entries (4.9): "Copy as hex", "Copy as rgb()"

### ✅ 4.10 File path detection *(done)*
- Classify entries that look like absolute paths (`/home/…`, `~/…`, `C:\…`) as `kind:path`
- Detail pane renders the path as a clickable link (`xdg-open` / `open`)
- `kind:path` filter support

### 4.11 Entry tagging
Freeform user-defined tags stored in a viewer-side JSON file (no watcher changes needed).
- `t` to open an inline tag editor on the selected entry
- `#tagname` in search filters to entries with that tag
- Tags shown as small chips in the sidebar row and detail pane

### 4.12 Quick-transform actions
Apply common text transformations to a selected entry without leaving the app.
- Accessible via a submenu or command palette (`x` key)
- Initial transforms: trim whitespace, UPPER/lower case, base64 encode/decode,
  URL encode/decode, JSON minify/prettify
- Result is copied to the clipboard and optionally saved as a new entry

### 4.16 User-defined custom types
Allow users to define their own content types in `config.toml` using regex patterns.
Each custom type gets its own sidebar accent color, detail label, icon, `kind:` filter,
and optional UI behavior — layered on top of the built-in type system (path, json, url,
password, color). Custom type detection runs before built-in heuristics; first match wins.

**Motivating use cases:**
- Crypto user: detect wallet addresses (ETH `0x…`, BTC), seed phrases — with `behavior = "security"` so they're masked and auth-gated like passwords
- Office user: detect invoice IDs (`INV-0001`), ticket numbers (`JIRA-123`), UUIDs, commit SHAs
- Any domain with stable, recognizable clipboard patterns

**Config shape (`config.toml`):**
```toml
[[types]]
name     = "Seed Phrase"
label    = "Seed"
icon     = "lock"
color    = "#f59e0b"
behavior = "security"        # masked by default, requires auth to reveal
patterns = [
  "^(\\w+ ){11}\\w+$",       # 12-word BIP39
  "^(\\w+ ){23}\\w+$"        # 24-word BIP39
]

[[types]]
name     = "Wallet Address"
label    = "Wallet"
icon     = "link"
color    = "#10b981"
patterns = [
  "^(0x)?[0-9a-fA-F]{40}$",
  "^[13][a-km-zA-HJ-NP-Z1-9]{25,34}$"
]

[[types]]
name    = "Invoice ID"
label   = "Invoice"
icon    = "file-text"
color   = "#6366f1"
patterns = ["^INV-[0-9]{4,}$"]
```

**`behavior` values:**
- `"security"` — masked in sidebar and detail pane by default; Show/Hide button requires
  local auth (reuses the existing `AuthCache` flow); sidebar card uses password accent treatment
- `"link"` — content rendered as a clickable link (useful for custom URI schemes like `myapp://…`)
- *(omitted)* — plain text display, standard accent color from `color` field

**Implementation notes:**
- `kind:<label>` filter works automatically for each custom type (e.g. `kind:seed`, `kind:wallet`)
- Accent color read from `color` field (CSS hex); no palette auto-assignment needed
- Purely viewer-side — no watcher changes required
- Config parsed at startup; invalid regex patterns emit a warning and are skipped
- `behavior = "security"` entries never leak content in search result previews

**Open questions:**
- Should richer matchers be supported beyond regex (e.g. `type = "wordlist"` for seed phrases,
  `type = "length_range"` for fixed-length tokens)?
- Does 4.9 colour detection become the first built-in default entry expressed in this same
  system, or remain a hardcoded special case?

### 4.17 Collections and grouping
Curate entries into named collections without modifying the original history.
- Create named collections (e.g. "Research links", "Code snippets", "SSH keys")
- Multi-select entries and add to a collection via a UI menu or keyboard shortcut
- Collections persisted to a viewer-side JSON file (no watcher changes needed)
- View a collection as a filtered list; export as markdown, JSON, or plain text
- Quick-add: entries can be tagged with a collection on creation
- Collections shown in a sidebar panel with entry counts; clicking a collection filters to it

### 4.18 Export to external apps
Send curated or filtered entries to external tools and applications.
- Export selected entries or a collection to markdown file (with optional formatting/annotations)
- Pipe entries to a shell command for custom processing (Unix philosophy)
- Open exported data in system default text editor
- Generate formatted reports (e.g. "all URLs I visited in the past week" as a readable document)
- Copy N consecutive entries concatenated (useful for batch work)

### 4.19 Clipboard analytics
Understand patterns and trends in your clipboard history.
- Frequency heatmap: time of day distribution (when do I copy most?)
- Content type breakdown: pie chart or bar graph (% passwords, URLs, text, images, JSON, etc.)
- Word frequency cloud: most-common terms across text entries
- Top domains: most-copied URL sources
- Timeline view: entries grouped by day, week, month
- Stats dashboard: total entries captured, average per day, retention culling stats

### 4.20 Smart grouping and detection
Automatically identify and group related entries.
- Detect consecutive duplicates or near-duplicates; suggest collapsing
- Cluster similar entries by domain, topic, or pattern
- Suggest collections based on detected relationships (e.g. "these 5 URLs are all GitHub")
- Mark entries as related to form chains (original → modified → final)

### 4.21 Structured data parsing and conversion
Handle semi-structured clipboard data (JSON, CSV, etc.) with richer extraction.
- Parse JSON blobs into an exploreable tree view in the detail pane
- Detect CSV/TSV content; display as a formatted table with sortable columns
- Extract fields from semi-structured text (email addresses, phone numbers, URLs, IP addresses)
- Convert between formats: JSON ↔ CSV, URL-encoded ↔ plain text, base64 encode/decode
- Quick-transform actions (4.12) integrate with this for format conversion

### 4.22 Snippet templates and template management
Store and reuse template entries without cluttering history.
- Mark entries as templates (separate storage, not in main history)
- Use templates to generate variations (e.g. bash script with `{date}` or `{user}` placeholders)
- Quick-insert: rapidly retrieve and copy a template without scrolling history
- Template library: organized view of saved templates with descriptions
- Combine with collections (4.17) for domain-specific template packs (e.g. "SQL queries", "API calls")

### 4.23 Advanced search and query language
Rich search capabilities beyond substring matching.
- Regex search support (alongside literal substring)
- Fuzzy search for typo tolerance
- Full-text search with result highlighting
- Search by metadata: copied on specific date, from specific app (once 4.5 is done)
- Saved searches: store complex queries and reuse them
- Search within search: refine results progressively

### 4.24 Search/replace and transformations within entries
Edit and derive new entries from existing clipboard data.
- Search/replace within an entry, save result as new entry (don't modify original)
- Quick transforms: trim whitespace, case conversions, URL decode, base64 decode inline
- Combine with collections (4.17) to bulk-apply transforms to multiple entries

### 4.25 Clipboard diffing
Compare two entries side-by-side to see changes.
- Select two entries to diff
- Highlight added/removed lines with line-by-line diffs (useful for configs, code, JSON)
- Show character-level diffs for small changes
- Export diff as unified/context diff format

### 4.26 FZF integration for terminal
Pipe clipboard history to fzf for fast selection in shell workflows.
- CLI command: `clipboard list | fzf` to search and select
- Copy selected entry to clipboard on choice
- Integration with shell aliases for one-liners
- No GUI dependency; works over SSH

### 4.27 Global hotkey and search popup
System-level quick access to clipboard history without switching windows.
- Platform-specific global hotkey (e.g. Ctrl+Shift+V or Cmd+Shift+V)
- Small popup window with search field (no full GUI overhead)
- Select and copy, or open in full app; closes on Escape
- Desktop-only; works from any application context

### 4.28 Interactive shell REPL for clipboard queries
Query-oriented interface for power users.
- `clipboard> select * where kind=url` style queries
- Natural language: `show me all passwords from today`
- One-off analysis without opening the GUI
- Pipe results to external tools

### 4.29 App-aware capture and filtering
Control what gets captured based on source application.
- Whitelist/blacklist apps (only capture from Firefox, VS Code)
- App-specific retention policies (browser history forever, terminal history for 7 days)
- App-specific transformations (strip shell escapes before storing)
- View entries grouped by source app in the sidebar

### 4.30 Passphrase-protected vaults
Additional encryption layer for highly sensitive entries.
- Create named vaults with separate passphrases beyond keyring
- Move entries into/out of vaults
- Vault entries require passphrase entry (not just local auth cache)
- Per-vault automatic lock timeout
- Combine with entry tagging (4.11) for easy categorization

### 4.31 Per-entry encryption and security settings
Fine-grained control over individual entry sensitivity.
- Mark specific entries for "extra scrutiny": require auth every time, not just once
- Time-limited access: "this API key expires in 1 hour"
- Per-entry auto-delete: "delete this password after 24 hours"
- Access logs: log when/what accessed each sensitive entry (optional, privacy-respecting)

### 4.32 Clipboard redaction and privacy mode
Protect sensitive data in screenshots and screen-sharing.
- Blur/pixelate password entries when taking screenshots
- Privacy mode: dim or hide selected entries while screen-sharing
- Redaction preview: see what others see
- Optional: automatic blur during screen recording

### 4.33 Secure deletion and overwrite
Ensure deleted entries can't be recovered.
- Multi-pass overwrite for deleted entries (DoD standard or similar)
- Secure delete option on per-entry basis
- Scheduled purge: periodically sanitize disk space
- Stats: show estimated recovery vulnerability

### 4.34 Custom clipboard actions & rules engine

Powerful, user-defined automation for clipboard content. Two-part feature:

#### Part A: Single-Entry Custom Actions
React to captured content with pattern-matched handlers. Extends 4.16 (custom types).

**Config Model:**
```toml
# Define reusable handlers
[handlers]
etherscan = { 
  type = "link", 
  template = "https://etherscan.io/address/{match}", 
  app = "firefox" 
}
email_client = { 
  type = "open", 
  protocol = "mailto", 
  app = "thunderbird" 
}
internal_sku_lookup = { 
  type = "link", 
  template = "https://internal.company.com/sku/{match}", 
  app = "chrome" 
}
append_to_notes = {
  type = "save_to_file",
  target = "~/notes/work.txt",
  append = true
}

# Associate handlers with custom types
[[custom_types]]
name = "crypto_address"
pattern = "^0x[a-fA-F0-9]{40}$"
behaviors = ["crypto"]
actions = [ { handler = "etherscan" } ]

[[custom_types]]
name = "mailto"
pattern = "^mailto:.*"
behaviors = ["email"]
actions = [ { handler = "email_client" } ]

[[custom_types]]
name = "product_sku"
pattern = "^SKU-[0-9]{6}$"
behaviors = ["sku"]
actions = [ { handler = "internal_sku_lookup" } ]
```

**Built-in action types:**
- `link` — open URL in specified app (template substitution from matched pattern)
- `open` — open via protocol handler (mailto, tel, etc.) in specified app
- `copy` — copy to clipboard with optional transform/format
- `save_to_file` — append or replace to file
- `command` — execute local shell command (permission required)
- `notify` — show UI toast/notification

**UI Behavior:**
- Single-entry actions appear as buttons in detail pane (next to copy/delete)
- One-click execution or confirmation dialog for sensitive actions (command, delete file)
- Fallback: if `app` not found, use system default or skip with warning

**Benefits:**
- No webhooks/external services required for local automation
- Users can define custom behaviors without modifying app code
- Reusable handlers reduce config duplication
- Composable: one handler used by multiple types

#### Part B: Bulk Operations Framework
Perform batch actions on multi-selected entries.

**Config Model:**
```toml
# Define bulk aggregation handlers
[handlers]
# Collect multiple entries into single file with separator
notepad_bulk = {
  type = "aggregate_to_file",
  target = "~/notes/bulk_entries.txt",
  separator = "\n---\n",
  include_metadata = false,  # true adds timestamps, types
  app = "notepad"  # optional: open after write
}

# Send batch to external service
slack_bulk = {
  type = "send_bulk",
  endpoint = "https://hooks.slack.com/services/...",
  format = "list",  # or "blocks", "json"
  delimiter = "\n"
}
```

**UI Behavior:**
- Multi-select (already exists via 4.3)
- "Bulk Actions" menu appears when 2+ entries selected
- Available bulk actions depend on entry type filter
- Confirmation dialog shows: count, action, target

**Implementation:**
1. Extend multi-select state tracking
2. Add aggregation engine: collect entries → apply separator/formatting → write/send
3. Permission model: ask user first for file write/command execute
4. Progress indicator for large batches

**Scope considerations:**
- **Shared logic**: Both A & B use the same handler definition and templating engine
- **Permissions**: Require explicit allow/deny for sensitive actions (write files, execute commands)
- **Feedback**: Show what was written/sent in a toast or status pane
- **Undo**: Optionally support undo for file operations (backup before write?)

**Design trade-off:**
- **Option 1 (Combined)**: Implement both A & B together as one cohesive feature (higher upfront cost, cleaner end result)
- **Option 2 (Staged)**: 4.34 = single actions, later roadmap item = bulk operations (lower initial scope, simpler first PR)

Recommend **Option 1** for consistency, but can split if reducing scope is priority.

### 4.34.1 Content transformations & field extraction

Parse and extract data from clipboard entries using patterns and transformations. Turns clipboard into a data pipeline.

**Use Cases:**
- JSON response → extract `id` field only
- URL with params → capture resource ID
- CSV row → parse specific columns
- Error log → extract stack trace
- API response → extract auth token
- Multiline text → extract first/last N lines

**Config Model:**
```toml
[handlers]
# Regex capture group
extract_id_from_url = {
  type = "transform",
  operation = "regex:id=([^&]+)",
  output = "group_1",
  then_action = { type = "copy" }
}

# jq for JSON parsing
extract_api_token = {
  type = "transform",
  operation = "jq .token",
  then_action = { type = "copy" }
}

# Simple substring
extract_first_line = {
  type = "transform",
  operation = "split_lines:first",
  then_action = { type = "copy" }
}

# Chainable: transform → format → send
transform_and_send = {
  type = "pipeline",
  steps = [
    { type = "transform", operation = "jq .data[]" },
    { type = "transform", operation = "join_newlines" },
    { type = "send_bulk", endpoint = "https://hooks.slack.com/..." }
  ]
}

[[custom_types]]
name = "json_response"
pattern = "^\\{.*\\}$"
actions = [
  { handler = "extract_api_token" }
]

[[custom_types]]
name = "parameterized_url"
pattern = "^https?://.+\\?.*id=.*"
actions = [
  { handler = "extract_id_from_url" }
]
```

**Built-in operations:**
- `regex:<pattern>` — capture groups, returns match or group_1/group_2/etc.
- `jq <filter>` — JSON query (requires jq binary or Rust jq library)
- `split_lines:<first|last|nth:N>` — extract line(s) from multiline text
- `split_by:<delimiter>` — split and return nth element
- `join_lines` / `join_by:<sep>` — combine multiple lines
- `trim` / `uppercase` / `lowercase` — simple transforms
- `url_decode` / `url_encode` — URL utilities
- `base64_encode` / `base64_decode` — encoding

**UI Behavior:**
- Transform button in detail pane: "Extract → Copy"
- Show preview of transformed result before copying (for complex transforms)
- Error state: if transform fails, show error and offer fallback (copy original)

**Implementation Notes:**
- Regex: use `regex` crate, stable
- jq: either shell out to `jq` binary (optional dependency) or use `jq-rs` crate
- Simple transforms: builtin Rust string operations
- Chaining: run transforms sequentially, pass output to next step
- Security: sandbox/validate regex and jq for DoS (regex backtracking, jq complexity)

**Unlocks:**
- Custom types + transforms: match pattern → extract field → copy (power users love this)
- Bulk operations: transform all selected → then aggregate (extract IDs, group by type, etc.)
- Workflows: copy URL → extract ID → auto-open in admin panel (one click)
- Community recipes: users share ".extract-github-issue-number.toml" config

### 4.34.2 HTML image extraction ✅ *COMPLETED*

Detect and render `<img src="...">` HTML tags as image entries in the UI.

**Use case:** User copies an HTML snippet containing an image tag; viewer recognizes the pattern and displays the image instead of raw HTML text.

**Implementation approach:**
- Viewer-side detection at render time (no watcher changes)
- Detect `<img src="...">` pattern in text entries
- Extract src URL using regex-free parsing
- Render as image in detail pane
- Support both external URLs (`https://...`) and data URLs (`data:image/...`)
- Fallback: display HTML text if image fails to load

**Technical details:**
- Added `is_html_img_tag()` to validate `<img .../>`  format
- Added `extract_html_img_src()` to parse src URL (handles quoted and unquoted URLs)
- Updated detail_pane to detect and render html_image entries with image display
- Watcher remains unchanged; classification happens at UI layer

**Status:** ✅ COMPLETE
- ✅ HTML img tag detection and src extraction
- ✅ Viewer detail pane renders images
- ✅ Support for https:// and data:image/ URLs
- ✅ All tests passing, no clippy warnings

### 4.35 Predictive suggestions and recommendations
ML-powered insights into clipboard patterns.
- Clustering: "based on history, you usually copy these 3 things together"
- Common sequences: "you often copy URL, then JSON, then text — quick-load pattern"
- Recommendation engine: "you searched for 'API' — here are your 5 most common API URLs"
- Learn from user behavior (opt-in telemetry on local machine only)

### 4.36 Clipboard timeline and visualization
Visual exploration of clipboard history over time.
- Timeline view: scrollable graph of entries by timestamp
- Activity heatmap: when are you most productive (most clipboard activity)?
- Trend analysis: which content types are increasing/decreasing over time?
- Filters apply to timeline (see only URLs, only passwords, etc.)

### 4.37 Cross-device sync
Access clipboard history from multiple machines.
- Encrypted sync to user-supplied S3/WebDAV endpoint (encryption keys never leave device)
- One-time share links: temporarily share an entry or collection to another device
- Mobile web interface to browse history on the go
- Selective sync: choose which entries to sync vs. keep local-only
- Conflict resolution for simultaneous edits

### 4.38 Plugin ecosystem and integrations
Official integrations with popular tools and platforms.
- **Obsidian plugin**: sync collections directly to vault, bidirectional linking
- **VS Code extension**: browse clipboard history in sidebar, insert snippets
- **Raycast/Alfred**: search history from spotlight, copy/paste workflows
- **Slack command**: `/clipboard search query` to search from Slack
- **System integrations**: Alfred, LaunchBar, etc.
- Plugin API: allow third-party developers to build extensions

### 4.39 Clipboard API and CLI tooling
Programmatic access to clipboard history for automation and integration.
- JSON API: query history with filters, retrieve raw entry data
- CLI: `clipboard search`, `clipboard add`, `clipboard delete`, `clipboard export`
- Batch operations: process multiple entries in one command
- Event hooks: scripts that run when certain content is captured (triggers for 4.34)
- Output formats: JSON, CSV, markdown, plain text

### 4.40 Debugging, diagnostics, and metrics
Tools for performance monitoring and troubleshooting.
- Clipboard history inspector: browse raw DB entries, view/edit metadata
- Performance profiling: identify slow operations (search, filtering, rendering)
- Watcher debug mode: verbose logs of capture events and state changes
- Metrics export: Prometheus-style metrics for uptime monitoring
- Health check: diagnose issues with database, keyring, file permissions

---

## Tier 5 — Deferred & Advanced

*Worth doing eventually, not blocking anything now. Some are platform-specific or require
external infrastructure.*

- **Global hotkey / quick-access popup** (4.27) — summon search overlay from anywhere
- **FZF integration** (4.26) — clipboard history in terminal workflows
- **Advanced search** (4.23) — regex, fuzzy, full-text search
- **Clipboard diffing** (4.25) — compare two entries side-by-side
- **Interactive REPL** (4.28) — `clipboard> select * where kind=url` queries
- **App-aware filtering** (4.29) — capture control and retention policies per-app
- **Passphrase-protected vaults** (4.30) — extra encryption layer for sensitive groups
- **Per-entry security settings** (4.31) — time-limited access, per-entry auth, expiry
- **Clipboard redaction & privacy mode** (4.32) — blur sensitive data in screenshots
- **Secure deletion** (4.33) — multi-pass overwrite for deleted entries
- **Automation triggers & webhooks** (4.34) — react to clipboard patterns
- **Predictive suggestions** (4.35) — ML clustering, common sequences, recommendations
- **Clipboard timeline visualization** (4.36) — timeline view, heatmaps, trend analysis
- **Cross-device sync** (4.37) — S3/WebDAV encrypted sync, mobile access
- **Plugin ecosystem** (4.38) — Obsidian, VS Code, Raycast, Slack, etc.
- **Clipboard API and CLI** (4.39) — programmatic query, batch operations, event hooks
- **Debugging & diagnostics** (4.40) — profiling, inspector, metrics export

---

## Build Order

1. ~~Workspace consolidation (1.0)~~ ✅
2. ~~Shared storage crate (1.1)~~ ✅
3. Image lifecycle cleanup (1.2)
4. ~~Password sidebar fix (2.1)~~ *(not an issue)*
5. ~~Relative timestamps (3.1)~~ ✅
6. ~~Search keyboard shortcut (3.2)~~ ✅
7. ~~JSON pretty-print (3.3)~~ ✅
8. Contract tests (1.3)
9. Watcher socket recovery (2.2)
10. Link preview retry (2.3)
11. ~~Theme persistence (3.4)~~ ✅
12. ~~Date range filtering (4.1)~~ ✅
13. URL detection edge cases (2.4)
14. ~~Watcher graceful shutdown (2.5)~~ ✅
15. Watcher logging framework (2.6)
16. Watcher configuration validation (2.7)
17. Watcher environment variable overrides (2.8)
18. Cross-platform watcher control (2.9)
19. ~~Watcher code modularization (2.10)~~ ✅
20. ~~Watcher type-safe error handling (2.11)~~ ✅
21. ~~Watcher keyboard shortcut (3.5)~~ ✅
22. ~~Per-type color accents (4.13)~~ ✅
23. ~~Per-entry copy feedback (3.6)~~ ✅
24. Entry count badge (3.10)
25. ~~`kind:pass` filter (3.11)~~ ✅
26. Extended `since:` syntax (3.7)
27. Search query history (3.9)
28. Duplicate entry collapsing (3.8)
29. Clickable image entries (3.12)
30. Tunable preferences in config.toml (4.7)
31. Watcher retention policy in config.toml (4.8)
32. Favorites / pinning (4.2)
33. Export entry to file (4.6)
34. ~~Multi-select bulk delete (4.3)~~ ✅
35. TUI viewer (4.0)
36. `kind:code` classification (4.4)
37. Hex / colour detection (4.9)
38. ~~File path detection (4.10)~~ ✅
39. App source tracking (4.5)
40. Sidebar entry density option (4.14)
41. Copy as… format variants (4.15)
42. Entry tagging (4.11)
43. Quick-transform actions (4.12)
44. User-defined custom types (4.16)
45. Collections and grouping (4.17)
46. Export to external apps (4.18)
47. Clipboard analytics (4.19)
48. Smart grouping and detection (4.20)
49. Structured data parsing and conversion (4.21)
50. Snippet templates and template management (4.22)
51. Advanced search and query language (4.23)
52. Search/replace and transformations (4.24)
53. Clipboard diffing (4.25)
54. FZF integration for terminal (4.26)
55. Global hotkey and search popup (4.27)
56. Interactive shell REPL (4.28)
57. App-aware capture and filtering (4.29)
58. Passphrase-protected vaults (4.30)
59. Per-entry encryption and security settings (4.31)
60. Clipboard redaction and privacy mode (4.32)
61. Secure deletion and overwrite (4.33)
62. Clipboard automation triggers and webhooks (4.34)
63. Content transformations & field extraction (4.34.1)
64. HTML image extraction (4.34.2) ✅
65. Predictive suggestions and recommendations (4.35)
66. Clipboard timeline and visualization (4.36)
66. Cross-device sync (4.37)
67. Plugin ecosystem and integrations (4.38)
68. Clipboard API and CLI tooling (4.39)
69. Debugging, diagnostics, and metrics (4.40)
