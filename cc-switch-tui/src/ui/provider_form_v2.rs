use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use serde_json::Value;

use super::forms::{Button, FormContainer, Select, TextArea, TextInput};
use super::provider_config::{ParsedProviderConfig, ProviderType};

/// Provider 表单模式
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderFormMode {
    Add,
    Edit(String), // provider_id
}

/// Provider 表单视图（改进版）
pub struct ProviderFormViewV2 {
    form: FormContainer,
    mode: ProviderFormMode,
    provider_type: ProviderType,
    error_message: Option<String>,
    show_preview: bool,
}

impl ProviderFormViewV2 {
    /// 创建新增 Provider 表单
    pub fn new_add() -> Self {
        let mut form = FormContainer::new("添加 Provider");

        // Provider 类型选择
        form = form.add_select(
            Select::new(
                "Provider 类型",
                vec![
                    "通用 (Generic)".to_string(),
                    "Claude".to_string(),
                    "Codex".to_string(),
                    "OpenCode".to_string(),
                ],
            )
            .required(),
        );

        // 名称（必填）
        form = form.add_text_input(
            TextInput::new("名称")
                .with_placeholder("例如: OpenAI")
                .required(),
        );

        // 请求地址（必填）
        form = form.add_text_input(
            TextInput::new("请求地址 (Base URL)")
                .with_placeholder("例如: https://api.openai.com/v1")
                .required(),
        );

        // API Key（必填）
        form = form.add_text_input(
            TextInput::new("API Key")
                .with_placeholder("sk-...")
                .required(),
        );

        // 模型列表（可选）
        form = form.add_text_area(
            TextArea::new("模型列表 (每行一个)")
                .with_value(""),
        );

        // API 格式（可选）
        form = form.add_select(Select::new(
            "API 格式",
            vec![
                "默认".to_string(),
                "anthropic".to_string(),
                "openai_chat".to_string(),
            ],
        ));

        // 网站 URL（可选）
        form = form.add_text_input(
            TextInput::new("网站 URL").with_placeholder("例如: https://openai.com"),
        );

        // 备注（可选）
        form = form.add_text_area(TextArea::new("备注"));

        // 添加按钮
        form = form.add_button(Button::new("保存").primary());
        form = form.add_button(Button::new("取消"));

        form.activate();

        Self {
            form,
            mode: ProviderFormMode::Add,
            provider_type: ProviderType::Generic,
            error_message: None,
            show_preview: false,
        }
    }

    /// 创建编辑 Provider 表单
    pub fn new_edit(
        provider_id: String,
        name: String,
        settings_config: &Value,
        meta: Option<&Value>,
        website_url: Option<String>,
        notes: Option<String>,
    ) -> Self {
        // 解析配置
        let parsed = ParsedProviderConfig::from_settings_config(settings_config, meta);

        let mut form = FormContainer::new("编辑 Provider");

        // Provider 类型（只读显示）
        let type_options = vec![
            "通用 (Generic)".to_string(),
            "Claude".to_string(),
            "Codex".to_string(),
            "OpenCode".to_string(),
        ];
        let type_index = match parsed.provider_type {
            ProviderType::Generic => 0,
            ProviderType::Claude => 1,
            ProviderType::Codex => 2,
            ProviderType::OpenCode => 3,
        };
        form = form.add_select(
            Select::new("Provider 类型", type_options)
                .with_selected(type_index)
                .required(),
        );

        // 名称（必填）
        form = form.add_text_input(TextInput::new("名称").with_value(name).required());

        // 请求地址（必填）
        form = form.add_text_input(
            TextInput::new("请求地址 (Base URL)")
                .with_value(parsed.base_url)
                .required(),
        );

        // API Key（必填）
        form = form.add_text_input(
            TextInput::new("API Key")
                .with_value(parsed.api_key)
                .required(),
        );

        // 模型列表（可选）
        let models_text = parsed.models.join("\n");
        form = form.add_text_area(
            TextArea::new("模型列表 (每行一个)")
                .with_value(models_text),
        );

        // API 格式（可选）
        let api_format_options = vec![
            "默认".to_string(),
            "anthropic".to_string(),
            "openai_chat".to_string(),
        ];
        let api_format_index = match parsed.api_format.as_deref() {
            Some("anthropic") => 1,
            Some("openai_chat") => 2,
            _ => 0,
        };
        form = form.add_select(
            Select::new("API 格式", api_format_options).with_selected(api_format_index),
        );

        // 网站 URL（可选）
        form = form.add_text_input(
            TextInput::new("网站 URL").with_value(website_url.unwrap_or_default()),
        );

        // 备注（可选）
        form = form.add_text_area(TextArea::new("备注").with_value(notes.unwrap_or_default()));

        // 添加按钮
        form = form.add_button(Button::new("保存").primary());
        form = form.add_button(Button::new("取消"));

        form.activate();

        Self {
            form,
            mode: ProviderFormMode::Edit(provider_id),
            provider_type: parsed.provider_type,
            error_message: None,
            show_preview: false,
        }
    }

    /// 处理键盘输入
    pub fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) -> FormAction {
        // Ctrl+P 切换预览
        if key == KeyCode::Char('p') && modifiers.contains(KeyModifiers::CONTROL) {
            self.show_preview = !self.show_preview;
            return FormAction::None;
        }

        match key {
            KeyCode::Esc => FormAction::Cancel,
            _ => {
                self.form.handle_key(key, modifiers);

                // 检查是否点击了按钮
                if key == KeyCode::Enter || key == KeyCode::Char(' ') {
                    if let Some(button_label) = self.form.get_focused_button_label() {
                        match button_label.as_str() {
                            "保存" => return self.submit(),
                            "取消" => return FormAction::Cancel,
                            _ => {}
                        }
                    }
                }

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
        if values.len() < 8 {
            self.error_message = Some("表单数据不完整".to_string());
            return FormAction::None;
        }

        // 解析 Provider 类型
        let type_index = values[0].as_select_index().unwrap_or(0);
        let provider_type = match type_index {
            1 => ProviderType::Claude,
            2 => ProviderType::Codex,
            3 => ProviderType::OpenCode,
            _ => ProviderType::Generic,
        };

        let name = values[1].as_text().unwrap_or("").to_string();
        let base_url = values[2].as_text().unwrap_or("").to_string();
        let api_key = values[3].as_text().unwrap_or("").to_string();
        let models_text = values[4].as_text().unwrap_or("");
        let api_format_index = values[5].as_select_index().unwrap_or(0);
        let website_url = values[6].as_text().unwrap_or("");
        let notes = values[7].as_text().unwrap_or("");

        // 解析模型列表
        let models: Vec<String> = models_text
            .lines()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        // 解析 API 格式
        let api_format = match api_format_index {
            1 => Some("anthropic".to_string()),
            2 => Some("openai_chat".to_string()),
            _ => None,
        };

        // 构建配置
        let parsed_config = ParsedProviderConfig {
            provider_type,
            base_url,
            api_key,
            models,
            api_format: api_format.clone(),
            extra_fields: std::collections::HashMap::new(),
        };

        // 转换为 settings_config
        let settings_config = parsed_config.to_settings_config();

        // 构建 meta（如果有 api_format）
        let meta = if api_format.is_some() {
            Some(serde_json::json!({
                "apiFormat": api_format
            }))
        } else {
            None
        };

        // 返回提交数据
        match &self.mode {
            ProviderFormMode::Add => FormAction::Submit(ProviderFormData {
                id: None,
                name,
                settings_config,
                meta,
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
                meta,
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

    /// 生成预览 JSON
    fn generate_preview_json(&self) -> String {
        // 获取表单值
        let values = self.form.get_values();
        if values.len() < 8 {
            return "表单数据不完整".to_string();
        }

        // 解析 Provider 类型
        let type_index = values[0].as_select_index().unwrap_or(0);
        let provider_type = match type_index {
            1 => ProviderType::Claude,
            2 => ProviderType::Codex,
            3 => ProviderType::OpenCode,
            _ => ProviderType::Generic,
        };

        let name = values[1].as_text().unwrap_or("");
        let base_url = values[2].as_text().unwrap_or("");
        let api_key = values[3].as_text().unwrap_or("");
        let models_text = values[4].as_text().unwrap_or("");
        let api_format_index = values[5].as_select_index().unwrap_or(0);
        let website_url = values[6].as_text().unwrap_or("");
        let notes = values[7].as_text().unwrap_or("");

        // 解析模型列表
        let models: Vec<String> = models_text
            .lines()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        // 解析 API 格式
        let api_format = match api_format_index {
            1 => Some("anthropic".to_string()),
            2 => Some("openai_chat".to_string()),
            _ => None,
        };

        // 构建配置
        let parsed_config = ParsedProviderConfig {
            provider_type: provider_type.clone(),
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            models,
            api_format: api_format.clone(),
            extra_fields: std::collections::HashMap::new(),
        };

        // 转换为 settings_config
        let settings_config = parsed_config.to_settings_config();

        // 构建 meta（如果有 api_format）
        let meta = if api_format.is_some() {
            Some(serde_json::json!({
                "apiFormat": api_format
            }))
        } else {
            None
        };

        // 对于 Claude/Codex/Gemini 等应用，只显示 settings_config 的内容
        // 因为这才是会被写入 live 配置文件的内容
        let preview_data = match provider_type {
            ProviderType::Claude | ProviderType::Generic => {
                // Claude 和通用类型：只显示 settings_config
                settings_config
            }
            ProviderType::Codex => {
                // Codex：显示 auth 和 config 结构
                settings_config
            }
            ProviderType::OpenCode => {
                // OpenCode：显示 provider 配置片段
                settings_config
            }
        };

        // 格式化 JSON
        match serde_json::to_string_pretty(&preview_data) {
            Ok(json) => json,
            Err(e) => format!("JSON 序列化失败: {}", e),
        }
    }

    /// 渲染表单
    pub fn render(&self, f: &mut Frame, area: Rect) {
        if self.show_preview {
            // 分屏模式：左侧表单，右侧预览
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),  // 左侧表单
                    Constraint::Percentage(50),  // 右侧预览
                ])
                .split(area);

            // 左侧：表单
            let left_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // 标题
                    Constraint::Min(10),    // 表单内容
                    Constraint::Length(3),  // 错误信息/提示
                ])
                .split(main_chunks[0]);

            // 渲染标题
            let title = match &self.mode {
                ProviderFormMode::Add => "添加 Provider",
                ProviderFormMode::Edit(_) => "编辑 Provider",
            };
            let title_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(title);
            f.render_widget(title_block, left_chunks[0]);

            // 渲染表单
            self.form.render(f, left_chunks[1]);

            // 渲染错误信息或提示
            let bottom_text = if let Some(error) = &self.error_message {
                vec![
                    Span::styled("错误: ", Style::default().fg(Color::Red)),
                    Span::styled(error, Style::default().fg(Color::Red)),
                ]
            } else {
                vec![
                    Span::styled("Ctrl+P", Style::default().fg(Color::Magenta)),
                    Span::raw(" 关闭预览  "),
                    Span::styled("Ctrl+Enter", Style::default().fg(Color::Green)),
                    Span::raw(" 提交  "),
                    Span::styled("Esc", Style::default().fg(Color::Yellow)),
                    Span::raw(" 取消"),
                ]
            };

            let bottom_paragraph = Paragraph::new(Line::from(bottom_text))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(bottom_paragraph, left_chunks[2]);

            // 右侧：JSON 预览
            let preview_json = self.generate_preview_json();
            let preview_paragraph = Paragraph::new(preview_json)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Green))
                        .title("JSON 预览"),
                )
                .style(Style::default().fg(Color::White));
            f.render_widget(preview_paragraph, main_chunks[1]);
        } else {
            // 原有的垂直布局
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
                    Span::styled("Ctrl+P", Style::default().fg(Color::Magenta)),
                    Span::raw(" 预览  "),
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
}

/// 表单操作结果
pub enum FormAction {
    None,
    Submit(ProviderFormData),
    Cancel,
}

/// Provider 表单数据（改进版）
#[derive(Debug, Clone)]
pub struct ProviderFormData {
    pub id: Option<String>,
    pub name: String,
    pub settings_config: Value,
    pub meta: Option<Value>,
    pub website_url: Option<String>,
    pub notes: Option<String>,
}
