use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// 单行文本输入组件
#[derive(Clone)]
pub struct TextInput {
    pub label: String,
    pub value: String,
    pub placeholder: String,
    pub is_password: bool,
    pub is_focused: bool,
    pub cursor_position: usize,
    pub error: Option<String>,
    pub required: bool,
}

impl TextInput {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: String::new(),
            placeholder: String::new(),
            is_password: false,
            is_focused: false,
            cursor_position: 0,
            error: None,
            required: false,
        }
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self.cursor_position = self.value.len();
        self
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn password(mut self) -> Self {
        self.is_password = true;
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    pub fn set_error(&mut self, error: Option<String>) {
        self.error = error;
    }

    /// 处理键盘输入
    pub fn handle_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char(c) => {
                self.value.insert(self.cursor_position, c);
                self.cursor_position += 1;
                self.error = None; // 清除错误
                true
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.value.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                    self.error = None;
                }
                true
            }
            KeyCode::Delete => {
                if self.cursor_position < self.value.len() {
                    self.value.remove(self.cursor_position);
                    self.error = None;
                }
                true
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
                true
            }
            KeyCode::Right => {
                if self.cursor_position < self.value.len() {
                    self.cursor_position += 1;
                }
                true
            }
            KeyCode::Home => {
                self.cursor_position = 0;
                true
            }
            KeyCode::End => {
                self.cursor_position = self.value.len();
                true
            }
            _ => false,
        }
    }

    /// 渲染输入框
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let border_style = if self.is_focused {
            Style::default().fg(Color::Cyan)
        } else if self.error.is_some() {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Gray)
        };

        let title = if self.required {
            format!("{} *", self.label)
        } else {
            self.label.clone()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(title);

        let inner = block.inner(area);
        f.render_widget(block, area);

        // 显示内容
        let display_value = if self.is_password {
            "*".repeat(self.value.len())
        } else {
            self.value.clone()
        };

        let content = if display_value.is_empty() && !self.is_focused {
            Span::styled(&self.placeholder, Style::default().fg(Color::DarkGray))
        } else {
            Span::raw(&display_value)
        };

        let text = Paragraph::new(Line::from(vec![content]));
        f.render_widget(text, inner);

        // 显示光标（仅在聚焦时）
        if self.is_focused {
            let cursor_x = inner.x + self.cursor_position as u16;
            let cursor_y = inner.y;
            if cursor_x < inner.x + inner.width {
                f.set_cursor_position((cursor_x, cursor_y));
            }
        }

        // 显示错误信息
        if let Some(error) = &self.error {
            if area.height > 3 {
                let error_area = Rect {
                    x: area.x,
                    y: area.y + 3,
                    width: area.width,
                    height: 1,
                };
                let error_text = Paragraph::new(Line::from(vec![Span::styled(
                    format!("  ⚠ {}", error),
                    Style::default().fg(Color::Red),
                )]));
                f.render_widget(error_text, error_area);
            }
        }
    }

    /// 验证输入
    pub fn validate(&self) -> Result<(), String> {
        if self.required && self.value.trim().is_empty() {
            return Err(format!("{} 不能为空", self.label));
        }
        Ok(())
    }

    /// 获取值
    pub fn get_value(&self) -> &str {
        &self.value
    }
}
