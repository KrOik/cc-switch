use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use serde_json::Value;

use super::forms::{FormContainer, FormValue, TextArea, TextInput};

/// Provider 表单模式
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderFormMode {
    Add,
    Edit(String), // provider_id
}

/// Provider 表单视图
pub struct ProviderFormView {
    form: FormContainer,
    mode: ProviderFormMode,
    error_message: Option<String>,
}

impl ProviderFormView {
    /// 创建新增 Provider 表单
    pub fn new_add() -> Self {
        let mut form = FormContainer::new("添加 Provider");

        // 名称（必填）
        form = form.add_text_input(
            TextInput::new("名称")
                .with_placeholder("例如: OpenAI")
                .required(),
        );

        // 配置 JSON（必填）
        form = form.add_text_area(
            TextArea::new("配置 (JSON)")
                .with_value(Self::default_settings_json())
                .required(),
        );

        // 网站 URL（可选）
        form = form.add_text_input(
            TextInput::new("网站 URL")
                .with_placeholder("例如: https://openai.com"),
        );

        // 备注（可选）
        form = form.add_text_area(TextArea::new("备注"));

        form.activate();

        Self {
            form,
            mode: ProviderFormMode::Add,
            error_message: None,
        }
    }

    /// 创建编辑 Provider 表单
    pub fn new_edit(
        provider_id: String,
        name: String,
        settings_config: &Value,
        website_url: Option<String>,
        notes: Option<String>,
    ) -> Self {
        let mut form = FormContainer::new("编辑 Provider");

        // 名称（必填）
        form = form.add_text_input(TextInput::new("名称").with_value(name).required());

        // 配置 JSON（必填）
        let config_json = serde_json::to_string_pretty(settings_config)
            .unwrap_or_else(|_| "{}".to_string());
        form = form.add_text_area(
            TextArea::new("配置 (JSON)")
                .with_value(config_json)
                .required(),
        );

        // 网站 URL（可选）
        form = form.add_text_input(
            TextInput::new("网站 URL")
                .with_value(website_url.unwrap_or_default()),
        );

        // 备注（可选）
        form = form.add_text_area(
            TextArea::new("备注")
                .with_value(notes.unwrap_or_default()),
        );

        form.activate();

        Self {
            form,
            mode: ProviderFormMode::Edit(provider_id),
            error_message: None,
        }
    }

    /// 默认配置 JSON 模板
    fn default_settings_json() -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "baseURL": "https://api.openai.com/v1",
            "apiKey": "",
            "models": []
        }))
        .unwrap_or_else(|_| "{}".to_string())
    }

    /// 处理键盘输入
    pub fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) -> FormAction {
        match key {
            KeyCode::Esc => FormAction::Cancel,
            KeyCode::Enter if modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+Enter 提交表单
                self.submit()
            }
            _ => {
                self.form.handle_key(key, modifiers);
                FormAction::None
            }
        }
    }

    /// 提交表单
    fn submit(&mut self) -> FormAction {
        // 验证表单
        if let Err(errors) = self.form.validate() {
            self.error_message = Some(errors.join("; "));
            return FormAction::None;
        }

        // 获取表单值
        let values = self.form.get_values();
        if values.len() < 4 {
            self.error_message = Some("表单数据不完整".to_string());
            return FormAction::None;
        }

        let name = values[0].as_text().unwrap_or("").to_string();
        let config_json = values[1].as_text().unwrap_or("{}");
        let website_url = values[2].as_text().unwrap_or("");
        let notes = values[3].as_text().unwrap_or("");

        // 解析 JSON 配置
        let settings_config = match serde_json::from_str::<Value>(config_json) {
            Ok(v) => v,
            Err(e) => {
                self.error_message = Some(format!("JSON 格式错误: {}", e));
                return FormAction::None;
            }
        };

        // 返回提交数据
        match &self.mode {
            ProviderFormMode::Add => FormAction::Submit(ProviderFormData {
                id: None,
                name,
                settings_config,
                website_url: if website_url.is_empty() {
                    None
                } else {
                    Some(website_url.to_string())
                },
                notes: if notes.is_empty() {
                    None
                } else {
                    Some(notes.to_string())
                },
            }),
            ProviderFormMode::Edit(id) => FormAction::Submit(ProviderFormData {
                id: Some(id.clone()),
                name,
                settings_config,
                website_url: if website_url.is_empty() {
                    None
                } else {
                    Some(website_url.to_string())
                },
                notes: if notes.is_empty() {
                    None
                } else {
                    Some(notes.to_string())
                },
            }),
        }
    }

    /// 渲染表单
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // 标题
                Constraint::Min(10),    // 表单内容
                Constraint::Length(3),  // 错误信息/提示
            ])
            .split(area);

        // 渲染标题
        let title = match &self.mode {
            ProviderFormMode::Add => "添加 Provider",
            ProviderFormMode::Edit(_) => "编辑 Provider",
        };
        let title_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(title);
        f.render_widget(title_block, chunks[0]);

        // 渲染表单
        self.form.render(f, chunks[1]);

        // 渲染错误信息或提示
        let bottom_text = if let Some(error) = &self.error_message {
            vec![
                Span::styled("错误: ", Style::default().fg(Color::Red)),
                Span::styled(error, Style::default().fg(Color::Red)),
            ]
        } else {
            vec![
                Span::styled("Ctrl+Enter", Style::default().fg(Color::Green)),
                Span::raw(" 提交  "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" 取消"),
            ]
        };

        let bottom_paragraph = Paragraph::new(Line::from(bottom_text))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(bottom_paragraph, chunks[2]);
    }
}

/// 表单操作结果
pub enum FormAction {
    None,
    Submit(ProviderFormData),
    Cancel,
}

/// Provider 表单数据
#[derive(Debug, Clone)]
pub struct ProviderFormData {
    pub id: Option<String>,
    pub name: String,
    pub settings_config: Value,
    pub website_url: Option<String>,
    pub notes: Option<String>,
}
