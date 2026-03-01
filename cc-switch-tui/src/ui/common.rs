use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// 渲染标题栏
pub fn render_title(f: &mut Frame, area: Rect, title: &str, subtitle: Option<&str>) {
    let title_text = if let Some(sub) = subtitle {
        vec![
            Line::from(vec![
                Span::styled(title, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(sub, Style::default().fg(Color::Gray)),
            ]),
        ]
    } else {
        vec![Line::from(vec![
            Span::styled(title, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ])]
    };

    let title_widget = Paragraph::new(title_text)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title_widget, area);
}

/// 渲染帮助栏
pub fn render_help(f: &mut Frame, area: Rect, shortcuts: &[(&str, &str)]) {
    let help_spans: Vec<Span> = shortcuts
        .iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(format!("[{}]", key), Style::default().fg(Color::Yellow)),
                Span::raw(format!(" {}  ", desc)),
            ]
        })
        .collect();

    let help_widget = Paragraph::new(Line::from(help_spans))
        .block(Block::default().borders(Borders::ALL).title("快捷键"));
    f.render_widget(help_widget, area);
}

/// 渲染状态指示器
pub fn status_indicator(is_active: bool) -> Span<'static> {
    if is_active {
        Span::styled("● 运行中", Style::default().fg(Color::Green))
    } else {
        Span::styled("○ 已停止", Style::default().fg(Color::Red))
    }
}

/// 渲染列表项
pub fn create_list_items<'a>(items: Vec<String>) -> Vec<ListItem<'a>> {
    items
        .into_iter()
        .map(|item| ListItem::new(item))
        .collect()
}

/// 创建三列布局
pub fn three_column_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // 标题
            Constraint::Min(0),     // 内容
            Constraint::Length(3),  // 帮助
        ])
        .split(area)
        .to_vec()
}

/// 创建两列布局
pub fn two_column_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area)
        .to_vec()
}

/// 渲染空状态
pub fn render_empty_state(f: &mut Frame, area: Rect, message: &str) {
    let empty_text = Paragraph::new(message)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(empty_text, area);
}

/// 渲染加载状态
pub fn render_loading(f: &mut Frame, area: Rect) {
    let loading_text = Paragraph::new("加载中...")
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(loading_text, area);
}
