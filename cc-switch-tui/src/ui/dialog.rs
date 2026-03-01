use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// 确认对话框状态
#[derive(Clone)]
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    pub confirm_text: String,
    pub cancel_text: String,
    pub selected: ConfirmButton,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ConfirmButton {
    Confirm,
    Cancel,
}

impl ConfirmDialog {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            confirm_text: "确认".to_string(),
            cancel_text: "取消".to_string(),
            selected: ConfirmButton::Cancel, // 默认选中取消，更安全
        }
    }

    pub fn with_confirm_text(mut self, text: impl Into<String>) -> Self {
        self.confirm_text = text.into();
        self
    }

    pub fn with_cancel_text(mut self, text: impl Into<String>) -> Self {
        self.cancel_text = text.into();
        self
    }

    pub fn toggle_selection(&mut self) {
        self.selected = match self.selected {
            ConfirmButton::Confirm => ConfirmButton::Cancel,
            ConfirmButton::Cancel => ConfirmButton::Confirm,
        };
    }

    pub fn is_confirm_selected(&self) -> bool {
        self.selected == ConfirmButton::Confirm
    }
}

/// 渲染确认对话框
pub fn render_confirm_dialog(f: &mut Frame, dialog: &ConfirmDialog) {
    let area = centered_rect(60, 40, f.area());

    // 清除背景
    f.render_widget(Clear, area);

    // 主容器
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(dialog.title.clone())
        .title_alignment(Alignment::Center);

    let inner = block.inner(area);
    f.render_widget(block, area);

    // 分割区域：消息 + 按钮
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),      // 消息区域
            Constraint::Length(3),   // 按钮区域
        ])
        .split(inner);

    // 渲染消息
    let message = Paragraph::new(dialog.message.clone())
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::White));
    f.render_widget(message, chunks[0]);

    // 渲染按钮
    render_buttons(f, chunks[1], dialog);
}

fn render_buttons(f: &mut Frame, area: Rect, dialog: &ConfirmDialog) {
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    // 确认按钮
    let confirm_style = if dialog.selected == ConfirmButton::Confirm {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };

    let confirm_button = Paragraph::new(Line::from(vec![
        Span::raw("[ "),
        Span::styled(&dialog.confirm_text, confirm_style),
        Span::raw(" ]"),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(confirm_button, button_chunks[1]);

    // 取消按钮
    let cancel_style = if dialog.selected == ConfirmButton::Cancel {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Red)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red)
    };

    let cancel_button = Paragraph::new(Line::from(vec![
        Span::raw("[ "),
        Span::styled(&dialog.cancel_text, cancel_style),
        Span::raw(" ]"),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(cancel_button, button_chunks[2]);

    // 提示文本
    let hint = Paragraph::new("← → 切换  Enter 确认  Esc 取消")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));

    let hint_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };
    f.render_widget(hint, hint_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
