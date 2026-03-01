use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::App;
use super::common::{render_help, status_indicator};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // 标题
            Constraint::Length(5),  // 代理状态
            Constraint::Length(7),  // 活跃供应商
            Constraint::Min(0),     // 帮助
        ])
        .split(f.area());

    // 标题
    let title = Paragraph::new("CC-Switch TUI v3.11.1")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // 代理状态
    render_proxy_status(f, chunks[1], app);

    // 活跃供应商
    render_active_providers(f, chunks[2], app);

    // 帮助
    let shortcuts = vec![
        ("P", "Providers"),
        ("X", "Proxy"),
        ("M", "MCP"),
        ("U", "Universal"),
        ("C", "Config"),
        ("Q", "Quit"),
    ];
    render_help(f, chunks[3], &shortcuts);
}

fn render_proxy_status(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let is_running = app.is_proxy_running();
    let config = app.get_proxy_config();

    let proxy_text = vec![
        Line::from(vec![
            Span::styled("状态: ", Style::default().fg(Color::Gray)),
            status_indicator(is_running),
        ]),
        Line::from(vec![
            Span::styled("监听地址: ", Style::default().fg(Color::Gray)),
            Span::raw(format!("{}:{}", config.listen_address, config.listen_port)),
        ]),
    ];

    let proxy_widget = Paragraph::new(proxy_text)
        .block(Block::default().borders(Borders::ALL).title("代理服务"));
    f.render_widget(proxy_widget, area);
}

fn render_active_providers(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let providers = app.get_active_providers();
    let default = "未配置".to_string();

    let provider_lines = vec![
        Line::from(vec![
            Span::styled("Claude:  ", Style::default().fg(Color::Gray)),
            Span::raw(providers.get("claude").unwrap_or(&default).clone()),
        ]),
        Line::from(vec![
            Span::styled("Codex:   ", Style::default().fg(Color::Gray)),
            Span::raw(providers.get("codex").unwrap_or(&default).clone()),
        ]),
        Line::from(vec![
            Span::styled("Gemini:  ", Style::default().fg(Color::Gray)),
            Span::raw(providers.get("gemini").unwrap_or(&default).clone()),
        ]),
        Line::from(vec![
            Span::styled("OpenCode:", Style::default().fg(Color::Gray)),
            Span::raw(providers.get("opencode").unwrap_or(&default).clone()),
        ]),
        Line::from(vec![
            Span::styled("OpenClaw:", Style::default().fg(Color::Gray)),
            Span::raw(providers.get("openclaw").unwrap_or(&default).clone()),
        ]),
    ];

    let providers_widget = Paragraph::new(provider_lines)
        .block(Block::default().borders(Borders::ALL).title("活跃供应商"));
    f.render_widget(providers_widget, area);
}
