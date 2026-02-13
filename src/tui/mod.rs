use crate::config::WatchedSite;
use crate::monitor::MonitorEvent;
use crate::Config;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;
use tokio::sync::mpsc::Receiver;

pub struct App {
    pub sites: Vec<WatchedSite>,
    pub list_state: ListState,
    pub logs: Vec<String>,
    pub should_quit: bool,
    pub input_mode: InputMode,
    pub input: String,
    pub status_message: String,
    pub has_token: bool,
    pub has_chat_id: bool,
    pub pending_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    AddUrl,
    AddName,
    AddSelector,
    SetToken,
    Help,
}

impl App {
    pub fn new(sites: Vec<WatchedSite>, has_token: bool, has_chat_id: bool) -> Self {
        Self {
            sites,
            list_state: ListState::default(),
            logs: Vec::new(),
            should_quit: false,
            input_mode: InputMode::Normal,
            input: String::new(),
            status_message: String::new(),
            has_token,
            has_chat_id,
            pending_url: None,
        }
    }

    pub fn add_log(&mut self, message: String) {
        let timestamp = chrono::Local::now().format("%H:%M:%S");
        self.logs.push(format!("[{}] {}", timestamp, message));
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }

    pub fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.sites.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.sites.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}

pub fn run_tui(config: &mut Config, mut events_rx: Receiver<MonitorEvent>) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let sites = config.app_config.sites.clone();
    let has_token = config.has_telegram_token();
    let has_chat_id = config.app_config.telegram_chat_id.is_some();
    let mut app = App::new(sites, has_token, has_chat_id);
    app.list_state.select(Some(0));

    let res = run_app(&mut terminal, &mut app, &mut events_rx, config);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    events_rx: &mut Receiver<MonitorEvent>,
    config: &mut Config,
) -> anyhow::Result<()> {
    loop {
        if let Ok(event) = events_rx.try_recv() {
            match event {
                MonitorEvent::SiteChecked { site_id, changed } => {
                    if let Some(site) = app.sites.iter().find(|s| s.id == site_id) {
                        app.add_log(format!(
                            "Checked: {} ({})",
                            site.name,
                            if changed { "changed" } else { "no change" }
                        ));
                    }
                }
                MonitorEvent::SiteChanged { site_id, diff } => {
                    if let Some(site) = app.sites.iter().find(|s| s.id == site_id) {
                        app.add_log(format!(
                            "CHANGED: {} - {}",
                            site.name,
                            diff.chars().take(50).collect::<String>()
                        ));
                    }
                }
                MonitorEvent::Error { site_id, error } => {
                    app.add_log(format!("Error on {}: {}", site_id, error));
                }
            }
        }

        terminal.draw(|f| ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => {
                            app.should_quit = true;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.next();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.previous();
                        }
                        KeyCode::Char('a') => {
                            app.input_mode = InputMode::AddUrl;
                            app.input.clear();
                            app.status_message = "Enter URL:".to_string();
                        }
                        KeyCode::Char('d') => {
                            if let Some(i) = app.list_state.selected() {
                                if i < app.sites.len() {
                                    let site_name = app.sites[i].name.clone();
                                    let site_id = app.sites[i].id.clone();
                                    app.add_log(format!("Deleted: {}", site_name));
                                    let _ = config.remove_site(&site_id);
                                    app.sites.remove(i);
                                    if !app.sites.is_empty() {
                                        app.list_state.select(Some(0));
                                    }
                                }
                            }
                        }
                        KeyCode::Char('t') => {
                            app.input_mode = InputMode::SetToken;
                            app.input.clear();
                            app.status_message = "Enter Telegram Token:".to_string();
                        }
                        KeyCode::Char('?') => {
                            app.input_mode = InputMode::Help;
                        }
                        KeyCode::Char('r') => {
                            if let Some(i) = app.list_state.selected() {
                                if i < app.sites.len() {
                                    app.add_log(format!("Refreshing: {}", app.sites[i].name));
                                }
                            }
                        }
                        _ => {}
                    },
                    InputMode::AddUrl => match key.code {
                        KeyCode::Enter => {
                            if !app.input.is_empty() {
                                app.pending_url = Some(app.input.clone());
                                app.input.clear();
                                app.status_message = "Enter name (or empty for auto):".to_string();
                                app.input_mode = InputMode::AddName;
                            }
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                            app.input.clear();
                            app.status_message.clear();
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        _ => {}
                    },
                    InputMode::AddName => match key.code {
                        KeyCode::Enter => {
                            let url = app.pending_url.take().unwrap_or_default();
                            let name = if app.input.is_empty() {
                                if let Some(pos) = url.find("://") {
                                    let domain = &url[pos + 3..];
                                    domain.split('/').next().unwrap_or(&url).to_string()
                                } else {
                                    url.clone()
                                }
                            } else {
                                app.input.clone()
                            };

                            app.input.clear();
                            app.status_message =
                                "Enter CSS selector (or empty for body):".to_string();
                            app.pending_url = Some(url);
                            app.input = name;
                            app.input_mode = InputMode::AddSelector;
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                            app.input.clear();
                            app.status_message.clear();
                            app.pending_url = None;
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        _ => {}
                    },
                    InputMode::AddSelector => match key.code {
                        KeyCode::Enter => {
                            let url = app.pending_url.take().unwrap_or_default();
                            let name = app.input.clone();
                            
                            let selector = if app.input.is_empty() || app.input == name {
                                None
                            } else {
                                Some(app.input.clone())
                            };

                            match config.add_site(url.clone(), name.clone(), selector) {
                                Ok(id) => {
                                    let site = WatchedSite {
                                        id,
                                        url,
                                        name,
                                        last_hash: None,
                                        last_checked: None,
                                        last_change: None,
                                        enabled: true,
                                        css_selector: None,
                                    };
                                    app.add_log(format!("Added: {}", site.name));
                                    app.sites.push(site);
                                }
                                Err(e) => {
                                    app.add_log(format!("Error adding site: {}", e));
                                }
                            }
                            app.input_mode = InputMode::Normal;
                            app.input.clear();
                            app.status_message.clear();
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                            app.input.clear();
                            app.status_message.clear();
                            app.pending_url = None;
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        _ => {}
                    },
                    InputMode::SetToken => match key.code {
                        KeyCode::Enter => {
                            if !app.input.is_empty() {
                                match config.set_telegram_token(&app.input) {
                                    Ok(()) => {
                                        app.has_token = true;
                                        app.add_log("Telegram token saved securely".to_string());
                                    }
                                    Err(e) => {
                                        app.add_log(format!("Error saving token: {}", e));
                                    }
                                }
                            }
                            app.input_mode = InputMode::Normal;
                            app.input.clear();
                            app.status_message.clear();
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                            app.input.clear();
                            app.status_message.clear();
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        _ => {}
                    },
                    InputMode::Help => {
                        app.input_mode = InputMode::Normal;
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    if app.input_mode == InputMode::Help {
        render_help(f);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(f.area());

    let sites: Vec<ListItem> = app
        .sites
        .iter()
        .map(|site| {
            let status = if site.enabled { "✓" } else { "✗" };
            let checked = site
                .last_checked
                .map(|t| t.format("%H:%M:%S").to_string())
                .unwrap_or_else(|| "Never".to_string());

            ListItem::new(Line::from(vec![
                Span::styled(status.to_string() + " ", Style::default().fg(Color::Green)),
                Span::styled(
                    site.name.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - "),
                Span::styled(checked, Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();

    let mut title = "Watched Sites (a: add, d: delete, t: token, ?: help, q: quit)".to_string();
    if !app.has_token {
        title = "⚠ No Token Set! Press 't' to set token".to_string();
    }

    let sites_list = List::new(sites)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("> ");

    let mut sites_state = app.list_state.clone();
    f.render_stateful_widget(sites_list, chunks[0], &mut sites_state);

    let logs: Vec<ListItem> = app
        .logs
        .iter()
        .rev()
        .take(20)
        .map(|log| ListItem::new(Line::from(log.clone())))
        .collect();

    let logs_list =
        List::new(logs).block(Block::default().title("Activity Log").borders(Borders::ALL));

    f.render_widget(logs_list, chunks[1]);

    if !app.status_message.is_empty() || !app.input.is_empty() {
        let input_area = Rect::new(chunks[0].x, chunks[0].bottom() + 1, chunks[0].width, 3);

        let display_input = if app.input_mode == InputMode::SetToken {
            "*".repeat(app.input.len())
        } else {
            app.input.clone()
        };

        let input_widget = Paragraph::new(format!("{} {}", app.status_message, display_input))
            .block(Block::default().borders(Borders::ALL).title("Input"));
        f.render_widget(input_widget, input_area);
    }
}

fn render_help(f: &mut Frame) {
    let help_text = vec![
        "RNOT - Website Monitor Help",
        "",
        "Keybindings:",
        "  a     - Add a new site to watch",
        "  d     - Delete selected site",
        "  t     - Set Telegram bot token",
        "  r     - Refresh selected site",
        "  j/↓   - Move down in list",
        "  k/↑   - Move up in list",
        "  q     - Quit",
        "  ?     - Show this help",
        "",
        "CLI Commands:",
        "  rnot add <URL> [--name NAME] [--selector CSS]",
        "  rnot remove <ID|URL>",
        "  rnot list",
        "  rnot set-token <TOKEN>",
        "  rnot telegram-setup",
        "  rnot daemon",
        "  rnot check",
        "  rnot status",
        "",
        "Press any key to close this help",
    ];

    let help_items: Vec<ListItem> = help_text
        .iter()
        .map(|line| ListItem::new(Line::from(*line)))
        .collect();

    let help_list =
        List::new(help_items).block(Block::default().title("Help").borders(Borders::ALL));

    let area = centered_rect(80, 80, f.area());
    f.render_widget(help_list, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
