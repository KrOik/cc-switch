use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::{Checkbox, Select, TextArea, TextInput};

/// 表单字段类型
#[derive(Clone)]
pub enum FormField {
    TextInput(TextInput),
    TextArea(TextArea),
    Checkbox(Checkbox),
    Select(Select),
}

/// 表单值类型
#[derive(Debug, Clone)]
pub enum FormValue {
    Text(String),
    Bool(bool),
    Index(usize),
}

impl FormValue {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            FormValue::Text(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            FormValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_index(&self) -> Option<usize> {
        match self {
            FormValue::Index(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_select_index(&self) -> Option<usize> {
        match self {
            FormValue::Index(i) => Some(*i),
            _ => None,
        }
    }
}

/// 表单容器
pub struct FormContainer {
    pub fields: Vec<FormField>,
    pub focused_index: usize,
    pub is_active: bool,
    pub title: String,
}

impl FormContainer {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            fields: Vec::new(),
            focused_index: 0,
            is_active: false,
            title: title.into(),
        }
    }

    /// 添加文本输入字段
    pub fn add_text_input(mut self, input: TextInput) -> Self {
        self.fields.push(FormField::TextInput(input));
        self
    }

    /// 添加文本区域字段
    pub fn add_text_area(mut self, area: TextArea) -> Self {
        self.fields.push(FormField::TextArea(area));
        self
    }

    /// 添加复选框字段
    pub fn add_checkbox(mut self, checkbox: Checkbox) -> Self {
        self.fields.push(FormField::Checkbox(checkbox));
        self
    }

    /// 添加选择器字段
    pub fn add_select(mut self, select: Select) -> Self {
        self.fields.push(FormField::Select(select));
        self
    }

    /// 激活表单
    pub fn activate(&mut self) {
        self.is_active = true;
        self.focused_index = 0;
        self.update_focus();
    }

    /// 停用表单
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.clear_focus();
    }

    /// 更新焦点状态
    fn update_focus(&mut self) {
        for (i, field) in self.fields.iter_mut().enumerate() {
            let focused = i == self.focused_index;
            match field {
                FormField::TextInput(input) => input.set_focused(focused),
                FormField::TextArea(area) => area.set_focused(focused),
                FormField::Checkbox(checkbox) => checkbox.set_focused(focused),
                FormField::Select(select) => select.set_focused(focused),
            }
        }
    }

    /// 清除所有焦点
    fn clear_focus(&mut self) {
        for field in &mut self.fields {
            match field {
                FormField::TextInput(input) => input.set_focused(false),
                FormField::TextArea(area) => area.set_focused(false),
                FormField::Checkbox(checkbox) => checkbox.set_focused(false),
                FormField::Select(select) => select.set_focused(false),
            }
        }
    }

    /// 移动到下一个字段
    pub fn focus_next(&mut self) {
        if !self.fields.is_empty() {
            self.focused_index = (self.focused_index + 1) % self.fields.len();
            self.update_focus();
        }
    }

    /// 移动到上一个字段
    pub fn focus_prev(&mut self) {
        if !self.fields.is_empty() {
            if self.focused_index == 0 {
                self.focused_index = self.fields.len() - 1;
            } else {
                self.focused_index -= 1;
            }
            self.update_focus();
        }
    }

    /// 处理键盘输入
    pub fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) -> bool {
        if !self.is_active || self.fields.is_empty() {
            return false;
        }

        // Tab 键切换字段
        if key == KeyCode::Tab {
            if modifiers.contains(KeyModifiers::SHIFT) {
                self.focus_prev();
            } else {
                self.focus_next();
            }
            return true;
        }

        // 将按键传递给当前聚焦的字段
        if let Some(field) = self.fields.get_mut(self.focused_index) {
            match field {
                FormField::TextInput(input) => input.handle_key(key),
                FormField::TextArea(area) => area.handle_key(key),
                FormField::Checkbox(checkbox) => {
                    if key == KeyCode::Enter || key == KeyCode::Char(' ') {
                        checkbox.toggle();
                        true
                    } else {
                        false
                    }
                }
                FormField::Select(select) => select.handle_key(key),
            }
        } else {
            false
        }
    }

    /// 验证所有字段
    pub fn validate(&mut self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for (i, field) in self.fields.iter_mut().enumerate() {
            let result = match field {
                FormField::TextInput(input) => {
                    let result = input.validate();
                    if let Err(e) = &result {
                        input.set_error(Some(e.clone()));
                    }
                    result
                }
                FormField::TextArea(area) => {
                    let result = area.validate();
                    if let Err(e) = &result {
                        area.set_error(Some(e.clone()));
                    }
                    result
                }
                FormField::Checkbox(_) => Ok(()),
                FormField::Select(select) => select.validate(),
            };

            if let Err(e) = result {
                errors.push(format!("字段 {}: {}", i + 1, e));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// 获取所有字段的值
    pub fn get_values(&self) -> Vec<FormValue> {
        self.fields
            .iter()
            .map(|field| match field {
                FormField::TextInput(input) => FormValue::Text(input.get_value().to_string()),
                FormField::TextArea(area) => FormValue::Text(area.get_value()),
                FormField::Checkbox(checkbox) => FormValue::Bool(checkbox.is_checked()),
                FormField::Select(select) => FormValue::Index(select.get_selected_index()),
            })
            .collect()
    }

    /// 渲染表单
    pub fn render(&self, f: &mut Frame, area: Rect) {
        if self.fields.is_empty() {
            return;
        }

        // 计算每个字段需要的高度
        let mut constraints = Vec::new();
        for field in &self.fields {
            let height = match field {
                FormField::TextInput(_) => 3,
                FormField::TextArea(_) => 8,
                FormField::Checkbox(_) => 1,
                FormField::Select(select) => {
                    if select.is_expanded {
                        (select.options.len() + 2).min(10) as u16
                    } else {
                        3
                    }
                }
            };
            constraints.push(Constraint::Length(height));
        }

        // 添加底部提示区域
        constraints.push(Constraint::Length(2));
        constraints.push(Constraint::Min(0));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        // 渲染每个字段
        for (i, field) in self.fields.iter().enumerate() {
            if i < chunks.len() {
                match field {
                    FormField::TextInput(input) => input.render(f, chunks[i]),
                    FormField::TextArea(area) => area.render(f, chunks[i]),
                    FormField::Checkbox(checkbox) => checkbox.render(f, chunks[i]),
                    FormField::Select(select) => select.render(f, chunks[i]),
                }
            }
        }

        // 渲染底部提示
        if let Some(hint_area) = chunks.get(self.fields.len()) {
            let hints = vec![
                Span::styled("Tab", Style::default().fg(Color::Cyan)),
                Span::raw(" 切换字段  "),
                Span::styled("Enter", Style::default().fg(Color::Cyan)),
                Span::raw(" 提交  "),
                Span::styled("Esc", Style::default().fg(Color::Cyan)),
                Span::raw(" 取消"),
            ];
            let hint_text = Paragraph::new(Line::from(hints));
            f.render_widget(hint_text, *hint_area);
        }
    }
}
