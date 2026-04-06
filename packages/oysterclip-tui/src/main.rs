use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use std::io;
use std::time::Duration;
use rusqlite::Connection;
use std::path::PathBuf;

use base64::{engine::general_purpose, Engine as _};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use keyring::Entry;
use common::constants::{HISTORY_FILE, KEYRING_ACCOUNT, PROJECT_NAME};

struct App {
    entries: Vec<(i64, String)>,
    selected_index: usize,
    running: bool,
}

impl App {
    fn new() -> io::Result<Self> {
        let entries = load_entries().map_err(|e| {
            io::Error::other(format!("Failed to load entries: {}", e))
        })?;
        
        Ok(Self {
            selected_index: 0,
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

    fn draw(&self, f: &mut Frame) {
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
            .title("History (↑↓ Navigate, q Quit)");
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

    fn render_list(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .entries
            .iter()
            .enumerate()
            .map(|(idx, (_, preview))| {
                let text = if preview.len() > 45 {
                    format!("{}...", &preview[..45])
                } else {
                    preview.clone()
                };
                
                let style = if idx == self.selected_index {
                    Style::default().fg(Color::Black).bg(Color::Cyan)
                } else {
                    Style::default()
                };

                ListItem::new(format!("{:3} {}", idx + 1, text)).style(style)
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
            }
            KeyCode::Down => {
                if self.entries.is_empty() {
                    return;
                }
                self.selected_index = (self.selected_index + 1) % self.entries.len();
            }
            _ => {}
        }
    }
}

fn load_entries() -> Result<Vec<(i64, String)>, String> {
    let db_path = get_db_path()?;
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    let has_image_blob = has_column(&conn, "entries", "image_png")?;
    
    let query = if has_image_blob {
        "SELECT id, text_kind, text_ciphertext, text_nonce FROM entries WHERE entry_type = 'text' ORDER BY id DESC LIMIT 100"
    } else {
        "SELECT id, text_kind, text_ciphertext, text_nonce FROM entries WHERE entry_type = 'text' ORDER BY id DESC LIMIT 100"
    };
    
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
        let (id, _kind, ciphertext, nonce) = row_result
            .map_err(|e| format!("Failed to read row: {}", e))?;
        
        match decrypt_text(&ciphertext, &nonce, &key) {
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
    let entry = Entry::new(PROJECT_NAME, KEYRING_ACCOUNT)
        .map_err(|e| format!("Failed to access keyring: {}", e))?;
    
    let key_str = entry
        .get_password()
        .map_err(|e| format!("Failed to retrieve encryption key from keyring: {}", e))?;
    
    let decoded = general_purpose::STANDARD
        .decode(&key_str)
        .map_err(|e| format!("Failed to decode encryption key: {}", e))?;
    
    if decoded.len() != 32 {
        return Err(format!("Invalid encryption key length: expected 32 bytes, got {}", decoded.len()));
    }
    
    let mut key = [0u8; 32];
    key.copy_from_slice(&decoded);
    Ok(key)
}

fn decrypt_text(ciphertext: &[u8], nonce: &[u8], key: &[u8; 32]) -> Result<String, String> {
    if nonce.len() != 24 {
        return Err(format!(
            "Invalid text nonce length: expected 24 bytes, got {}",
            nonce.len()
        ));
    }

    let cipher = XChaCha20Poly1305::new(key.into());
    let plaintext = cipher
        .decrypt(XNonce::from_slice(nonce), ciphertext)
        .map_err(|e| format!("Failed to decrypt clipboard text: {}", e))?;

    String::from_utf8(plaintext)
        .map_err(|e| format!("Failed to decode decrypted clipboard text: {}", e))
}

fn has_column(conn: &Connection, table_name: &str, column_name: &str) -> Result<bool, String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table_name})"))
        .map_err(|e| format!("Failed to inspect schema: {}", e))?;
    
    let mut rows = stmt
        .query([])
        .map_err(|e| format!("Failed to query schema: {}", e))?;

    while let Some(row) = rows
        .next()
        .map_err(|e| format!("Failed to iterate schema: {}", e))?
    {
        let name: String = row
            .get(1)
            .map_err(|e| format!("Failed to read column name: {}", e))?;
        if name == column_name {
            return Ok(true);
        }
    }

    Ok(false)
}

fn get_db_path() -> Result<PathBuf, String> {
    let dirs = directories::ProjectDirs::from("", "", PROJECT_NAME)
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
