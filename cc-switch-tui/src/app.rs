use anyhow::Result;
use cc_switch_core::{Database, ProxyService};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::sync::Arc;

pub enum AppMode {
    Dashboard,
    Providers,
    Proxy,
    Mcp,
    Config,
    Stats,
    Logs,
}

pub struct App {
    mode: AppMode,
    should_quit: bool,
    db: Arc<Database>,
    proxy_service: ProxyService,
}

impl App {
    pub fn new() -> Result<Self> {
        let db = Arc::new(Database::init()?);
        let proxy_service = ProxyService::new(db.clone());

        Ok(Self {
            mode: AppMode::Dashboard,
            should_quit: false,
            db,
            proxy_service,
        })
    }

    pub fn handle_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') | KeyCode::Char('Q') => self.should_quit = true,
            KeyCode::Char('p') | KeyCode::Char('P') => self.mode = AppMode::Providers,
            KeyCode::Char('x') | KeyCode::Char('X') => self.mode = AppMode::Proxy,
            KeyCode::Char('m') | KeyCode::Char('M') => self.mode = AppMode::Mcp,
            KeyCode::Char('s') | KeyCode::Char('S') => self.mode = AppMode::Stats,
            KeyCode::Char('c') | KeyCode::Char('C') => self.mode = AppMode::Config,
            KeyCode::Char('l') | KeyCode::Char('L') => self.mode = AppMode::Logs,
            KeyCode::Esc => self.mode = AppMode::Dashboard,
            _ => {}
        }
        Ok(())
    }
}

pub fn run_tui() -> Result<()> {
    enable_raw_mode()?;
    std::io::stdout().execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;

    loop {
        terminal.draw(|f| render_ui(f, &app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                app.handle_key(code)?;
            }
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    std::io::stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

fn render_ui(f: &mut Frame, app: &App) {
    match app.mode {
        AppMode::Dashboard => render_dashboard(f, app),
        AppMode::Providers => render_placeholder(f, "Providers", app),
        AppMode::Proxy => render_placeholder(f, "Proxy Control", app),
        AppMode::Mcp => render_placeholder(f, "MCP Servers", app),
        AppMode::Config => render_placeholder(f, "Configuration", app),
        AppMode::Stats => render_placeholder(f, "Statistics", app),
        AppMode::Logs => render_placeholder(f, "Logs", app),
    }
}

fn render_dashboard(f: &mut Frame, _app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(5),  // Proxy status
            Constraint::Length(5),  // Active providers
            Constraint::Min(0),     // Help
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("CC-Switch TUI v3.11.1")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Proxy status
    let proxy_text = vec![
        Line::from(vec![
            Span::styled("Proxy Status: ", Style::default().fg(Color::Gray)),
            Span::styled("○ STOPPED", Style::default().fg(Color::Red)),
        ]),
        Line::from(vec![
            Span::styled("Listen: ", Style::default().fg(Color::Gray)),
            Span::raw("127.0.0.1:15721"),
        ]),
    ];
    let proxy_widget = Paragraph::new(proxy_text)
        .block(Block::default().borders(Borders::ALL).title("Proxy"));
    f.render_widget(proxy_widget, chunks[1]);

    // Active providers
    let providers_text = vec![
        Line::from("Claude:  Not configured"),
        Line::from("Codex:   Not configured"),
        Line::from("Gemini:  Not configured"),
    ];
    let providers_widget = Paragraph::new(providers_text)
        .block(Block::default().borders(Borders::ALL).title("Active Providers"));
    f.render_widget(providers_widget, chunks[2]);

    // Help
    let help_text = Line::from(vec![
        Span::styled("[P]", Style::default().fg(Color::Yellow)),
        Span::raw(" Providers  "),
        Span::styled("[X]", Style::default().fg(Color::Yellow)),
        Span::raw(" Proxy  "),
        Span::styled("[M]", Style::default().fg(Color::Yellow)),
        Span::raw(" MCP  "),
        Span::styled("[S]", Style::default().fg(Color::Yellow)),
        Span::raw(" Stats  "),
        Span::styled("[C]", Style::default().fg(Color::Yellow)),
        Span::raw(" Config  "),
        Span::styled("[L]", Style::default().fg(Color::Yellow)),
        Span::raw(" Logs  "),
        Span::styled("[Q]", Style::default().fg(Color::Yellow)),
        Span::raw(" Quit"),
    ]);
    let help_widget = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Quick Actions"));
    f.render_widget(help_widget, chunks[3]);
}

fn render_placeholder(f: &mut Frame, title: &str, _app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(f.area());

    let title_widget = Paragraph::new(format!("CC-Switch TUI - {}", title))
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title_widget, chunks[0]);

    let content = Paragraph::new(format!("{} view - Coming soon...\n\nPress [Esc] to return to dashboard", title))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(content, chunks[1]);
}
