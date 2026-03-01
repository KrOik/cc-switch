use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// 多行文本编辑器组件
#[derive(Clone)]
pub struct TextArea {
    pub label: String,
    pub lines: Vec<String>,
    pub is_focused: bool,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
    pub error: Option<String>,
    pub required: bool,
}

impl TextArea {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            lines: vec![String::new()],
            is_focused: false,
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            error: None,
            required: false,
        }
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        let text = value.into();
        self.lines = if text.is_empty() {
            vec![String::new()]
        } else {
            text.lines().map(|s| s.to_string()).collect()
        };
        self.cursor_line = self.lines.len().saturating_sub(1);
        self.cursor_col = self.lines.last().map(|l| l.len()).unwrap_or(0);
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
                if let Some(line) = self.lines.get_mut(self.cursor_line) {
                    line.insert(self.cursor_col, c);
                    self.cursor_col += 1;
                    self.error = None;
                }
                true
            }
            KeyCode::Enter => {
                if let Some(line) = self.lines.get_mut(self.cursor_line) {
                    let rest = line.split_off(self.cursor_col);
                    self.lines.insert(self.cursor_line + 1, rest);
                    self.cursor_line += 1;
                    self.cursor_col = 0;
                    self.error = None;
                }
                true
            }
            KeyCode::Backspace => {
                if self.cursor_col > 0 {
                    if let Some(line) = self.lines.get_mut(self.cursor_line) {
                        line.remove(self.cursor_col - 1);
                        self.cursor_col -= 1;
                        self.error = None;
                    }
                } else if self.cursor_line > 0 {
                    // 合并到上一行
                    let current_line = self.lines.remove(self.cursor_line);
                    self.cursor_line -= 1;
                    if let Some(prev_line) = self.lines.get_mut(self.cursor_line) {
                        self.cursor_col = prev_line.len();
                        prev_line.push_str(&current_line);
                        self.error = None;
                    }
                }
                true
            }
            KeyCode::Delete => {
                let line_len = self.lines.get(self.cursor_line).map(|l| l.len()).unwrap_or(0);
                if self.cursor_col < line_len {
                    if let Some(line) = self.lines.get_mut(self.cursor_line) {
                        line.remove(self.cursor_col);
                        self.error = None;
                    }
                } else if self.cursor_line < self.lines.len() - 1 {
                    // 合并下一行
                    let next_line = self.lines.remove(self.cursor_line + 1);
                    if let Some(line) = self.lines.get_mut(self.cursor_line) {
                        line.push_str(&next_line);
                        self.error = None;
                    }
                }
                true
            }
            KeyCode::Left => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                } else if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.cursor_col = self.lines.get(self.cursor_line).map(|l| l.len()).unwrap_or(0);
                }
                true
            }
            KeyCode::Right => {
                if let Some(line) = self.lines.get(self.cursor_line) {
                    if self.cursor_col < line.len() {
                        self.cursor_col += 1;
                    } else if self.cursor_line < self.lines.len() - 1 {
                        self.cursor_line += 1;
                        self.cursor_col = 0;
                    }
                }
                true
            }
            KeyCode::Up => {
                if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    if let Some(line) = self.lines.get(self.cursor_line) {
                        self.cursor_col = self.cursor_col.min(line.len());
                    }
                }
                true
            }
            KeyCode::Down => {
                if self.cursor_line < self.lines.len() - 1 {
                    self.cursor_line += 1;
                    if let Some(line) = self.lines.get(self.cursor_line) {
                        self.cursor_col = self.cursor_col.min(line.len());
                    }
                }
                true
            }
            KeyCode::Home => {
                self.cursor_col = 0;
                true
            }
            KeyCode::End => {
                if let Some(line) = self.lines.get(self.cursor_line) {
                    self.cursor_col = line.len();
                }
                true
            }
            _ => false,
        }
    }

    /// 渲染文本区域
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

        // 计算可见行数
        let visible_lines = inner.height as usize;

        // 调整滚动偏移
        let mut scroll = self.scroll_offset;
        if self.cursor_line < scroll {
            scroll = self.cursor_line;
        } else if self.cursor_line >= scroll + visible_lines {
            scroll = self.cursor_line - visible_lines + 1;
        }

        // 渲染可见行
        let visible_text: Vec<Line> = self.lines
            .iter()
            .skip(scroll)
            .take(visible_lines)
            .map(|line| Line::from(line.clone()))
            .collect();

        let text = Paragraph::new(visible_text).wrap(Wrap { trim: false });
        f.render_widget(text, inner);

        // 显示光标
        if self.is_focused && self.cursor_line >= scroll && self.cursor_line < scroll + visible_lines {
            let cursor_y = inner.y + (self.cursor_line - scroll) as u16;
            let cursor_x = inner.x + self.cursor_col as u16;
            if cursor_x < inner.x + inner.width && cursor_y < inner.y + inner.height {
                f.set_cursor_position((cursor_x, cursor_y));
            }
        }
    }

    /// 验证输入
    pub fn validate(&self) -> Result<(), String> {
        if self.required && self.lines.iter().all(|l| l.trim().is_empty()) {
            return Err(format!("{} 不能为空", self.label));
        }
        Ok(())
    }

    /// 获取值
    pub fn get_value(&self) -> String {
        self.lines.join("\n")
    }
}
