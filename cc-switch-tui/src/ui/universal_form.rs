use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::forms::{Checkbox, FormContainer, FormValue, TextInput};

/// Universal Provider 表单模式
#[derive(Debug, Clone, PartialEq)]
pub enum UniversalFormMode {
    Add,
    Edit(String), // provider_id
}

/// Universal Provider 表单视图
pub struct UniversalFormView {
    form: FormContainer,
    mode: UniversalFormMode,
    error_message: Option<String>,
}

impl UniversalFormView {
    /// 创建新增 Universal Provider 表单
    pub fn new_add() -> Self {
        let mut form = FormContainer::new("添加统一供应商");

        // 名称（必填）
        form = form.add_text_input(
            TextInput::new("名称")
                .with_placeholder("例如: OpenAI")
                .required(),
        );

        // 供应商类型（必填）
        form = form.add_text_input(
            TextInput::new("供应商类型")
                .with_placeholder("例如: newapi, custom")
                .required(),
        );

        // Base URL（必填）
        form = form.add_text_input(
            TextInput::new("Base URL")
                .with_placeholder("https://api.openai.com/v1")
                .required(),
        );

        // API Key（必填）
        form = form.add_text_input(
            TextInput::new("API Key")
                .with_placeholder("sk-...")
                .password()
                .required(),
        );

        // 网站 URL（可选）
        form = form.add_text_input(
            TextInput::new("网站 URL")
                .with_placeholder("https://..."),
        );

        // 备注（可选）
        form = form.add_text_input(
            TextInput::new("备注")
                .with_placeholder("供应商说明"),
        );

        // 应用启用状态
        form = form.add_checkbox(Checkbox::new("启用 Claude"));
        form = form.add_checkbox(Checkbox::new("启用 Codex"));
        form = form.add_checkbox(Checkbox::new("启用 Gemini"));

        form.activate();

        Self {
            form,
            mode: UniversalFormMode::Add,
            error_message: None,
        }
    }

    /// 创建编辑 Universal Provider 表单
    pub fn new_edit(
        provider_id: String,
        name: String,
        provider_type: String,
        base_url: String,
        api_key: String,
        website_url: Option<String>,
        notes: Option<String>,
        claude_enabled: bool,
        codex_enabled: bool,
        gemini_enabled: bool,
    ) -> Self {
        let mut form = FormContainer::new("编辑统一供应商");

        // 名称（必填）
        form = form.add_text_input(TextInput::new("名称").with_value(name).required());

        // 供应商类型（必填）
        form = form.add_text_input(
            TextInput::new("供应商类型")
                .with_value(provider_type)
                .required(),
        );

        // Base URL（必填）
        form = form.add_text_input(
            TextInput::new("Base URL")
                .with_value(base_url)
                .required(),
        );

        // API Key（必填）
        form = form.add_text_input(
            TextInput::new("API Key")
                .with_value(api_key)
                .password()
                .required(),
        );

        // 网站 URL（可选）
        form = form.add_text_input(
            TextInput::new("网站 URL")
                .with_value(website_url.unwrap_or_default()),
        );

        // 备注（可选）
        form = form.add_text_input(
            TextInput::new("备注")
                .with_value(notes.unwrap_or_default()),
        );

        // 应用启用状态
        let mut checkbox_claude = Checkbox::new("启用 Claude");
        if claude_enabled {
            checkbox_claude = checkbox_claude.checked();
        }
        form = form.add_checkbox(checkbox_claude);

        let mut checkbox_codex = Checkbox::new("启用 Codex");
        if codex_enabled {
            checkbox_codex = checkbox_codex.checked();
        }
        form = form.add_checkbox(checkbox_codex);

        let mut checkbox_gemini = Checkbox::new("启用 Gemini");
        if gemini_enabled {
            checkbox_gemini = checkbox_gemini.checked();
        }
        form = form.add_checkbox(checkbox_gemini);

        form.activate();

        Self {
            form,
            mode: UniversalFormMode::Edit(provider_id),
            error_message: None,
        }
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
        if values.len() < 9 {
            self.error_message = Some("表单数据不完整".to_string());
            return FormAction::None;
        }

        let name = values[0].as_text().unwrap_or("").to_string();
        let provider_type = values[1].as_text().unwrap_or("").to_string();
        let base_url = values[2].as_text().unwrap_or("").to_string();
        let api_key = values[3].as_text().unwrap_or("").to_string();
        let website_url = values[4].as_text().unwrap_or("");
        let notes = values[5].as_text().unwrap_or("");
        let claude_enabled = values[6].as_bool().unwrap_or(false);
        let codex_enabled = values[7].as_bool().unwrap_or(false);
        let gemini_enabled = values[8].as_bool().unwrap_or(false);

        // 返回提交数据
        match &self.mode {
            UniversalFormMode::Add => FormAction::Submit(UniversalFormData {
                id: None,
                name,
                provider_type,
                base_url,
                api_key,
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
                claude_enabled,
                codex_enabled,
                gemini_enabled,
            }),
            UniversalFormMode::Edit(id) => FormAction::Submit(UniversalFormData {
                id: Some(id.clone()),
                name,
                provider_type,
                base_url,
                api_key,
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
                claude_enabled,
                codex_enabled,
                gemini_enabled,
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
            UniversalFormMode::Add => "添加统一供应商",
            UniversalFormMode::Edit(_) => "编辑统一供应商",
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
    Submit(UniversalFormData),
    Cancel,
}

/// Universal Provider 表单数据
#[derive(Debug, Clone)]
pub struct UniversalFormData {
    pub id: Option<String>,
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub api_key: String,
    pub website_url: Option<String>,
    pub notes: Option<String>,
    pub claude_enabled: bool,
    pub codex_enabled: bool,
    pub gemini_enabled: bool,
}
