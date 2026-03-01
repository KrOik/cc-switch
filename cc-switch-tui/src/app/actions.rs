use super::App;
use anyhow::Result;

impl App {
    /// 切换 Provider
    pub async fn switch_provider(&mut self, provider_id: &str) -> Result<()> {
        // TODO: 实现实际的切换逻辑
        // 1. 更新数据库中的当前 provider
        // 2. 写入 live 配置
        // 3. 刷新缓存

        log::info!("Switching to provider: {}", provider_id);
        self.refresh_providers()?;
        Ok(())
    }

    /// 启动代理服务
    pub async fn start_proxy(&mut self) -> Result<()> {
        log::info!("Starting proxy service...");

        match self.proxy_service.start().await {
            Ok(info) => {
                self.proxy_running = true;
                let addr = info.address.clone();
                let port = info.port;
                self.proxy_config.listen_address = addr.clone();
                self.proxy_config.listen_port = port;
                log::info!("Proxy started at {}:{}", addr, port);
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

        match self.proxy_service.stop().await {
            Ok(_) => {
                self.proxy_running = false;
                self.proxy_status = None;
                log::info!("Proxy stopped");
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

        // TODO: 实现实际的接管切换逻辑
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
        self.provider_form = Some(crate::ui::provider_form::ProviderFormView::new_add());
        self.mode = super::AppMode::ProviderForm;
    }

    /// 显示编辑 Provider 表单
    pub fn show_edit_provider_form(&mut self, provider_id: &str) {
        if let Some(provider) = self.providers_cache.get(provider_id) {
            self.provider_form = Some(crate::ui::provider_form::ProviderFormView::new_edit(
                provider.id.clone(),
                provider.name.clone(),
                &provider.settings_config,
                provider.website_url.clone(),
                provider.notes.clone(),
            ));
            self.mode = super::AppMode::ProviderForm;
        }
    }

    /// 关闭 Provider 表单
    pub fn close_provider_form(&mut self) {
        self.provider_form = None;
        self.mode = super::AppMode::Providers;
    }

    /// 保存 Provider（新增或编辑）
    pub async fn save_provider(&mut self, data: crate::ui::provider_form::ProviderFormData) -> Result<()> {
        use cc_switch_core::Provider;

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

        self.db.save_provider(&self.current_app_type, &provider)?;
        self.refresh_providers()?;
        self.close_provider_form();
        Ok(())
    }

    /// 删除 Provider
    pub async fn delete_provider(&mut self, provider_id: &str) -> Result<()> {
        log::info!("Deleting provider: {}", provider_id);

        self.db.delete_provider(provider_id, &self.current_app_type)?;
        self.refresh_providers()?;
        Ok(())
    }

    /// 删除 MCP 服务器
    pub async fn delete_mcp_server(&mut self, server_id: &str) -> Result<()> {
        log::info!("Deleting MCP server: {}", server_id);

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
        use cc_switch_core::app_config::{McpServer, McpApps};

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
        self.refresh_mcp_servers()?;
        self.close_mcp_form();
        Ok(())
    }

    /// 切换 MCP 服务器的应用启用状态
    pub async fn toggle_mcp_app(&mut self, server_id: &str, app: &str, enabled: bool) -> Result<()> {
        log::info!("Toggling MCP {} for {}: {}", server_id, app, enabled);

        // TODO: 实现实际的切换逻辑
        self.refresh_mcp_servers()?;
        Ok(())
    }

    /// 同步统一供应商
    pub async fn sync_universal_provider(&mut self, provider_id: &str) -> Result<()> {
        log::info!("Syncing universal provider: {}", provider_id);

        // TODO: 实现实际的同步逻辑
        self.refresh_universal_providers()?;
        Ok(())
    }

    /// 刷新代理状态
    pub async fn refresh_proxy_status_async(&mut self) -> Result<()> {
        match self.proxy_service.get_status().await {
            Ok(status) => {
                self.proxy_running = true;
                self.proxy_status = Some(super::ProxyStatusStub {
                    uptime_seconds: status.uptime_seconds,
                    active_connections: status.active_connections as u32,
                    total_requests: status.total_requests,
                    success_rate: status.success_rate as f64,
                });
            }
            Err(_) => {
                self.proxy_running = false;
                self.proxy_status = None;
            }
        }
        Ok(())
    }
}
