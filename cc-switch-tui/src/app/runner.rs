use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal, Frame};
use tokio::runtime::Runtime;

use super::{App, AppMode};
use crate::ui;
use super::input::AppAction;

pub fn run_tui() -> Result<()> {
    enable_raw_mode()?;
    std::io::stdout().execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;
    let rt = Runtime::new()?;

    loop {
        terminal.draw(|f| render_ui(f, &app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                // 如果有对话框，优先处理对话框输入
                if app.has_dialog() {
                    if let Ok(handled) = app.handle_dialog_key(code) {
                        if !handled {
                            // 用户确认了操作，执行待处理的操作
                            if let Some(action) = app.take_pending_action() {
                                if let Err(e) = execute_pending_action(&rt, &mut app, action) {
                                    log::error!("Failed to execute action: {}", e);
                                }
                            }
                        }
                    }
                } else {
                    // 正常的键盘处理
                    if let Ok(Some(action)) = app.handle_key_extended(code, modifiers) {
                        if let Err(e) = execute_action(&rt, &mut app, action) {
                            log::error!("Failed to execute action: {}", e);
                        }
                    }
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
            // 优先渲染 V2 表单
            if let Some(form) = &app.provider_form_v2 {
                form.render(f, f.area());
            } else if let Some(form) = &app.provider_form {
                form.render(f, f.area());
            }
        }
        AppMode::McpForm => {
            if let Some(form) = &app.mcp_form {
                form.render(f, f.area());
            }
        }
        AppMode::UniversalForm => {
            if let Some(form) = &app.universal_form {
                form.render(f, f.area());
            }
        }
    }

    // 如果有对话框，渲染在最上层
    if let Some(dialog) = app.get_dialog() {
        ui::dialog::render_confirm_dialog(f, dialog);
    }
}

/// 执行应用操作
fn execute_action(rt: &Runtime, app: &mut App, action: AppAction) -> Result<()> {
    match action {
        AppAction::SwitchProvider(id) => {
            rt.block_on(app.switch_provider(&id))?;
        }
        AppAction::SaveProvider(data) => {
            rt.block_on(app.save_provider(data))?;
        }
        AppAction::SaveProviderV2(data) => {
            rt.block_on(app.save_provider_v2(data))?;
        }
        AppAction::SaveMcpServer(data) => {
            rt.block_on(app.save_mcp_server(data))?;
        }
        AppAction::SaveUniversalProvider(data) => {
            rt.block_on(app.save_universal_provider(data))?;
        }
        AppAction::StartProxy => {
            rt.block_on(app.start_proxy())?;
        }
        AppAction::StopProxy => {
            rt.block_on(app.stop_proxy())?;
        }
        AppAction::RefreshProxyStatus => {
            rt.block_on(app.refresh_proxy_status_async())?;
        }
        AppAction::DeleteProvider(id) => {
            rt.block_on(app.delete_provider(&id))?;
        }
        AppAction::DeleteMcpServer(id) => {
            rt.block_on(app.delete_mcp_server(&id))?;
        }
        AppAction::DeleteUniversalProvider(id) => {
            rt.block_on(app.delete_universal_provider(&id))?;
        }
        AppAction::SyncUniversalProvider(id) => {
            rt.block_on(app.sync_universal_provider(&id))?;
        }
        AppAction::RestartProxy => {
            rt.block_on(app.stop_proxy())?;
            rt.block_on(app.start_proxy())?;
        }
        AppAction::ToggleProxyTakeover(app_type, enabled) => {
            rt.block_on(app.toggle_proxy_takeover(&app_type, enabled))?;
        }
    }
    Ok(())
}

/// 执行待处理的操作（从对话框确认后）
fn execute_pending_action(rt: &Runtime, app: &mut App, action: super::PendingAction) -> Result<()> {
    match action {
        super::PendingAction::DeleteProvider(id) => {
            rt.block_on(app.delete_provider(&id))?;
        }
        super::PendingAction::DeleteMcpServer(id) => {
            rt.block_on(app.delete_mcp_server(&id))?;
        }
        super::PendingAction::DeleteUniversalProvider(id) => {
            rt.block_on(app.delete_universal_provider(&id))?;
        }
        super::PendingAction::StopProxy => {
            rt.block_on(app.stop_proxy())?;
        }
        _ => {}
    }
    Ok(())
}
