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
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                // 如果有对话框，优先处理对话框输入
                if app.has_dialog() {
                    if let Ok(handled) = app.handle_dialog_key(code) {
                        if !handled {
                            // 用户确认了操作，需要执行待处理的操作
                            // 这里暂时只是关闭对话框，实际操作需要异步处理
                            // TODO: 集成异步操作执行
                        }
                    }
                } else {
                    // 正常的键盘处理
                    app.handle_key_extended(code, modifiers)?;
                }
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
    // 渲染主界面
    match app.mode {
        AppMode::Dashboard => ui::dashboard::render(f, app),
        AppMode::Providers => ui::providers::render(f, app),
        AppMode::Proxy => ui::proxy::render(f, app),
        AppMode::Mcp => ui::mcp::render(f, app),
        AppMode::Universal => ui::universal::render(f, app),
        AppMode::Config => ui::config::render(f, app),
        AppMode::ProviderForm => {
            if let Some(form) = &app.provider_form {
                form.render(f, f.area());
            }
        }
    }

    // 如果有对话框，渲染在最上层
    if let Some(dialog) = app.get_dialog() {
        ui::dialog::render_confirm_dialog(f, dialog);
    }
}
