use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::App;
use super::common::{render_title, render_help, status_indicator};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // 标题
            Constraint::Length(8),  // 代理状态详情
            Constraint::Length(6),  // 配置信息
            Constraint::Length(6),  // 统计信息
            Constraint::Min(0),     // 帮助
        ])
        .split(f.area());

    // 标题
    render_title(f, chunks[0], "代理控制", Some("管理代理服务器的启动、停止和配置"));

    // 代理状态详情
    render_proxy_details(f, chunks[1], app);

    // 配置信息
    render_proxy_config(f, chunks[2], app);

    // 统计信息
    render_proxy_stats(f, chunks[3], app);

    // 帮助
    let shortcuts = vec![
        ("S", "启动/停止"),
        ("R", "重启"),
        ("1", "切换Claude接管"),
        ("2", "切换Codex接管"),
        ("3", "切换Gemini接管"),
        ("Esc", "返回"),
    ];
    render_help(f, chunks[4], &shortcuts);
}

fn render_proxy_details(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let is_running = app.is_proxy_running();
    let status = app.get_proxy_status();

    let mut lines = vec![
        Line::from(vec![
            Span::styled("服务状态: ", Style::default().fg(Color::Gray)),
            status_indicator(is_running),
        ]),
    ];

    if is_running {
        if let Some(status) = status {
            lines.push(Line::from(vec![
                Span::styled("运行时间: ", Style::default().fg(Color::Gray)),
                Span::raw(format_uptime(status.uptime_seconds)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("活跃连接: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    status.active_connections.to_string(),
                    Style::default().fg(Color::Green),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled("总请求数: ", Style::default().fg(Color::Gray)),
                Span::raw(status.total_requests.to_string()),
            ]));
            lines.push(Line::from(vec![
                Span::styled("成功率: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:.1}%", status.success_rate),
                    if status.success_rate > 90.0 {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Yellow)
                    },
                ),
            ]));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "代理服务未运行",
            Style::default().fg(Color::Yellow),
        )));
    }

    let widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("服务详情"));
    f.render_widget(widget, area);
}

fn render_proxy_config(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let config = app.get_proxy_config();

    let lines = vec![
        Line::from(vec![
            Span::styled("监听地址: ", Style::default().fg(Color::Gray)),
            Span::raw(&config.listen_address),
        ]),
        Line::from(vec![
            Span::styled("监听端口: ", Style::default().fg(Color::Gray)),
            Span::raw(config.listen_port.to_string()),
        ]),
        Line::from(vec![
            Span::styled("日志记录: ", Style::default().fg(Color::Gray)),
            Span::raw(if config.enable_logging { "启用" } else { "禁用" }),
        ]),
    ];

    let widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("配置信息"));
    f.render_widget(widget, area);
}

fn render_proxy_stats(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let takeover = app.get_proxy_takeover_status();

    let lines = vec![
        Line::from(vec![
            Span::styled("[1] Claude 接管: ", Style::default().fg(Color::Gray)),
            Span::styled(
                if takeover.claude { "✓ 启用" } else { "✗ 禁用" },
                if takeover.claude {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
        ]),
        Line::from(vec![
            Span::styled("[2] Codex 接管:  ", Style::default().fg(Color::Gray)),
            Span::styled(
                if takeover.codex { "✓ 启用" } else { "✗ 禁用" },
                if takeover.codex {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
        ]),
        Line::from(vec![
            Span::styled("[3] Gemini 接管: ", Style::default().fg(Color::Gray)),
            Span::styled(
                if takeover.gemini { "✓ 启用" } else { "✗ 禁用" },
                if takeover.gemini {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
        ]),
    ];

    let widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("接管状态 (按数字键切换)"));
    f.render_widget(widget, area);
}

fn format_uptime(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}
