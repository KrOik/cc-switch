use super::App;
use indexmap::IndexMap;
use std::collections::HashMap;

impl App {
    // Provider 相关
    pub fn get_providers_for_current_app(&self) -> &IndexMap<String, cc_switch_core::Provider> {
        &self.providers_cache
    }

    pub fn get_current_provider_id(&self) -> String {
        // TODO: 从数据库获取当前 provider ID
        "default".to_string()
    }

    pub fn get_selected_provider_index(&self) -> Option<usize> {
        self.selected_provider_index
    }

    pub fn get_selected_provider(&self) -> Option<&cc_switch_core::Provider> {
        self.selected_provider_index
            .and_then(|idx| self.providers_cache.values().nth(idx))
    }

    // MCP 相关
    pub fn get_mcp_servers(&self) -> &IndexMap<String, cc_switch_core::McpServer> {
        &self.mcp_servers_cache
    }

    pub fn get_selected_mcp_index(&self) -> Option<usize> {
        self.selected_mcp_index
    }

    pub fn get_selected_mcp_server(&self) -> Option<&cc_switch_core::McpServer> {
        self.selected_mcp_index
            .and_then(|idx| self.mcp_servers_cache.values().nth(idx))
    }

    // 统一供应商相关
    pub fn get_universal_providers(&self) -> &HashMap<String, super::UniversalProviderStub> {
        &self.universal_providers_cache
    }

    pub fn get_selected_universal_index(&self) -> Option<usize> {
        self.selected_universal_index
    }

    pub fn get_selected_universal_provider(&self) -> Option<&super::UniversalProviderStub> {
        self.selected_universal_index
            .and_then(|idx| self.universal_providers_cache.values().nth(idx))
    }

    // 代理相关
    pub fn is_proxy_running(&self) -> bool {
        self.proxy_running
    }

    pub fn get_proxy_status(&self) -> Option<&super::ProxyStatusStub> {
        self.proxy_status.as_ref()
    }

    pub fn get_proxy_config(&self) -> &super::ProxyConfigStub {
        &self.proxy_config
    }

    pub fn get_proxy_takeover_status(&self) -> &super::ProxyTakeoverStub {
        &self.proxy_takeover
    }

    // Dashboard 相关
    pub fn get_active_providers(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("claude".to_string(), "未配置".to_string());
        map.insert("codex".to_string(), "未配置".to_string());
        map.insert("gemini".to_string(), "未配置".to_string());
        map.insert("opencode".to_string(), "未配置".to_string());
        map.insert("openclaw".to_string(), "未配置".to_string());
        map
    }

    // 配置相关
    pub fn get_config_info(&self) -> ConfigInfo {
        ConfigInfo {
            config_dir: "~/.cc-switch".to_string(),
            database_path: "~/.cc-switch/cc-switch.db".to_string(),
            database_size: 0,
            total_providers: self.providers_cache.len(),
            total_mcp_servers: self.mcp_servers_cache.len(),
        }
    }

    pub fn get_app_paths(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("claude".to_string(), "未配置".to_string());
        map.insert("codex".to_string(), "未配置".to_string());
        map.insert("gemini".to_string(), "未配置".to_string());
        map.insert("opencode".to_string(), "未配置".to_string());
        map.insert("openclaw".to_string(), "未配置".to_string());
        map
    }
}

pub struct ConfigInfo {
    pub config_dir: String,
    pub database_path: String,
    pub database_size: u64,
    pub total_providers: usize,
    pub total_mcp_servers: usize,
}
