use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal, Frame};

use super::{App, AppMode};
use crate::ui;

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
        AppMode::Dashboard => ui::dashboard::render(f, app),
        AppMode::Providers => ui::providers::render(f, app),
        AppMode::Proxy => ui::proxy::render(f, app),
        AppMode::Mcp => ui::mcp::render(f, app),
        AppMode::Universal => ui::universal::render(f, app),
        AppMode::Config => ui::config::render(f, app),
    }
}
