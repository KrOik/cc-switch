use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// 按钮组件
#[derive(Clone)]
pub struct Button {
    pub label: String,
    pub is_focused: bool,
    pub is_primary: bool,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            is_focused: false,
            is_primary: false,
        }
    }

    pub fn primary(mut self) -> Self {
        self.is_primary = true;
        self
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    /// 处理键盘输入
    pub fn handle_key(&mut self, key: KeyCode) -> bool {
        matches!(key, KeyCode::Enter | KeyCode::Char(' '))
    }

    /// 渲染按钮
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let style = if self.is_focused {
            if self.is_primary {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD)
            }
        } else {
            if self.is_primary {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Gray)
            }
        };

        let border_style = if self.is_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        };

        let text = if self.is_focused {
            format!("[ {} ]", self.label)
        } else {
            format!("  {}  ", self.label)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        f.render_widget(block, area);

        let paragraph = Paragraph::new(Line::from(text)).style(style);
        f.render_widget(paragraph, inner);
    }
}
