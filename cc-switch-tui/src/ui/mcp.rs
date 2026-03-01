use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, ListItem, ListState, Paragraph, List},
    Frame,
};
use crate::app::App;
use super::common::{render_title, render_help};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // 标题
            Constraint::Min(0),     // MCP 服务器列表
            Constraint::Length(6),  // 详情
            Constraint::Length(3),  // 帮助
        ])
        .split(f.area());

    // 标题
    render_title(
        f,
        chunks[0],
        "MCP 服务器管理",
        Some("管理 Model Context Protocol 服务器"),
    );

    // MCP 服务器列表
    render_mcp_list(f, chunks[1], app);

    // 详情
    render_mcp_details(f, chunks[2], app);

    // 帮助
    let shortcuts = vec![
        ("↑/↓", "选择"),
        ("Space", "启用/禁用"),
        ("A", "添加"),
        ("E", "编辑"),
        ("D", "删除"),
        ("Esc", "返回"),
    ];
    render_help(f, chunks[3], &shortcuts);
}

fn render_mcp_list(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let servers = app.get_mcp_servers();

    if servers.is_empty() {
        let empty = Paragraph::new("暂无 MCP 服务器配置\n\n按 [A] 添加新服务器")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title("MCP 服务器"));
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = servers
        .iter()
        .map(|(id, server)| {
            let name = if server.name.is_empty() {
                id
            } else {
                &server.name
            };
            let enabled_count = count_enabled_apps(server);
            let status = if enabled_count > 0 {
                format!("✓ {} 个应用", enabled_count)
            } else {
                "✗ 未启用".to_string()
            };

            let style = if enabled_count > 0 {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Gray)
            };

            let content = format!("  {} - {}", name, status);
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("MCP 服务器 ({})", servers.len())),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    state.select(app.get_selected_mcp_index());
    f.render_stateful_widget(list, area, &mut state);
}

fn render_mcp_details(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let selected = app.get_selected_mcp_server();

    let lines = if let Some(server) = selected {
        let name = if server.name.is_empty() {
            &server.id
        } else {
            &server.name
        };
        vec![
            Line::from(vec![
                Span::styled("名称: ", Style::default().fg(Color::Gray)),
                Span::raw(name),
            ]),
            Line::from(vec![
                Span::styled("Claude: ", Style::default().fg(Color::Gray)),
                Span::raw(if server.apps.claude { "✓" } else { "✗" }),
            ]),
            Line::from(vec![
                Span::styled("Codex: ", Style::default().fg(Color::Gray)),
                Span::raw(if server.apps.codex { "✓" } else { "✗" }),
            ]),
            Line::from(vec![
                Span::styled("Gemini: ", Style::default().fg(Color::Gray)),
                Span::raw(if server.apps.gemini { "✓" } else { "✗" }),
            ]),
        ]
    } else {
        vec![Line::from("未选择 MCP 服务器")]
    };

    let widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("详细信息"));
    f.render_widget(widget, area);
}

fn count_enabled_apps(server: &cc_switch_core::McpServer) -> usize {
    let mut count = 0;
    if server.apps.claude {
        count += 1;
    }
    if server.apps.codex {
        count += 1;
    }
    if server.apps.gemini {
        count += 1;
    }
    if server.apps.opencode {
        count += 1;
    }
    // Note: openclaw field may not exist in older McpApps versions
    // if server.apps.openclaw {
    //     count += 1;
    // }
    count
}
