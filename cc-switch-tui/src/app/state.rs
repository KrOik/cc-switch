use anyhow::Result;
use cc_switch_core::{Database, ProxyService};
use indexmap::IndexMap;
use std::sync::Arc;
use std::collections::HashMap;
use crate::ui::dialog::ConfirmDialog;

pub enum AppMode {
    Dashboard,
    Providers,
    Proxy,
    Mcp,
    Universal,
    Config,
}

pub struct App {
    pub mode: AppMode,
    pub should_quit: bool,
    pub(crate) db: Arc<Database>,
    pub(crate) proxy_service: ProxyService,

    // 当前选择的应用类型
    pub(crate) current_app_type: String,

    // 列表选择索引
    pub(crate) selected_provider_index: Option<usize>,
    pub(crate) selected_mcp_index: Option<usize>,
    pub(crate) selected_universal_index: Option<usize>,

    // 缓存的数据
    pub(crate) providers_cache: IndexMap<String, cc_switch_core::Provider>,
    pub(crate) mcp_servers_cache: IndexMap<String, cc_switch_core::McpServer>,
    pub(crate) universal_providers_cache: HashMap<String, UniversalProviderStub>,

    // 代理状态缓存
    pub(crate) proxy_running: bool,
    pub(crate) proxy_status: Option<ProxyStatusStub>,
    pub(crate) proxy_config: ProxyConfigStub,
    pub(crate) proxy_takeover: ProxyTakeoverStub,

    // UI 状态
    pub(crate) confirm_dialog: Option<ConfirmDialog>,
    pub(crate) pending_action: Option<PendingAction>,
}

// 临时结构体，用于存储数据（避免依赖完整的类型定义）
#[derive(Clone)]
pub struct UniversalProviderStub {
    pub id: String,
    pub name: String,
    pub enabled_apps: EnabledAppsStub,
}

#[derive(Clone)]
pub struct EnabledAppsStub {
    pub claude: bool,
    pub codex: bool,
    pub gemini: bool,
}

#[derive(Clone)]
pub struct ProxyStatusStub {
    pub uptime_seconds: u64,
    pub active_connections: u32,
    pub total_requests: u64,
    pub success_rate: f64,
}

#[derive(Clone)]
pub struct ProxyConfigStub {
    pub listen_address: String,
    pub listen_port: u16,
    pub enable_logging: bool,
}

#[derive(Clone)]
pub struct ProxyTakeoverStub {
    pub claude: bool,
    pub codex: bool,
    pub gemini: bool,
}

impl Default for ProxyConfigStub {
    fn default() -> Self {
        Self {
            listen_address: "127.0.0.1".to_string(),
            listen_port: 15721,
            enable_logging: true,
        }
    }
}

impl Default for ProxyTakeoverStub {
    fn default() -> Self {
        Self {
            claude: false,
            codex: false,
            gemini: false,
        }
    }
}

/// 待执行的操作（等待用户确认）
#[derive(Clone, Debug)]
pub enum PendingAction {
    DeleteProvider(String),
    DeleteMcpServer(String),
    DeleteUniversalProvider(String),
    StopProxy,
    RestartProxy,
}

impl App {
    pub fn new() -> Result<Self> {
        let db = Arc::new(Database::init()?);
        let proxy_service = ProxyService::new(db.clone());

        Ok(Self {
            mode: AppMode::Dashboard,
            should_quit: false,
            db,
            proxy_service,
            current_app_type: "claude".to_string(),
            selected_provider_index: Some(0),
            selected_mcp_index: Some(0),
            selected_universal_index: Some(0),
            providers_cache: IndexMap::new(),
            mcp_servers_cache: IndexMap::new(),
            universal_providers_cache: HashMap::new(),
            proxy_running: false,
            proxy_status: None,
            proxy_config: ProxyConfigStub::default(),
            proxy_takeover: ProxyTakeoverStub::default(),
            confirm_dialog: None,
            pending_action: None,
        })
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        use crossterm::event::KeyCode;

        match key {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                if matches!(self.mode, AppMode::Dashboard) {
                    self.should_quit = true;
                }
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.mode = AppMode::Providers;
                self.refresh_providers()?;
            }
            KeyCode::Char('x') | KeyCode::Char('X') => {
                self.mode = AppMode::Proxy;
                self.refresh_proxy_status()?;
            }
            KeyCode::Char('m') | KeyCode::Char('M') => {
                self.mode = AppMode::Mcp;
                self.refresh_mcp_servers()?;
            }
            KeyCode::Char('u') | KeyCode::Char('U') => {
                self.mode = AppMode::Universal;
                self.refresh_universal_providers()?;
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                self.mode = AppMode::Config;
            }
            KeyCode::Esc => {
                self.mode = AppMode::Dashboard;
            }
            KeyCode::Up => {
                self.handle_list_up();
            }
            KeyCode::Down => {
                self.handle_list_down();
            }
            _ => {}
        }
        Ok(())
    }

    pub(crate) fn handle_list_up(&mut self) {
        match self.mode {
            AppMode::Providers => {
                if let Some(idx) = self.selected_provider_index {
                    if idx > 0 {
                        self.selected_provider_index = Some(idx - 1);
                    }
                }
            }
            AppMode::Mcp => {
                if let Some(idx) = self.selected_mcp_index {
                    if idx > 0 {
                        self.selected_mcp_index = Some(idx - 1);
                    }
                }
            }
            AppMode::Universal => {
                if let Some(idx) = self.selected_universal_index {
                    if idx > 0 {
                        self.selected_universal_index = Some(idx - 1);
                    }
                }
            }
            _ => {}
        }
    }

    pub(crate) fn handle_list_down(&mut self) {
        match self.mode {
            AppMode::Providers => {
                let max = self.providers_cache.len().saturating_sub(1);
                if let Some(idx) = self.selected_provider_index {
                    if idx < max {
                        self.selected_provider_index = Some(idx + 1);
                    }
                }
            }
            AppMode::Mcp => {
                let max = self.mcp_servers_cache.len().saturating_sub(1);
                if let Some(idx) = self.selected_mcp_index {
                    if idx < max {
                        self.selected_mcp_index = Some(idx + 1);
                    }
                }
            }
            AppMode::Universal => {
                let max = self.universal_providers_cache.len().saturating_sub(1);
                if let Some(idx) = self.selected_universal_index {
                    if idx < max {
                        self.selected_universal_index = Some(idx + 1);
                    }
                }
            }
            _ => {}
        }
    }

    pub(crate) fn refresh_providers(&mut self) -> Result<()> {
        self.providers_cache = self.db.get_all_providers(&self.current_app_type)?;
        Ok(())
    }

    pub(crate) fn refresh_mcp_servers(&mut self) -> Result<()> {
        self.mcp_servers_cache = self.db.get_all_mcp_servers()?;
        Ok(())
    }

    pub(crate) fn refresh_universal_providers(&mut self) -> Result<()> {
        // TODO: 实现统一供应商的获取
        // 目前使用空数据
        self.universal_providers_cache.clear();
        Ok(())
    }

    pub(crate) fn refresh_proxy_status(&mut self) -> Result<()> {
        // TODO: 实现代理状态的获取
        // 目前使用默认值
        self.proxy_running = false;
        self.proxy_status = None;
        Ok(())
    }
}
