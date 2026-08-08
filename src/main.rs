use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
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
use std::{fs, io::stdout, path::PathBuf};

#[derive(Parser)]
#[command(author, version, about = "Interactive TUI File Explorer in Rust")]
struct Cli {
    /// Directory path to start in (defaults to current directory)
    #[arg(default_value = ".")]
    path: PathBuf,
}

enum InputMode {
    Normal,
    Searching,
}

struct App {
    current_dir: PathBuf,
    all_items: Vec<PathBuf>,
    filtered_items: Vec<PathBuf>,
    state: ListState,
    search_query: String,
    mode: InputMode,
}

impl App {
    fn new(start_dir: PathBuf) -> Self {
        let canonical_dir = fs::canonicalize(start_dir).unwrap_or_else(|_| PathBuf::from("."));
        let mut app = Self {
            current_dir: canonical_dir,
            all_items: Vec::new(),
            filtered_items: Vec::new(),
            state: ListState::default(),
            search_query: String::new(),
            mode: InputMode::Normal,
        };
        app.load_directory();
        app
    }

    /// Reads directory contents and sorts entries (Directories first, then Files)
    fn load_directory(&mut self) {
        self.all_items.clear();
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

            self.all_items.extend(dirs);
            self.all_items.extend(files);
        }

        self.apply_filter();
    }

    /// Filters items based on `search_query`
    fn apply_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_items = self.all_items.clone();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_items = self
                .all_items
                .iter()
                .filter(|path| {
                    path.file_name()
                        .map(|n| n.to_string_lossy().to_lowercase().contains(&query))
                        .unwrap_or(false)
                })
                .cloned()
                .collect();
        }

        if !self.filtered_items.is_empty() {
            self.state.select(Some(0));
        } else {
            self.state.select(None);
        }
    }

    fn next(&mut self) {
        if self.filtered_items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.filtered_items.len() - 1 {
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
        if self.filtered_items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_items.len() - 1
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
            if let Some(target_path) = self.filtered_items.get(selected_index) {
                if target_path.is_dir() {
                    self.current_dir = target_path.clone();
                    self.clear_search();
                    self.load_directory();
                }
            }
        }
    }

    /// Go up to parent directory (Left Arrow only)
    fn go_up(&mut self) {
        if let Some(parent) = self.current_dir.parent().map(|p| p.to_path_buf()) {
            let exited_dir = self.current_dir.clone();

            self.current_dir = parent;
            
            // 1. Reset search and reload parent directory
            self.search_query.clear();
            self.mode = InputMode::Normal;
            self.load_directory(); // This runs apply_filter() and sets selection to 0

            // 2. Override the selection to highlight the directory we just left
            if let Some(target_index) = self.filtered_items.iter().position(|path| path == &exited_dir) {
                self.state.select(Some(target_index));
            }
        }
    }

    fn clear_search(&mut self) {
        self.search_query.clear();
        self.mode = InputMode::Normal;
        self.apply_filter();
    }
}

fn format_size(bytes: u64) -> String {
    let kb = 1024.0;
    let mb = kb * 1024.0;
    let gb = mb * 1024.0;
    let tb = gb * 1024.0;

    let bytes_f = bytes as f64;

    if bytes_f < kb {
        format!("[{:>5}  B]", bytes)
    } else if bytes_f < mb {
        format!("[{:>5.1} KB]", bytes_f / kb)
    } else if bytes_f < gb {
        format!("[{:>5.1} MB]", bytes_f / mb)
    } else if bytes_f < tb {
        format!("[{:>5.1} GB]", bytes_f / gb)
    } else {
        format!("[{:>5.1} TB]", bytes_f / tb)
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
            let has_search_bar = match app.mode {
                InputMode::Searching => true,
                InputMode::Normal => !app.search_query.is_empty(),
            };

            let constraints = if has_search_bar {
                vec![
                    Constraint::Min(1),
                    Constraint::Length(3),
                    Constraint::Length(1),
                ]
            } else {
                vec![Constraint::Min(1), Constraint::Length(1)]
            };

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(f.area());

            // Build List Items with styling
            let list_items: Vec<ListItem> = app
                .filtered_items
                .iter()
                .map(|path| {
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy())
                        .unwrap_or_default();

                    if path.is_dir() {
                        let label = format!("           📁  {}/", name);
                        ListItem::new(label).style(
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        let size = path
                            .metadata()
                            .map(|m| format_size(m.len()))
                            .unwrap_or_else(|_| "[    -    ]".to_string());
                        let label = format!("{} 📄  {}", size, name);
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

            // Render Search Input Bar if active
            if has_search_bar {
                let search_title = match app.mode {
                    InputMode::Searching => " Search (Type to filter, Esc to cancel) ",
                    InputMode::Normal => " Filter Active (Esc to clear) ",
                };
                let search_bar = Paragraph::new(app.search_query.as_str()).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(search_title)
                        .border_style(Style::default().fg(Color::Yellow)),
                );
                f.render_widget(search_bar, chunks[1]);
            }

            // Footer instructions bar
            let footer_text = match app.mode {
                InputMode::Searching => {
                    " Typing... | [Backspace] Delete | [Esc] Clear Search | [Ctrl+C] Quit "
                }
                InputMode::Normal => {
                    " [Type to Filter] | [↑/↓] Navigate | [→/Enter] Open | [←] Back | [Ctrl+C] Quit "
                }
            };
            let footer_idx = if has_search_bar { 2 } else { 1 };
            let footer = Paragraph::new(footer_text)
                .style(Style::default().fg(Color::Black).bg(Color::White));
            f.render_widget(footer, chunks[footer_idx]);
        })?;

        // Handle user input
        if let Event::Key(key) = event::read()? {
            // Global Exit: Ctrl+C closes the program cleanly from any mode
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                break;
            }

            match app.mode {
                InputMode::Normal => match key.code {
                    KeyCode::Esc => {
                        app.clear_search();
                    }
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    KeyCode::Right | KeyCode::Enter => app.enter_directory(),
                    KeyCode::Left => app.go_up(), // <--- Backspace removed from here
                    KeyCode::Char(c) => {
                        app.mode = InputMode::Searching;
                        app.search_query.push(c);
                        app.apply_filter();
                    }
                    _ => {}
                },
                InputMode::Searching => match key.code {
                    KeyCode::Esc => {
                        app.clear_search();
                    }
                    KeyCode::Enter => {
                        app.enter_directory();
                    }
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    KeyCode::Backspace => {
                        app.search_query.pop();
                        if app.search_query.is_empty() {
                            app.mode = InputMode::Normal;
                        }
                        app.apply_filter();
                    }
                    KeyCode::Char(c) => {
                        app.search_query.push(c);
                        app.apply_filter();
                    }
                    _ => {}
                },
            }
        }
    }

    // Restore terminal state before exiting
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}