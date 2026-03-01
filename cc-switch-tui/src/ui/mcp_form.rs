use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use serde_json::Value;

use super::forms::{Checkbox, FormContainer, FormValue, TextArea, TextInput};

/// MCP 表单模式
#[derive(Debug, Clone, PartialEq)]
pub enum McpFormMode {
    Add,
    Edit(String), // server_id
}

/// MCP 表单视图
pub struct McpFormView {
    form: FormContainer,
    mode: McpFormMode,
    error_message: Option<String>,
}

impl McpFormView {
    /// 创建新增 MCP 表单
    pub fn new_add() -> Self {
        let mut form = FormContainer::new("添加 MCP 服务器");

        // 名称（必填）
        form = form.add_text_input(
            TextInput::new("名称")
                .with_placeholder("例如: filesystem")
                .required(),
        );

        // 服务器配置 JSON（必填）
        form = form.add_text_area(
            TextArea::new("服务器配置 (JSON)")
                .with_value(Self::default_server_json())
                .required(),
        );

        // 描述（可选）
        form = form.add_text_input(
            TextInput::new("描述")
                .with_placeholder("服务器功能描述"),
        );

        // 主页 URL（可选）
        form = form.add_text_input(
            TextInput::new("主页 URL")
                .with_placeholder("https://..."),
        );

        // 文档 URL（可选）
        form = form.add_text_input(
            TextInput::new("文档 URL")
                .with_placeholder("https://..."),
        );

        // 应用启用状态
        form = form.add_checkbox(Checkbox::new("启用 Claude"));
        form = form.add_checkbox(Checkbox::new("启用 Codex"));
        form = form.add_checkbox(Checkbox::new("启用 Gemini"));
        form = form.add_checkbox(Checkbox::new("启用 OpenCode"));

        form.activate();

        Self {
            form,
            mode: McpFormMode::Add,
            error_message: None,
        }
    }

    /// 创建编辑 MCP 表单
    pub fn new_edit(
        server_id: String,
        name: String,
        server: &Value,
        description: Option<String>,
        homepage: Option<String>,
        docs: Option<String>,
        apps: &cc_switch_core::app_config::McpApps,
    ) -> Self {
        let mut form = FormContainer::new("编辑 MCP 服务器");

        // 名称（必填）
        form = form.add_text_input(TextInput::new("名称").with_value(name).required());

        // 服务器配置 JSON（必填）
        let server_json = serde_json::to_string_pretty(server)
            .unwrap_or_else(|_| "{}".to_string());
        form = form.add_text_area(
            TextArea::new("服务器配置 (JSON)")
                .with_value(server_json)
                .required(),
        );

        // 描述（可选）
        form = form.add_text_input(
            TextInput::new("描述")
                .with_value(description.unwrap_or_default()),
        );

        // 主页 URL（可选）
        form = form.add_text_input(
            TextInput::new("主页 URL")
                .with_value(homepage.unwrap_or_default()),
        );

        // 文档 URL（可选）
        form = form.add_text_input(
            TextInput::new("文档 URL")
                .with_value(docs.unwrap_or_default()),
        );

        // 应用启用状态
        let mut checkbox_claude = Checkbox::new("启用 Claude");
        if apps.claude {
            checkbox_claude = checkbox_claude.checked();
        }
        form = form.add_checkbox(checkbox_claude);

        let mut checkbox_codex = Checkbox::new("启用 Codex");
        if apps.codex {
            checkbox_codex = checkbox_codex.checked();
        }
        form = form.add_checkbox(checkbox_codex);

        let mut checkbox_gemini = Checkbox::new("启用 Gemini");
        if apps.gemini {
            checkbox_gemini = checkbox_gemini.checked();
        }
        form = form.add_checkbox(checkbox_gemini);

        let mut checkbox_opencode = Checkbox::new("启用 OpenCode");
        if apps.opencode {
            checkbox_opencode = checkbox_opencode.checked();
        }
        form = form.add_checkbox(checkbox_opencode);

        form.activate();

        Self {
            form,
            mode: McpFormMode::Edit(server_id),
            error_message: None,
        }
    }

    /// 默认服务器配置 JSON 模板
    fn default_server_json() -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "command": "npx",
            "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/directory"]
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
        if values.len() < 9 {
            self.error_message = Some("表单数据不完整".to_string());
            return FormAction::None;
        }

        let name = values[0].as_text().unwrap_or("").to_string();
        let server_json = values[1].as_text().unwrap_or("{}");
        let description = values[2].as_text().unwrap_or("");
        let homepage = values[3].as_text().unwrap_or("");
        let docs = values[4].as_text().unwrap_or("");
        let claude_enabled = values[5].as_bool().unwrap_or(false);
        let codex_enabled = values[6].as_bool().unwrap_or(false);
        let gemini_enabled = values[7].as_bool().unwrap_or(false);
        let opencode_enabled = values[8].as_bool().unwrap_or(false);

        // 解析 JSON 配置
        let server = match serde_json::from_str::<Value>(server_json) {
            Ok(v) => v,
            Err(e) => {
                self.error_message = Some(format!("JSON 格式错误: {}", e));
                return FormAction::None;
            }
        };

        // 返回提交数据
        match &self.mode {
            McpFormMode::Add => FormAction::Submit(McpFormData {
                id: None,
                name,
                server,
                description: if description.is_empty() {
                    None
                } else {
                    Some(description.to_string())
                },
                homepage: if homepage.is_empty() {
                    None
                } else {
                    Some(homepage.to_string())
                },
                docs: if docs.is_empty() {
                    None
                } else {
                    Some(docs.to_string())
                },
                claude_enabled,
                codex_enabled,
                gemini_enabled,
                opencode_enabled,
            }),
            McpFormMode::Edit(id) => FormAction::Submit(McpFormData {
                id: Some(id.clone()),
                name,
                server,
                description: if description.is_empty() {
                    None
                } else {
                    Some(description.to_string())
                },
                homepage: if homepage.is_empty() {
                    None
                } else {
                    Some(homepage.to_string())
                },
                docs: if docs.is_empty() {
                    None
                } else {
                    Some(docs.to_string())
                },
                claude_enabled,
                codex_enabled,
                gemini_enabled,
                opencode_enabled,
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
            McpFormMode::Add => "添加 MCP 服务器",
            McpFormMode::Edit(_) => "编辑 MCP 服务器",
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
    Submit(McpFormData),
    Cancel,
}

/// MCP 表单数据
#[derive(Debug, Clone)]
pub struct McpFormData {
    pub id: Option<String>,
    pub name: String,
    pub server: Value,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub docs: Option<String>,
    pub claude_enabled: bool,
    pub codex_enabled: bool,
    pub gemini_enabled: bool,
    pub opencode_enabled: bool,
}
