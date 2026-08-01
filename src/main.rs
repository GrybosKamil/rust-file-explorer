use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use std::{
    fs,
    io::stdout,
    path::PathBuf,
};

#[derive(Parser)]
#[command(author, version, about = "Interactive TUI File Explorer in Rust")]
struct Cli {
    /// Directory path to start in (defaults to current directory)
    #[arg(default_value = ".")]
    path: PathBuf,
}

struct App {
    current_dir: PathBuf,
    items: Vec<PathBuf>,
    state: ListState,
}

impl App {
    fn new(start_dir: PathBuf) -> Self {
        let canonical_dir = fs::canonicalize(start_dir).unwrap_or_else(|_| PathBuf::from("."));
        let mut app = Self {
            current_dir: canonical_dir,
            items: Vec::new(),
            state: ListState::default(),
        };
        app.load_directory();
        app
    }

    /// Reads directory contents and sorts entries (Directories first, then Files)
    fn load_directory(&mut self) {
        self.items.clear();
        if let Ok(entries) = fs::read_dir(&self.current_dir) {
            let mut dirs = Vec::new();
            let mut files = Vec::new();

            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    dirs.push(path);
                } else {
                    files.push(path);
                }
            }

            dirs.sort();
            files.sort();

            self.items.extend(dirs);
            self.items.extend(files);
        }

        if !self.items.is_empty() {
            self.state.select(Some(0));
        } else {
            self.state.select(None);
        }
    }

    fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Enter selected directory (Right Arrow / Enter)
    fn enter_directory(&mut self) {
        if let Some(selected_index) = self.state.selected() {
            if let Some(target_path) = self.items.get(selected_index) {
                if target_path.is_dir() {
                    self.current_dir = target_path.clone();
                    self.load_directory();
                }
            }
        }
    }

    /// Go up to parent directory (Left Arrow / Backspace)
    fn go_up(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.load_directory();
        }
    }
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    // Enable terminal raw mode & alternate screen
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(args.path);

    // Event Loop
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
                .split(f.area());

            // Build List Items with styling
            let list_items: Vec<ListItem> = app
                .items
                .iter()
                .map(|path| {
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy())
                        .unwrap_or_default();

                    if path.is_dir() {
                        let label = format!("📁  {}/", name);
                        ListItem::new(label).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                    } else {
                        let size = path
                            .metadata()
                            .map(|m| format_size(m.len()))
                            .unwrap_or_else(|_| "-".to_string());
                        let label = format!("📄  {:<35} [{}]", name, size);
                        ListItem::new(label).style(Style::default().fg(Color::Green))
                    }
                })
                .collect();

            let title = format!(" Directory: {} ", app.current_dir.display());
            let list = List::new(list_items)
                .block(Block::default().borders(Borders::ALL).title(title))
                .highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, chunks[0], &mut app.state);

            // Footer instructions bar
            let footer = Paragraph::new(" Navigation: [↑/↓] Select | [→/Enter] Enter Dir | [←/Backspace] Go Up | [q] Quit ")
                .style(Style::default().fg(Color::Black).bg(Color::White));
            f.render_widget(footer, chunks[1]);
        })?;

        // Handle user input
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Down | KeyCode::Char('j') => app.next(),
                KeyCode::Up | KeyCode::Char('k') => app.previous(),
                KeyCode::Right | KeyCode::Enter | KeyCode::Char('l') => app.enter_directory(),
                KeyCode::Left | KeyCode::Backspace | KeyCode::Char('h') => app.go_up(),
                _ => {}
            }
        }
    }

    // Restore terminal state before exiting
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}