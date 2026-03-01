use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::App;
use super::common::{render_title, render_help};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // 标题
            Constraint::Length(8),  // 数据库配置
            Constraint::Length(8),  // 应用路径
            Constraint::Min(0),     // 帮助
        ])
        .split(f.area());

    // 标题
    render_title(f, chunks[0], "配置信息", Some("查看系统配置和路径"));

    // 数据库配置
    render_database_config(f, chunks[1], app);

    // 应用路径
    render_app_paths(f, chunks[2], app);

    // 帮助
    let shortcuts = vec![("Esc", "返回")];
    render_help(f, chunks[3], &shortcuts);
}

fn render_database_config(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let config = app.get_config_info();

    let lines = vec![
        Line::from(vec![
            Span::styled("配置目录: ", Style::default().fg(Color::Gray)),
            Span::raw(&config.config_dir),
        ]),
        Line::from(vec![
            Span::styled("数据库路径: ", Style::default().fg(Color::Gray)),
            Span::raw(&config.database_path),
        ]),
        Line::from(vec![
            Span::styled("数据库大小: ", Style::default().fg(Color::Gray)),
            Span::raw(format_size(config.database_size)),
        ]),
        Line::from(vec![
            Span::styled("Provider 数量: ", Style::default().fg(Color::Gray)),
            Span::styled(
                config.total_providers.to_string(),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::styled("MCP 服务器: ", Style::default().fg(Color::Gray)),
            Span::styled(
                config.total_mcp_servers.to_string(),
                Style::default().fg(Color::Green),
            ),
        ]),
    ];

    let widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("数据库配置"));
    f.render_widget(widget, area);
}

fn render_app_paths(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let paths = app.get_app_paths();
    let default = "未配置".to_string();

    let lines = vec![
        Line::from(vec![
            Span::styled("Claude: ", Style::default().fg(Color::Gray)),
            Span::raw(paths.get("claude").unwrap_or(&default).clone()),
        ]),
        Line::from(vec![
            Span::styled("Codex:  ", Style::default().fg(Color::Gray)),
            Span::raw(paths.get("codex").unwrap_or(&default).clone()),
        ]),
        Line::from(vec![
            Span::styled("Gemini: ", Style::default().fg(Color::Gray)),
            Span::raw(paths.get("gemini").unwrap_or(&default).clone()),
        ]),
        Line::from(vec![
            Span::styled("OpenCode: ", Style::default().fg(Color::Gray)),
            Span::raw(paths.get("opencode").unwrap_or(&default).clone()),
        ]),
        Line::from(vec![
            Span::styled("OpenClaw: ", Style::default().fg(Color::Gray)),
            Span::raw(paths.get("openclaw").unwrap_or(&default).clone()),
        ]),
    ];

    let widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("应用路径"));
    f.render_widget(widget, area);
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
