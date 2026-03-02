use super::App;
use anyhow::Result;
use std::str::FromStr;

impl App {
    /// 切换 Provider
    pub async fn switch_provider(&mut self, provider_id: &str) -> Result<()> {
        log::info!("Switching to provider: {}", provider_id);

        // 获取目标 provider
        let provider = self.providers_cache.get(provider_id)
            .ok_or_else(|| anyhow::anyhow!("Provider {} 不存在", provider_id))?
            .clone();

        // 解析 app_type
        let app_type = cc_switch_core::app_config::AppType::from_str(&self.current_app_type)
            .map_err(|e| anyhow::anyhow!("无效的应用类型: {}", e))?;

        // 更新数据库中的当前 provider
        self.db.set_current_provider(&self.current_app_type, provider_id)?;

        // 写入 live 配置文件（关键步骤）
        cc_switch_core::services::provider::write_live_snapshot(&app_type, &provider)
            .map_err(|e| anyhow::anyhow!("写入 live 配置失败: {}", e))?;

        log::info!("Provider switched to {} and live config updated", provider_id);

        // 同步所有启用的 MCP 服务器到当前应用
        self.sync_mcp_for_current_app(&app_type)?;

        // 刷新缓存
        self.refresh_providers()?;

        Ok(())
    }

    /// 启动代理服务
    pub async fn start_proxy(&mut self) -> Result<()> {
        log::info!("Starting proxy service with takeover...");

        match self.proxy_service.start_with_takeover().await {
            Ok(info) => {
                self.proxy_running = true;
                let addr = info.address.clone();
                let port = info.port;
                self.proxy_config.listen_address = addr.clone();
                self.proxy_config.listen_port = port;
                log::info!("Proxy started at {}:{} with live config takeover", addr, port);
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to start proxy: {}", e);
                Err(anyhow::anyhow!("启动代理失败: {}", e))
            }
        }
    }

    /// 停止代理服务
    pub async fn stop_proxy(&mut self) -> Result<()> {
        log::info!("Stopping proxy service...");

        match self.proxy_service.stop_with_restore().await {
            Ok(_) => {
                self.proxy_running = false;
                self.proxy_status = None;
                log::info!("Proxy stopped and live configs restored");
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to stop proxy: {}", e);
                Err(anyhow::anyhow!("停止代理失败: {}", e))
            }
        }
    }

    /// 切换代理接管状态
    pub async fn toggle_proxy_takeover(&mut self, app_type: &str, enabled: bool) -> Result<()> {
        log::info!("Toggling proxy takeover for {}: {}", app_type, enabled);

        // 调用 ProxyService 的接管切换方法
        self.proxy_service.set_takeover_for_app(app_type, enabled).await
            .map_err(|e| anyhow::anyhow!("切换接管失败: {}", e))?;

        // 更新本地状态
        match app_type {
            "claude" => self.proxy_takeover.claude = enabled,
            "codex" => self.proxy_takeover.codex = enabled,
            "gemini" => self.proxy_takeover.gemini = enabled,
            _ => return Err(anyhow::anyhow!("Unknown app type: {}", app_type)),
        }

        Ok(())
    }

    /// 显示添加 Provider 表单
    pub fn show_add_provider_form(&mut self) {
        // 使用改进的 V2 表单
        self.provider_form_v2 = Some(crate::ui::provider_form_v2::ProviderFormViewV2::new_add());
        self.mode = super::AppMode::ProviderForm;
    }

    /// 显示编辑 Provider 表单
    pub fn show_edit_provider_form(&mut self, provider_id: &str) {
        if let Some(provider) = self.providers_cache.get(provider_id) {
            // 使用改进的 V2 表单
            self.provider_form_v2 = Some(crate::ui::provider_form_v2::ProviderFormViewV2::new_edit(
                provider.id.clone(),
                provider.name.clone(),
                &provider.settings_config,
                provider.meta.as_ref().map(|m| serde_json::to_value(m).ok()).flatten().as_ref(),
                provider.website_url.clone(),
                provider.notes.clone(),
            ));
            self.mode = super::AppMode::ProviderForm;
        }
    }

    /// 关闭 Provider 表单
    pub fn close_provider_form(&mut self) {
        self.provider_form = None;
        self.provider_form_v2 = None;
        self.mode = super::AppMode::Providers;
    }

    /// 保存 Provider（新增或编辑）
    pub async fn save_provider(&mut self, data: crate::ui::provider_form::ProviderFormData) -> Result<()> {
        use cc_switch_core::Provider;

        let is_new = data.id.is_none();

        let mut provider = if let Some(id) = data.id {
            // 编辑现有 Provider
            log::info!("Updating provider: {}", id);

            self.providers_cache.get(&id)
                .ok_or_else(|| anyhow::anyhow!("Provider not found"))?
                .clone()
        } else {
            // 新增 Provider - 使用时间戳生成简单 ID
            log::info!("Adding new provider: {}", data.name);

            let id = format!("provider_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis());

            Provider::with_id(
                id,
                data.name.clone(),
                data.settings_config.clone(),
                data.website_url.clone(),
            )
        };

        // 更新字段
        provider.name = data.name;
        provider.settings_config = data.settings_config;
        provider.website_url = data.website_url;
        provider.notes = data.notes;

        // 保存到数据库
        self.db.save_provider(&self.current_app_type, &provider)?;

        // 检查是否需要写入 live 配置
        let current_provider_id = self.db.get_current_provider(&self.current_app_type)?;
        let should_write_live = if is_new {
            // 新增：如果没有当前 provider，设为当前并写入 live
            if current_provider_id.is_none() {
                self.db.set_current_provider(&self.current_app_type, &provider.id)?;
                true
            } else {
                false
            }
        } else {
            // 编辑：如果是当前 provider，写入 live
            current_provider_id.as_deref() == Some(provider.id.as_str())
        };

        if should_write_live {
            let app_type = cc_switch_core::app_config::AppType::from_str(&self.current_app_type)
                .map_err(|e| anyhow::anyhow!("无效的应用类型: {}", e))?;

            cc_switch_core::services::provider::write_live_snapshot(&app_type, &provider)
                .map_err(|e| anyhow::anyhow!("写入 live 配置失败: {}", e))?;

            log::info!("Live config updated for provider {}", provider.id);

            // 同步 MCP 配置
            self.sync_mcp_for_current_app(&app_type)?;
        }

        self.refresh_providers()?;
        self.close_provider_form();
        Ok(())
    }

    /// 保存 Provider V2（支持 meta 字段）
    pub async fn save_provider_v2(&mut self, data: crate::ui::provider_form_v2::ProviderFormData) -> Result<()> {
        use cc_switch_core::Provider;

        let is_new = data.id.is_none();

        let mut provider = if let Some(id) = &data.id {
            // 编辑现有 Provider
            log::info!("Updating provider: {}", id);

            self.providers_cache.get(id)
                .ok_or_else(|| anyhow::anyhow!("Provider not found"))?
                .clone()
        } else {
            // 新增 Provider - 使用时间戳生成简单 ID
            log::info!("Adding new provider: {}", data.name);

            let id = format!("provider_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis());

            Provider::with_id(
                id,
                data.name.clone(),
                data.settings_config.clone(),
                data.website_url.clone(),
            )
        };

        // 更新字段
        provider.name = data.name;
        provider.settings_config = data.settings_config;
        provider.website_url = data.website_url;
        provider.notes = data.notes;

        // 更新 meta 字段
        if let Some(meta_value) = data.meta {
            // 解析 meta 为 ProviderMeta 结构
            let mut meta = provider.meta.unwrap_or_default();

            // 更新 api_format
            if let Some(api_format) = meta_value.get("apiFormat").and_then(|v| v.as_str()) {
                meta.api_format = Some(api_format.to_string());
            }

            provider.meta = Some(meta);
        }

        // 保存到数据库
        self.db.save_provider(&self.current_app_type, &provider)?;

        // 检查是否需要写入 live 配置
        let current_provider_id = self.db.get_current_provider(&self.current_app_type)?;
        let should_write_live = if is_new {
            // 新增：如果没有当前 provider，设为当前并写入 live
            if current_provider_id.is_none() {
                self.db.set_current_provider(&self.current_app_type, &provider.id)?;
                true
            } else {
                false
            }
        } else {
            // 编辑：如果是当前 provider，写入 live
            current_provider_id.as_deref() == Some(provider.id.as_str())
        };

        if should_write_live {
            let app_type = cc_switch_core::app_config::AppType::from_str(&self.current_app_type)
                .map_err(|e| anyhow::anyhow!("无效的应用类型: {}", e))?;

            cc_switch_core::services::provider::write_live_snapshot(&app_type, &provider)
                .map_err(|e| anyhow::anyhow!("写入 live 配置失败: {}", e))?;

            log::info!("Live config updated for provider {}", provider.id);

            // 同步 MCP 配置
            self.sync_mcp_for_current_app(&app_type)?;
        }

        self.refresh_providers()?;
        self.close_provider_form();
        Ok(())
    }

    /// 删除 Provider
    pub async fn delete_provider(&mut self, provider_id: &str) -> Result<()> {
        log::info!("Deleting provider: {}", provider_id);

        // 检查是否为当前 provider
        let current_provider_id = self.db.get_current_provider(&self.current_app_type)?;
        if current_provider_id.as_deref() == Some(provider_id) {
            return Err(anyhow::anyhow!("无法删除当前正在使用的 Provider，请先切换到其他 Provider"));
        }

        self.db.delete_provider(provider_id, &self.current_app_type)?;
        self.refresh_providers()?;
        Ok(())
    }

    /// 删除 MCP 服务器
    pub async fn delete_mcp_server(&mut self, server_id: &str) -> Result<()> {
        use cc_switch_core::app_config::AppType;
        use cc_switch_core::mcp;

        log::info!("Deleting MCP server: {}", server_id);

        // 从所有启用的应用中移除
        if let Some(server) = self.mcp_servers_cache.get(server_id) {
            for app in server.apps.enabled_apps() {
                match app {
                    AppType::Claude => {
                        mcp::remove_server_from_claude(server_id)?;
                    }
                    AppType::Codex => {
                        mcp::remove_server_from_codex(server_id)?;
                    }
                    AppType::Gemini => {
                        mcp::remove_server_from_gemini(server_id)?;
                    }
                    AppType::OpenCode => {
                        mcp::remove_server_from_opencode(server_id)?;
                    }
                    AppType::OpenClaw => {
                        log::debug!("OpenClaw MCP support is still in development, skipping remove");
                    }
                }
            }
        }

        self.db.delete_mcp_server(server_id)?;
        self.refresh_mcp_servers()?;
        Ok(())
    }

    /// 显示添加 MCP 表单
    pub fn show_add_mcp_form(&mut self) {
        self.mcp_form = Some(crate::ui::mcp_form::McpFormView::new_add());
        self.mode = super::AppMode::McpForm;
    }

    /// 显示编辑 MCP 表单
    pub fn show_edit_mcp_form(&mut self, server_id: &str) {
        if let Some(server) = self.mcp_servers_cache.get(server_id) {
            self.mcp_form = Some(crate::ui::mcp_form::McpFormView::new_edit(
                server.id.clone(),
                server.name.clone(),
                &server.server,
                server.description.clone(),
                server.homepage.clone(),
                server.docs.clone(),
                &server.apps,
            ));
            self.mode = super::AppMode::McpForm;
        }
    }

    /// 关闭 MCP 表单
    pub fn close_mcp_form(&mut self) {
        self.mcp_form = None;
        self.mode = super::AppMode::Mcp;
    }

    /// 保存 MCP 服务器（新增或编辑）
    pub async fn save_mcp_server(&mut self, data: crate::ui::mcp_form::McpFormData) -> Result<()> {
        use cc_switch_core::app_config::{McpServer, McpApps, AppType};
        use cc_switch_core::mcp;

        // 读取旧状态（用于处理禁用）
        let prev_apps = if let Some(ref id) = data.id {
            self.mcp_servers_cache.get(id)
                .map(|s| s.apps.clone())
                .unwrap_or_default()
        } else {
            McpApps::default()
        };

        let mut server = if let Some(id) = data.id {
            // 编辑现有 MCP 服务器
            log::info!("Updating MCP server: {}", id);

            self.mcp_servers_cache.get(&id)
                .ok_or_else(|| anyhow::anyhow!("MCP server not found"))?
                .clone()
        } else {
            // 新增 MCP 服务器 - 使用时间戳生成简单 ID
            log::info!("Adding new MCP server: {}", data.name);

            let id = format!("mcp_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis());

            McpServer {
                id,
                name: data.name.clone(),
                server: data.server.clone(),
                apps: McpApps::default(),
                description: data.description.clone(),
                homepage: data.homepage.clone(),
                docs: data.docs.clone(),
                tags: Vec::new(),
            }
        };

        // 更新字段
        server.name = data.name;
        server.server = data.server;
        server.description = data.description;
        server.homepage = data.homepage;
        server.docs = data.docs;
        server.apps.claude = data.claude_enabled;
        server.apps.codex = data.codex_enabled;
        server.apps.gemini = data.gemini_enabled;
        server.apps.opencode = data.opencode_enabled;

        self.db.save_mcp_server(&server)?;

        // 处理禁用：从 live 配置移除
        if prev_apps.claude && !server.apps.claude {
            mcp::remove_server_from_claude(&server.id)?;
        }
        if prev_apps.codex && !server.apps.codex {
            mcp::remove_server_from_codex(&server.id)?;
        }
        if prev_apps.gemini && !server.apps.gemini {
            mcp::remove_server_from_gemini(&server.id)?;
        }
        if prev_apps.opencode && !server.apps.opencode {
            mcp::remove_server_from_opencode(&server.id)?;
        }

        // 同步到启用的应用
        for app in server.apps.enabled_apps() {
            match app {
                AppType::Claude => {
                    mcp::sync_single_server_to_claude(&Default::default(), &server.id, &server.server)?;
                }
                AppType::Codex => {
                    mcp::sync_single_server_to_codex(&Default::default(), &server.id, &server.server)?;
                }
                AppType::Gemini => {
                    mcp::sync_single_server_to_gemini(&Default::default(), &server.id, &server.server)?;
                }
                AppType::OpenCode => {
                    mcp::sync_single_server_to_opencode(&Default::default(), &server.id, &server.server)?;
                }
                AppType::OpenClaw => {
                    log::debug!("OpenClaw MCP support is still in development, skipping sync");
                }
            }
        }

        self.refresh_mcp_servers()?;
        self.close_mcp_form();
        Ok(())
    }

    /// 切换 MCP 服务器的应用启用状态
    pub async fn toggle_mcp_app(&mut self, server_id: &str, app: &str, enabled: bool) -> Result<()> {
        use cc_switch_core::app_config::AppType;
        use cc_switch_core::mcp;

        log::info!("Toggling MCP {} for {}: {}", server_id, app, enabled);

        // 获取服务器并更新应用启用状态
        if let Some(mut server) = self.mcp_servers_cache.get(server_id).cloned() {
            let app_type = AppType::from_str(app)
                .map_err(|e| anyhow::anyhow!("无效的应用类型: {}", e))?;

            server.apps.set_enabled_for(&app_type, enabled);

            // 保存到数据库
            self.db.save_mcp_server(&server)?;

            // 同步到 live 配置
            if enabled {
                match app_type {
                    AppType::Claude => {
                        mcp::sync_single_server_to_claude(&Default::default(), &server.id, &server.server)?;
                    }
                    AppType::Codex => {
                        mcp::sync_single_server_to_codex(&Default::default(), &server.id, &server.server)?;
                    }
                    AppType::Gemini => {
                        mcp::sync_single_server_to_gemini(&Default::default(), &server.id, &server.server)?;
                    }
                    AppType::OpenCode => {
                        mcp::sync_single_server_to_opencode(&Default::default(), &server.id, &server.server)?;
                    }
                    AppType::OpenClaw => {
                        log::debug!("OpenClaw MCP support is still in development, skipping sync");
                    }
                }
            } else {
                match app_type {
                    AppType::Claude => {
                        mcp::remove_server_from_claude(&server.id)?;
                    }
                    AppType::Codex => {
                        mcp::remove_server_from_codex(&server.id)?;
                    }
                    AppType::Gemini => {
                        mcp::remove_server_from_gemini(&server.id)?;
                    }
                    AppType::OpenCode => {
                        mcp::remove_server_from_opencode(&server.id)?;
                    }
                    AppType::OpenClaw => {
                        log::debug!("OpenClaw MCP support is still in development, skipping remove");
                    }
                }
            }

            self.refresh_mcp_servers()?;
        }

        Ok(())
    }

    /// 同步统一供应商
    pub async fn sync_universal_provider(&mut self, provider_id: &str) -> Result<()> {
        log::info!("Syncing universal provider: {}", provider_id);

        // TODO: 实现实际的同步逻辑
        self.refresh_universal_providers()?;
        Ok(())
    }

    /// 显示添加统一供应商表单
    pub fn show_add_universal_form(&mut self) {
        self.universal_form = Some(crate::ui::universal_form::UniversalFormView::new_add());
        self.mode = super::AppMode::UniversalForm;
    }

    /// 显示编辑统一供应商表单
    pub fn show_edit_universal_form(&mut self, provider_id: &str) {
        // 从数据库获取完整的 UniversalProvider 数据
        if let Ok(Some(provider)) = self.db.get_universal_provider(provider_id) {
            self.universal_form = Some(crate::ui::universal_form::UniversalFormView::new_edit(
                provider.id.clone(),
                provider.name.clone(),
                provider.provider_type.clone(),
                provider.base_url.clone(),
                provider.api_key.clone(),
                provider.website_url.clone(),
                provider.notes.clone(),
                provider.apps.claude,
                provider.apps.codex,
                provider.apps.gemini,
            ));
            self.mode = super::AppMode::UniversalForm;
        }
    }

    /// 关闭统一供应商表单
    pub fn close_universal_form(&mut self) {
        self.universal_form = None;
        self.mode = super::AppMode::Universal;
    }

    /// 保存统一供应商（新增或编辑）
    pub async fn save_universal_provider(&mut self, data: crate::ui::universal_form::UniversalFormData) -> Result<()> {
        use cc_switch_core::provider::{UniversalProvider, UniversalProviderApps};

        let provider = if let Some(id) = data.id {
            // 编辑现有统一供应商
            log::info!("Updating universal provider: {}", id);

            let mut provider = self.db.get_universal_provider(&id)?
                .ok_or_else(|| anyhow::anyhow!("Universal provider not found"))?;

            provider.name = data.name;
            provider.provider_type = data.provider_type;
            provider.base_url = data.base_url;
            provider.api_key = data.api_key;
            provider.website_url = data.website_url;
            provider.notes = data.notes;
            provider.apps.claude = data.claude_enabled;
            provider.apps.codex = data.codex_enabled;
            provider.apps.gemini = data.gemini_enabled;
            provider
        } else {
            // 新增统一供应商
            log::info!("Adding new universal provider: {}", data.name);

            let id = format!("universal_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis());

            let mut provider = UniversalProvider::new(
                id,
                data.name,
                data.provider_type,
                data.base_url,
                data.api_key,
            );

            provider.website_url = data.website_url;
            provider.notes = data.notes;
            provider.apps = UniversalProviderApps {
                claude: data.claude_enabled,
                codex: data.codex_enabled,
                gemini: data.gemini_enabled,
            };
            provider
        };

        self.db.save_universal_provider(&provider)?;
        self.refresh_universal_providers()?;
        self.close_universal_form();
        Ok(())
    }

    /// 删除统一供应商
    pub async fn delete_universal_provider(&mut self, provider_id: &str) -> Result<()> {
        log::info!("Deleting universal provider: {}", provider_id);

        self.db.delete_universal_provider(provider_id)?;
        self.refresh_universal_providers()?;
        Ok(())
    }

    /// 刷新代理状态
    pub async fn refresh_proxy_status_async(&mut self) -> Result<()> {
        match self.proxy_service.get_status().await {
            Ok(status) => {
                self.proxy_running = status.running;
                if status.running {
                    self.proxy_status = Some(super::ProxyStatusStub {
                        uptime_seconds: status.uptime_seconds,
                        active_connections: status.active_connections as u32,
                        total_requests: status.total_requests,
                        success_rate: status.success_rate as f64,
                    });
                } else {
                    self.proxy_status = None;
                }
            }
            Err(_) => {
                self.proxy_running = false;
                self.proxy_status = None;
            }
        }
        Ok(())
    }

    /// 同步所有启用的 MCP 服务器到指定应用
    fn sync_mcp_for_current_app(&self, app_type: &cc_switch_core::app_config::AppType) -> Result<()> {
        use cc_switch_core::mcp;

        log::info!("Syncing MCP servers for app: {:?}", app_type);

        // 获取所有 MCP 服务器
        let servers = self.db.get_all_mcp_servers()?;

        // 同步所有为当前应用启用的 MCP 服务器
        for server in servers.values() {
            if server.apps.is_enabled_for(app_type) {
                match app_type {
                    cc_switch_core::app_config::AppType::Claude => {
                        mcp::sync_single_server_to_claude(&Default::default(), &server.id, &server.server)?;
                    }
                    cc_switch_core::app_config::AppType::Codex => {
                        mcp::sync_single_server_to_codex(&Default::default(), &server.id, &server.server)?;
                    }
                    cc_switch_core::app_config::AppType::Gemini => {
                        mcp::sync_single_server_to_gemini(&Default::default(), &server.id, &server.server)?;
                    }
                    cc_switch_core::app_config::AppType::OpenCode => {
                        mcp::sync_single_server_to_opencode(&Default::default(), &server.id, &server.server)?;
                    }
                    cc_switch_core::app_config::AppType::OpenClaw => {
                        log::debug!("OpenClaw MCP support is still in development, skipping sync");
                    }
                }
            }
        }

        log::info!("MCP sync completed for app: {:?}", app_type);
        Ok(())
    }
}
