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
            Constraint::Min(0),     // 统一供应商列表
            Constraint::Length(6),  // 详情
            Constraint::Length(3),  // 帮助
        ])
        .split(f.area());

    // 标题
    render_title(
        f,
        chunks[0],
        "统一供应商管理",
        Some("统一管理多个应用的供应商配置"),
    );

    // 统一供应商列表
    render_universal_list(f, chunks[1], app);

    // 详情
    render_universal_details(f, chunks[2], app);

    // 帮助
    let shortcuts = vec![
        ("↑/↓", "选择"),
        ("S", "同步"),
        ("A", "添加"),
        ("E", "编辑"),
        ("D", "删除"),
        ("Esc", "返回"),
    ];
    render_help(f, chunks[3], &shortcuts);
}

fn render_universal_list(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let providers = app.get_universal_providers();

    if providers.is_empty() {
        let empty = Paragraph::new(
            "暂无统一供应商配置\n\n统一供应商可以同时管理多个应用的配置\n按 [A] 添加新的统一供应商",
        )
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("统一供应商"));
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = providers
        .iter()
        .map(|(_id, provider)| {
            let enabled_apps = count_enabled_apps_universal(provider);
            let status = if enabled_apps > 0 {
                format!("✓ {} 个应用", enabled_apps)
            } else {
                "✗ 未启用".to_string()
            };

            let style = if enabled_apps > 0 {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Gray)
            };

            let content = format!("  {} - {}", provider.name, status);
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("统一供应商 ({})", providers.len())),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    state.select(app.get_selected_universal_index());
    f.render_stateful_widget(list, area, &mut state);
}

fn render_universal_details(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let selected = app.get_selected_universal_provider();

    let lines = if let Some(provider) = selected {
        vec![
            Line::from(vec![
                Span::styled("名称: ", Style::default().fg(Color::Gray)),
                Span::raw(&provider.name),
            ]),
            Line::from(vec![
                Span::styled("Claude: ", Style::default().fg(Color::Gray)),
                Span::raw(if provider.enabled_apps.claude { "✓" } else { "✗" }),
            ]),
            Line::from(vec![
                Span::styled("Codex: ", Style::default().fg(Color::Gray)),
                Span::raw(if provider.enabled_apps.codex { "✓" } else { "✗" }),
            ]),
            Line::from(vec![
                Span::styled("Gemini: ", Style::default().fg(Color::Gray)),
                Span::raw(if provider.enabled_apps.gemini { "✓" } else { "✗" }),
            ]),
        ]
    } else {
        vec![Line::from("未选择统一供应商")]
    };

    let widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("详细信息"));
    f.render_widget(widget, area);
}

fn count_enabled_apps_universal(provider: &crate::app::UniversalProviderStub) -> usize {
    let mut count = 0;
    if provider.enabled_apps.claude {
        count += 1;
    }
    if provider.enabled_apps.codex {
        count += 1;
    }
    if provider.enabled_apps.gemini {
        count += 1;
    }
    count
}
