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
            Constraint::Min(0),     // 供应商列表
            Constraint::Length(5),  // 详情
            Constraint::Length(3),  // 帮助
        ])
        .split(f.area());

    // 标题
    render_title(
        f,
        chunks[0],
        "Provider 管理",
        Some("查看和切换 AI 供应商配置"),
    );

    // 供应商列表
    render_provider_list(f, chunks[1], app);

    // 详情
    render_provider_details(f, chunks[2], app);

    // 帮助
    let shortcuts = vec![
        ("↑/↓", "选择"),
        ("Enter", "切换"),
        ("A", "添加"),
        ("E", "编辑"),
        ("D", "删除"),
        ("Esc", "返回"),
    ];
    render_help(f, chunks[3], &shortcuts);
}

fn render_provider_list(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let providers = app.get_providers_for_current_app();
    let current_id = app.get_current_provider_id();

    if providers.is_empty() {
        let empty = Paragraph::new("暂无供应商配置\n\n按 [A] 添加新供应商")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title("供应商列表"));
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = providers
        .iter()
        .map(|(id, provider)| {
            let is_current = id == &current_id;
            let prefix = if is_current { "● " } else { "  " };
            let style = if is_current {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let content = format!("{}{} - {}", prefix, provider.name, id);
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("供应商列表 ({})", providers.len())),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    state.select(app.get_selected_provider_index());
    f.render_stateful_widget(list, area, &mut state);
}

fn render_provider_details(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let selected = app.get_selected_provider();

    let lines = if let Some(provider) = selected {
        vec![
            Line::from(vec![
                Span::styled("名称: ", Style::default().fg(Color::Gray)),
                Span::raw(&provider.name),
            ]),
            Line::from(vec![
                Span::styled("ID: ", Style::default().fg(Color::Gray)),
                Span::raw(&provider.id),
            ]),
            Line::from(vec![
                Span::styled("网站: ", Style::default().fg(Color::Gray)),
                Span::raw(provider.website_url.as_deref().unwrap_or("未设置")),
            ]),
        ]
    } else {
        vec![Line::from("未选择供应商")]
    };

    let widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("详细信息"));
    f.render_widget(widget, area);
}
