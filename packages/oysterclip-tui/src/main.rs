use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use rusqlite::Connection;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use common::classification::is_password;
use common::constants::{APP_NAME, APP_ORGANIZATION, APP_QUALIFIER, HISTORY_FILE};
use common::crypto::{decrypt_text, get_or_create_key};

struct App {
    entries: Vec<(i64, String)>,
    selected_index: usize,
    scroll_offset: usize,
    list_viewport_height: usize,
    running: bool,
}

impl App {
    fn new() -> io::Result<Self> {
        let entries = load_entries()
            .map_err(|e| io::Error::other(format!("Failed to load entries: {}", e)))?;

        Ok(Self {
            selected_index: 0,
            scroll_offset: 0,
            list_viewport_height: 15,
            running: true,
            entries,
        })
    }

    fn run(&mut self) -> io::Result<()> {
        let mut terminal = setup_terminal()?;

        while self.running {
            terminal.draw(|f| self.draw(f))?;
            self.handle_events()?;
        }

        restore_terminal(&mut terminal)?;
        Ok(())
    }

    fn draw(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(3), Constraint::Min(10)])
            .split(f.area());

        // Title
        let title = Paragraph::new("OysterClip TUI - Clipboard History")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan).bold());
        f.render_widget(title, chunks[0]);

        // Split list and detail
        let list_detail = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(chunks[1]);

        // List view
        let list_block = Block::default()
            .borders(Borders::ALL)
            .title("History (↑↓ Navigate, PgUp/PgDn Scroll, q Quit)");
        f.render_widget(list_block, list_detail[0]);

        let list_area = Rect {
            x: list_detail[0].x + 1,
            y: list_detail[0].y + 1,
            width: list_detail[0].width.saturating_sub(2),
            height: list_detail[0].height.saturating_sub(2),
        };
        self.render_list(f, list_area);

        // Detail view
        self.render_detail(f, list_detail[1]);
    }

    fn render_list(&mut self, f: &mut Frame, area: Rect) {
        self.list_viewport_height = area.height as usize;

        let items: Vec<ListItem> = self
            .entries
            .iter()
            .skip(self.scroll_offset)
            .enumerate()
            .map(|(idx, (_, preview))| {
                let actual_idx = idx + self.scroll_offset;
                let display_text = if is_password(preview) {
                    "•".repeat(8)
                } else {
                    if preview.len() > 45 {
                        format!("{}...", &preview[..45])
                    } else {
                        preview.clone()
                    }
                };

                let style = if actual_idx == self.selected_index {
                    Style::default().fg(Color::Black).bg(Color::Cyan)
                } else {
                    Style::default()
                };

                ListItem::new(format!("{:3} {}", actual_idx + 1, display_text)).style(style)
            })
            .collect();

        let list = List::new(items);
        f.render_widget(list, area);
    }

    fn render_detail(&self, f: &mut Frame, area: Rect) {
        let content = if let Some((_, preview)) = self.entries.get(self.selected_index) {
            vec![
                Line::from(""),
                Line::from("Content:"),
                Line::from("─".repeat(40)),
                Line::from(preview.clone()),
            ]
        } else {
            vec![Line::from("No entry selected")]
        };

        let paragraph = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title("Detail"))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    self.handle_key(key);
                }
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.running = false,
            KeyCode::Up => {
                if self.entries.is_empty() {
                    return;
                }
                self.selected_index = if self.selected_index == 0 {
                    self.entries.len() - 1
                } else {
                    self.selected_index - 1
                };
                self.ensure_selection_visible();
            }
            KeyCode::Down => {
                if self.entries.is_empty() {
                    return;
                }
                self.selected_index = (self.selected_index + 1) % self.entries.len();
                self.ensure_selection_visible();
            }
            KeyCode::PageUp => {
                if self.scroll_offset >= 5 {
                    self.scroll_offset -= 5;
                } else {
                    self.scroll_offset = 0;
                }
            }
            KeyCode::PageDown => {
                self.scroll_offset = self.scroll_offset.saturating_add(5);
            }
            _ => {}
        }
    }

    fn ensure_selection_visible(&mut self) {
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + self.list_viewport_height {
            self.scroll_offset = self.selected_index - self.list_viewport_height + 1;
        }
    }
}

fn load_entries() -> Result<Vec<(i64, String)>, String> {
    let db_path = get_db_path()?;
    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;

    let query = "SELECT id, text_kind, text_ciphertext, text_nonce FROM entries WHERE entry_type = 'text' ORDER BY id DESC LIMIT 100";

    let mut stmt = conn
        .prepare(query)
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let key = load_encryption_key()?;
    let mut entries = Vec::new();

    let rows = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let text_kind: Option<String> = row.get(1)?;
            let ciphertext: Vec<u8> = row.get(2)?;
            let nonce: Vec<u8> = row.get(3)?;
            Ok((id, text_kind, ciphertext, nonce))
        })
        .map_err(|e| format!("Failed to query entries: {}", e))?;

    for row_result in rows {
        let (id, _kind, ciphertext, nonce) =
            row_result.map_err(|e| format!("Failed to read row: {}", e))?;

        match decrypt_wrapper(&ciphertext, &nonce, &key) {
            Ok(content) => {
                let preview = content.replace('\n', " ↵ ");
                entries.push((id, preview));
            }
            Err(_e) => {
                // Skip entries that fail to decrypt
                continue;
            }
        }
    }

    Ok(entries)
}

fn load_encryption_key() -> Result<[u8; 32], String> {
    get_or_create_key().map_err(|e| e.to_string())
}

fn decrypt_wrapper(ciphertext: &[u8], nonce: &[u8], key: &[u8; 32]) -> Result<String, String> {
    decrypt_text(ciphertext, nonce, key).map_err(|e| e.to_string())
}

fn get_db_path() -> Result<PathBuf, String> {
    let dirs = directories::ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, APP_NAME)
        .ok_or_else(|| "Could not determine project directories".to_string())?;

    Ok(dirs.data_local_dir().join(HISTORY_FILE))
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn main() -> io::Result<()> {
    let mut app = App::new()?;
    app.run()
}
