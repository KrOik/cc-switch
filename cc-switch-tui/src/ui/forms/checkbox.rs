use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// 复选框组件
#[derive(Clone)]
pub struct Checkbox {
    pub label: String,
    pub checked: bool,
    pub is_focused: bool,
}

impl Checkbox {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            checked: false,
            is_focused: false,
        }
    }

    pub fn checked(mut self) -> Self {
        self.checked = true;
        self
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    pub fn toggle(&mut self) {
        self.checked = !self.checked;
    }

    /// 渲染复选框
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let style = if self.is_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };

        let checkbox_char = if self.checked { "☑" } else { "☐" };

        let text = Line::from(vec![
            Span::styled(checkbox_char, style),
            Span::raw(" "),
            Span::styled(&self.label, style),
        ]);

        let paragraph = Paragraph::new(text);
        f.render_widget(paragraph, area);
    }

    pub fn is_checked(&self) -> bool {
        self.checked
    }
}
