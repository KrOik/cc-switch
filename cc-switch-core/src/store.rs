use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};
use serde_json::Value;

use crate::error::AppError;
use crate::database::Database;
use crate::services::ProxyService;
use std::sync::Arc;

/// Application state for TUI version
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub proxy_service: ProxyService,
}

impl AppState {
    pub fn new(db: Arc<Database>) -> Self {
        let proxy_service = ProxyService::new(db.clone());
        Self { db, proxy_service }
    }
}

/// Store key for app config directory override
const STORE_KEY_APP_CONFIG_DIR: &str = "app_config_dir_override";

/// Cache for app_config_dir override path
static APP_CONFIG_DIR_OVERRIDE: OnceLock<RwLock<Option<PathBuf>>> = OnceLock::new();

fn override_cache() -> &'static RwLock<Option<PathBuf>> {
    APP_CONFIG_DIR_OVERRIDE.get_or_init(|| RwLock::new(None))
}

fn update_cached_override(value: Option<PathBuf>) {
    if let Ok(mut guard) = override_cache().write() {
        *guard = value;
    }
}

/// Get cached app_config_dir override path
pub fn get_app_config_dir_override() -> Option<PathBuf> {
    override_cache().read().ok()?.clone()
}

/// Read override from JSON store file
fn read_override_from_store() -> Option<PathBuf> {
    let store_path = dirs::config_dir()?.join("cc-switch").join("app_paths.json");

    if !store_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&store_path).ok()?;
    let store: serde_json::Map<String, Value> = serde_json::from_str(&content).ok()?;

    match store.get(STORE_KEY_APP_CONFIG_DIR) {
        Some(Value::String(path_str)) => {
            let path_str = path_str.trim();
            if path_str.is_empty() {
                return None;
            }

            let path = resolve_path(path_str);

            if !path.exists() {
                log::warn!(
                    "Store 中配置的 app_config_dir 不存在: {path:?}\n\
                     将使用默认路径。"
                );
                return None;
            }

            log::info!("使用 Store 中的 app_config_dir: {path:?}");
            Some(path)
        }
        Some(_) => {
            log::warn!("Store 中的 {STORE_KEY_APP_CONFIG_DIR} 类型不正确，应为字符串");
            None
        }
        None => None,
    }
}

/// Refresh app_config_dir override from store and update cache
pub fn refresh_app_config_dir_override() -> Option<PathBuf> {
    let value = read_override_from_store();
    update_cached_override(value.clone());
    value
}

/// Write app_config_dir to JSON store
pub fn set_app_config_dir_to_store(path: Option<&str>) -> Result<(), AppError> {
    let store_dir = dirs::config_dir()
        .ok_or_else(|| AppError::Message("无法获取配置目录".to_string()))?
        .join("cc-switch");

    std::fs::create_dir_all(&store_dir)
        .map_err(|e| AppError::Message(format!("创建配置目录失败: {e}")))?;

    let store_path = store_dir.join("app_paths.json");

    let mut store: serde_json::Map<String, Value> = if store_path.exists() {
        let content = std::fs::read_to_string(&store_path)
            .map_err(|e| AppError::Message(format!("读取 Store 失败: {e}")))?;
        serde_json::from_str(&content)
            .map_err(|e| AppError::Message(format!("解析 Store 失败: {e}")))?
    } else {
        serde_json::Map::new()
    };

    match path {
        Some(p) => {
            let trimmed = p.trim();
            if !trimmed.is_empty() {
                store.insert(STORE_KEY_APP_CONFIG_DIR.to_string(), Value::String(trimmed.to_string()));
                log::info!("已将 app_config_dir 写入 Store: {trimmed}");
            } else {
                store.remove(STORE_KEY_APP_CONFIG_DIR);
                log::info!("已从 Store 中删除 app_config_dir 配置");
            }
        }
        None => {
            store.remove(STORE_KEY_APP_CONFIG_DIR);
            log::info!("已从 Store 中删除 app_config_dir 配置");
        }
    }

    let content = serde_json::to_string_pretty(&store)
        .map_err(|e| AppError::Message(format!("序列化 Store 失败: {e}")))?;

    std::fs::write(&store_path, content)
        .map_err(|e| AppError::Message(format!("保存 Store 失败: {e}")))?;

    refresh_app_config_dir_override();
    Ok(())
}

/// Resolve path, supporting ~ prefix
fn resolve_path(raw: &str) -> PathBuf {
    if raw == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    } else if let Some(stripped) = raw.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    } else if let Some(stripped) = raw.strip_prefix("~\\") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    }

    PathBuf::from(raw)
}
