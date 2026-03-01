// Core library for CC-Switch
// Provides business logic for AI provider proxy with failover

pub mod database;
pub mod proxy;
pub mod services;
pub mod mcp;
pub mod session_manager;
pub mod deeplink;
pub mod store;

// Configuration and settings
pub mod provider;
pub mod config;
pub mod settings;
pub mod error;
pub mod app_config;

// App-specific configurations
pub mod claude_mcp;
pub mod claude_plugin;
pub mod codex_config;
pub mod gemini_config;
pub mod gemini_mcp;
pub mod openclaw_config;
pub mod opencode_config;

// Utilities
pub mod prompt;
pub mod prompt_files;
pub mod provider_defaults;
pub mod usage_script;
pub mod auto_launch;
pub mod panic_hook;
pub mod init_status;

// Re-export commonly used types
pub use database::Database;
pub use proxy::{ProxyConfig, ProxyStatus, ProxyServerInfo};
pub use services::{
    ConfigService, McpService, OmoService, PromptService, ProviderService, ProxyService,
    SkillService, SpeedtestService,
};
pub use provider::{Provider, ProviderMeta};
pub use settings::AppSettings;
pub use app_config::{AppType, McpApps, McpServer, MultiAppConfig};
pub use error::AppError;

/// Initialize logging for the library
pub fn init_logging(level: log::LevelFilter) {
    env_logger::Builder::new()
        .filter_level(level)
        .init();
}
