use super::{App, AppMode};
use anyhow::Result;
use crossterm::event::KeyCode;

impl App {
    /// 处理键盘输入（扩展版本，支持更多操作）
    pub fn handle_key_extended(&mut self, key: KeyCode, modifiers: crossterm::event::KeyModifiers) -> Result<Option<AppAction>> {
        match self.mode {
            AppMode::Dashboard => self.handle_dashboard_key(key),
            AppMode::Providers => self.handle_providers_key(key),
            AppMode::Proxy => self.handle_proxy_key(key),
            AppMode::Mcp => self.handle_mcp_key(key),
            AppMode::Universal => self.handle_universal_key(key),
            AppMode::Config => self.handle_config_key(key),
            AppMode::ProviderForm => self.handle_provider_form_key(key, modifiers),
            AppMode::McpForm => self.handle_mcp_form_key(key, modifiers),
            AppMode::UniversalForm => self.handle_universal_form_key(key, modifiers),
        }
    }

    fn handle_dashboard_key(&mut self, key: KeyCode) -> Result<Option<AppAction>> {
        match key {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_quit = true;
                Ok(None)
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.mode = AppMode::Providers;
                self.refresh_providers()?;
                Ok(None)
            }
            KeyCode::Char('x') | KeyCode::Char('X') => {
                self.mode = AppMode::Proxy;
                Ok(Some(AppAction::RefreshProxyStatus))
            }
            KeyCode::Char('m') | KeyCode::Char('M') => {
                self.mode = AppMode::Mcp;
                self.refresh_mcp_servers()?;
                Ok(None)
            }
            KeyCode::Char('u') | KeyCode::Char('U') => {
                self.mode = AppMode::Universal;
                self.refresh_universal_providers()?;
                Ok(None)
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                self.mode = AppMode::Config;
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn handle_providers_key(&mut self, key: KeyCode) -> Result<Option<AppAction>> {
        match key {
            KeyCode::Esc => {
                self.mode = AppMode::Dashboard;
                Ok(None)
            }
            KeyCode::Up => {
                self.handle_list_up();
                Ok(None)
            }
            KeyCode::Down => {
                self.handle_list_down();
                Ok(None)
            }
            KeyCode::Enter => {
                if let Some(provider) = self.get_selected_provider() {
                    let id = provider.id.clone();
                    Ok(Some(AppAction::SwitchProvider(id)))
                } else {
                    Ok(None)
                }
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if let Some(provider) = self.get_selected_provider() {
                    let id = provider.id.clone();
                    let name = provider.name.clone();
                    self.show_delete_provider_confirm(id, &name);
                    Ok(None)
                } else {
                    Ok(None)
                }
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                self.show_add_provider_form();
                Ok(None)
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                if let Some(provider) = self.get_selected_provider() {
                    let id = provider.id.clone();
                    self.show_edit_provider_form(&id);
                    Ok(None)
                } else {
                    Ok(None)
                }
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.refresh_providers()?;
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn handle_proxy_key(&mut self, key: KeyCode) -> Result<Option<AppAction>> {
        match key {
            KeyCode::Esc => {
                self.mode = AppMode::Dashboard;
                Ok(None)
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                if self.proxy_running {
                    Ok(Some(AppAction::StopProxy))
                } else {
                    Ok(Some(AppAction::StartProxy))
                }
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                Ok(Some(AppAction::RestartProxy))
            }
            KeyCode::Char('1') => {
                let enabled = !self.proxy_takeover.claude;
                Ok(Some(AppAction::ToggleProxyTakeover("claude".to_string(), enabled)))
            }
            KeyCode::Char('2') => {
                let enabled = !self.proxy_takeover.codex;
                Ok(Some(AppAction::ToggleProxyTakeover("codex".to_string(), enabled)))
            }
            KeyCode::Char('3') => {
                let enabled = !self.proxy_takeover.gemini;
                Ok(Some(AppAction::ToggleProxyTakeover("gemini".to_string(), enabled)))
            }
            _ => Ok(None),
        }
    }

    fn handle_mcp_key(&mut self, key: KeyCode) -> Result<Option<AppAction>> {
        match key {
            KeyCode::Esc => {
                self.mode = AppMode::Dashboard;
                Ok(None)
            }
            KeyCode::Up => {
                self.handle_list_up();
                Ok(None)
            }
            KeyCode::Down => {
                self.handle_list_down();
                Ok(None)
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if let Some(server) = self.get_selected_mcp_server() {
                    let id = server.id.clone();
                    let name = if server.name.is_empty() {
                        server.id.clone()
                    } else {
                        server.name.clone()
                    };
                    self.show_delete_mcp_confirm(id, &name);
                    Ok(None)
                } else {
                    Ok(None)
                }
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                self.show_add_mcp_form();
                Ok(None)
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                if let Some(server) = self.get_selected_mcp_server() {
                    let id = server.id.clone();
                    self.show_edit_mcp_form(&id);
                    Ok(None)
                } else {
                    Ok(None)
                }
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.refresh_mcp_servers()?;
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn handle_universal_key(&mut self, key: KeyCode) -> Result<Option<AppAction>> {
        match key {
            KeyCode::Esc => {
                self.mode = AppMode::Dashboard;
                Ok(None)
            }
            KeyCode::Up => {
                self.handle_list_up();
                Ok(None)
            }
            KeyCode::Down => {
                self.handle_list_down();
                Ok(None)
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                if let Some(provider) = self.get_selected_universal_provider() {
                    let id = provider.id.clone();
                    Ok(Some(AppAction::SyncUniversalProvider(id)))
                } else {
                    Ok(None)
                }
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                self.show_add_universal_form();
                Ok(None)
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                if let Some(provider) = self.get_selected_universal_provider() {
                    let id = provider.id.clone();
                    self.show_edit_universal_form(&id);
                    Ok(None)
                } else {
                    Ok(None)
                }
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if let Some(provider) = self.get_selected_universal_provider() {
                    let id = provider.id.clone();
                    let name = provider.name.clone();
                    self.show_delete_universal_confirm(id, &name);
                    Ok(None)
                } else {
                    Ok(None)
                }
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.refresh_universal_providers()?;
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn handle_config_key(&mut self, key: KeyCode) -> Result<Option<AppAction>> {
        match key {
            KeyCode::Esc => {
                self.mode = AppMode::Dashboard;
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn handle_provider_form_key(&mut self, key: KeyCode, modifiers: crossterm::event::KeyModifiers) -> Result<Option<AppAction>> {
        // 优先处理 V2 表单
        if let Some(form) = &mut self.provider_form_v2 {
            use crate::ui::provider_form_v2::FormAction;

            match form.handle_key(key, modifiers) {
                FormAction::Submit(data) => {
                    Ok(Some(AppAction::SaveProviderV2(data)))
                }
                FormAction::Cancel => {
                    self.close_provider_form();
                    Ok(None)
                }
                FormAction::None => Ok(None),
            }
        } else if let Some(form) = &mut self.provider_form {
            use crate::ui::provider_form::FormAction;

            match form.handle_key(key, modifiers) {
                FormAction::Submit(data) => {
                    Ok(Some(AppAction::SaveProvider(data)))
                }
                FormAction::Cancel => {
                    self.close_provider_form();
                    Ok(None)
                }
                FormAction::None => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    fn handle_mcp_form_key(&mut self, key: KeyCode, modifiers: crossterm::event::KeyModifiers) -> Result<Option<AppAction>> {
        if let Some(form) = &mut self.mcp_form {
            use crate::ui::mcp_form::FormAction;

            match form.handle_key(key, modifiers) {
                FormAction::Submit(data) => {
                    Ok(Some(AppAction::SaveMcpServer(data)))
                }
                FormAction::Cancel => {
                    self.close_mcp_form();
                    Ok(None)
                }
                FormAction::None => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    fn handle_universal_form_key(&mut self, key: KeyCode, modifiers: crossterm::event::KeyModifiers) -> Result<Option<AppAction>> {
        if let Some(form) = &mut self.universal_form {
            use crate::ui::universal_form::FormAction;

            match form.handle_key(key, modifiers) {
                FormAction::Submit(data) => {
                    Ok(Some(AppAction::SaveUniversalProvider(data)))
                }
                FormAction::Cancel => {
                    self.close_universal_form();
                    Ok(None)
                }
                FormAction::None => Ok(None),
            }
        } else {
            Ok(None)
        }
    }
}

/// 应用操作枚举（需要异步执行的操作）
#[derive(Debug, Clone)]
pub enum AppAction {
    SwitchProvider(String),
    DeleteProvider(String),
    SaveProvider(crate::ui::provider_form::ProviderFormData),
    SaveProviderV2(crate::ui::provider_form_v2::ProviderFormData),
    SaveMcpServer(crate::ui::mcp_form::McpFormData),
    SaveUniversalProvider(crate::ui::universal_form::UniversalFormData),
    StartProxy,
    StopProxy,
    RestartProxy,
    ToggleProxyTakeover(String, bool),
    DeleteMcpServer(String),
    DeleteUniversalProvider(String),
    SyncUniversalProvider(String),
    RefreshProxyStatus,
}
