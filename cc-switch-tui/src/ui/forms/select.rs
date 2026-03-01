use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

/// 选择器组件
#[derive(Clone)]
pub struct Select {
    pub label: String,
    pub options: Vec<String>,
    pub selected_index: usize,
    pub is_focused: bool,
    pub is_expanded: bool,
    pub error: Option<String>,
    pub required: bool,
}

impl Select {
    pub fn new(label: impl Into<String>, options: Vec<String>) -> Self {
        Self {
            label: label.into(),
            options,
            selected_index: 0,
            is_focused: false,
            is_expanded: false,
            error: None,
            required: false,
        }
    }

    pub fn with_selected(mut self, index: usize) -> Self {
        if index < self.options.len() {
            self.selected_index = index;
        }
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
        if !focused {
            self.is_expanded = false;
        }
    }

    pub fn set_error(&mut self, error: Option<String>) {
        self.error = error;
    }

    /// 处理键盘输入
    pub fn handle_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.is_expanded = !self.is_expanded;
                true
            }
            KeyCode::Up => {
                if self.is_expanded && self.selected_index > 0 {
                    self.selected_index -= 1;
                    self.error = None;
                }
                true
            }
            KeyCode::Down => {
                if self.is_expanded && self.selected_index < self.options.len() - 1 {
                    self.selected_index += 1;
                    self.error = None;
                }
                true
            }
            KeyCode::Esc => {
                if self.is_expanded {
                    self.is_expanded = false;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// 渲染选择器
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

        if self.is_expanded {
            // 显示选项列表
            let items: Vec<ListItem> = self.options
                .iter()
                .enumerate()
                .map(|(i, opt)| {
                    let prefix = if i == self.selected_index { "▶ " } else { "  " };
                    ListItem::new(format!("{}{}", prefix, opt))
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(border_style)
                        .title(title),
                )
                .highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                );

            let mut state = ListState::default();
            state.select(Some(self.selected_index));
            f.render_stateful_widget(list, area, &mut state);
        } else {
            // 显示当前选中项
            let selected_text = self.options.get(self.selected_index)
                .map(|s| s.as_str())
                .unwrap_or("");

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(title);

            let inner = block.inner(area);
            f.render_widget(block, area);

            let text = Line::from(format!("{} ▼", selected_text));
            let paragraph = ratatui::widgets::Paragraph::new(text);
            f.render_widget(paragraph, inner);
        }
    }

    /// 验证输入
    pub fn validate(&self) -> Result<(), String> {
        if self.required && self.options.is_empty() {
            return Err(format!("{} 没有可选项", self.label));
        }
        Ok(())
    }

    /// 获取选中的值
    pub fn get_selected(&self) -> Option<&str> {
        self.options.get(self.selected_index).map(|s| s.as_str())
    }

    /// 获取选中的索引
    pub fn get_selected_index(&self) -> usize {
        self.selected_index
    }
}
