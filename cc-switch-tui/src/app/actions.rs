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

    /// 删除 Provider
    pub async fn delete_provider(&mut self, provider_id: &str) -> Result<()> {
        log::info!("Deleting provider: {}", provider_id);

        // TODO: 实现实际的删除逻辑
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
