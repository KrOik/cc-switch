use serde_json::Value;
use std::collections::HashMap;

/// Provider 配置类型
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderType {
    Claude,      // 使用 env.ANTHROPIC_* 格式
    Codex,       // 使用 auth + config TOML 格式
    OpenCode,    // 使用 npm + options + models 格式
    Generic,     // 通用格式（baseURL + apiKey + models）
}

/// 解析后的 Provider 配置
#[derive(Debug, Clone)]
pub struct ParsedProviderConfig {
    pub provider_type: ProviderType,
    pub base_url: String,
    pub api_key: String,
    pub models: Vec<String>,
    pub api_format: Option<String>,
    pub extra_fields: HashMap<String, Value>,
}

impl ParsedProviderConfig {
    /// 从 settings_config 解析配置
    pub fn from_settings_config(settings_config: &Value, meta: Option<&Value>) -> Self {
        // 检测 Provider 类型
        let provider_type = Self::detect_provider_type(settings_config);

        match provider_type {
            ProviderType::Claude => Self::parse_claude_config(settings_config, meta),
            ProviderType::Codex => Self::parse_codex_config(settings_config, meta),
            ProviderType::OpenCode => Self::parse_opencode_config(settings_config, meta),
            ProviderType::Generic => Self::parse_generic_config(settings_config, meta),
        }
    }

    /// 检测 Provider 类型
    fn detect_provider_type(settings_config: &Value) -> ProviderType {
        // Claude: 有 env.ANTHROPIC_* 字段
        if let Some(env) = settings_config.get("env") {
            if env.get("ANTHROPIC_BASE_URL").is_some()
                || env.get("ANTHROPIC_AUTH_TOKEN").is_some() {
                return ProviderType::Claude;
            }
        }

        // Codex: 有 auth 和 config 字段
        if settings_config.get("auth").is_some()
            && settings_config.get("config").is_some() {
            return ProviderType::Codex;
        }

        // OpenCode: 有 npm 字段
        if settings_config.get("npm").is_some() {
            return ProviderType::OpenCode;
        }

        // 通用格式
        ProviderType::Generic
    }

    /// 解析 Claude 配置
    fn parse_claude_config(settings_config: &Value, meta: Option<&Value>) -> Self {
        let env = settings_config.get("env");

        let base_url = env
            .and_then(|e| e.get("ANTHROPIC_BASE_URL"))
            .and_then(|v| v.as_str())
            .unwrap_or("https://api.anthropic.com")
            .to_string();

        let api_key = env
            .and_then(|e| e.get("ANTHROPIC_AUTH_TOKEN"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // 提取模型列表
        let mut models = Vec::new();
        if let Some(env) = env {
            if let Some(model) = env.get("ANTHROPIC_MODEL").and_then(|v| v.as_str()) {
                models.push(model.to_string());
            }
            if let Some(haiku) = env.get("ANTHROPIC_DEFAULT_HAIKU_MODEL").and_then(|v| v.as_str()) {
                if !models.contains(&haiku.to_string()) {
                    models.push(haiku.to_string());
                }
            }
            if let Some(sonnet) = env.get("ANTHROPIC_DEFAULT_SONNET_MODEL").and_then(|v| v.as_str()) {
                if !models.contains(&sonnet.to_string()) {
                    models.push(sonnet.to_string());
                }
            }
            if let Some(opus) = env.get("ANTHROPIC_DEFAULT_OPUS_MODEL").and_then(|v| v.as_str()) {
                if !models.contains(&opus.to_string()) {
                    models.push(opus.to_string());
                }
            }
        }

        let api_format = meta
            .and_then(|m| m.get("apiFormat"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Self {
            provider_type: ProviderType::Claude,
            base_url,
            api_key,
            models,
            api_format,
            extra_fields: HashMap::new(),
        }
    }

    /// 解析 Codex 配置
    fn parse_codex_config(settings_config: &Value, meta: Option<&Value>) -> Self {
        let api_key = settings_config
            .get("auth")
            .and_then(|a| a.get("OPENAI_API_KEY"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // 从 config TOML 中提取 base_url
        let config_toml = settings_config
            .get("config")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let base_url = Self::extract_base_url_from_toml(config_toml);
        let models = Self::extract_models_from_toml(config_toml);

        let api_format = meta
            .and_then(|m| m.get("apiFormat"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Self {
            provider_type: ProviderType::Codex,
            base_url,
            api_key,
            models,
            api_format,
            extra_fields: HashMap::new(),
        }
    }

    /// 解析 OpenCode 配置
    fn parse_opencode_config(settings_config: &Value, meta: Option<&Value>) -> Self {
        let base_url = settings_config
            .get("options")
            .and_then(|o| o.get("baseURL"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let api_key = settings_config
            .get("options")
            .and_then(|o| o.get("apiKey"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // 提取模型列表
        let mut models = Vec::new();
        if let Some(models_obj) = settings_config.get("models").and_then(|v| v.as_object()) {
            for key in models_obj.keys() {
                models.push(key.clone());
            }
        }

        let api_format = meta
            .and_then(|m| m.get("apiFormat"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Self {
            provider_type: ProviderType::OpenCode,
            base_url,
            api_key,
            models,
            api_format,
            extra_fields: HashMap::new(),
        }
    }

    /// 解析通用配置
    fn parse_generic_config(settings_config: &Value, meta: Option<&Value>) -> Self {
        let base_url = settings_config
            .get("baseURL")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let api_key = settings_config
            .get("apiKey")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut models = Vec::new();
        if let Some(models_array) = settings_config.get("models").and_then(|v| v.as_array()) {
            for model in models_array {
                if let Some(model_str) = model.as_str() {
                    models.push(model_str.to_string());
                }
            }
        }

        let api_format = meta
            .and_then(|m| m.get("apiFormat"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Self {
            provider_type: ProviderType::Generic,
            base_url,
            api_key,
            models,
            api_format,
            extra_fields: HashMap::new(),
        }
    }

    /// 从 TOML 中提取 base_url
    fn extract_base_url_from_toml(toml: &str) -> String {
        for line in toml.lines() {
            let line = line.trim();
            if line.starts_with("base_url") {
                if let Some(url) = line.split('=').nth(1) {
                    return url.trim().trim_matches('"').to_string();
                }
            }
        }
        "".to_string()
    }

    /// 从 TOML 中提取模型列表
    fn extract_models_from_toml(toml: &str) -> Vec<String> {
        let mut models = Vec::new();
        for line in toml.lines() {
            let line = line.trim();
            if line.starts_with("model") && line.contains('=') {
                if let Some(model) = line.split('=').nth(1) {
                    let model = model.trim().trim_matches('"').to_string();
                    if !models.contains(&model) {
                        models.push(model);
                    }
                }
            }
        }
        models
    }

    /// 转换回 settings_config
    pub fn to_settings_config(&self) -> Value {
        match self.provider_type {
            ProviderType::Claude => self.to_claude_settings(),
            ProviderType::Codex => self.to_codex_settings(),
            ProviderType::OpenCode => self.to_opencode_settings(),
            ProviderType::Generic => self.to_generic_settings(),
        }
    }

    /// 转换为 Claude settings
    fn to_claude_settings(&self) -> Value {
        let mut env = serde_json::Map::new();
        env.insert("ANTHROPIC_BASE_URL".to_string(), Value::String(self.base_url.clone()));
        env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), Value::String(self.api_key.clone()));

        if let Some(model) = self.models.first() {
            env.insert("ANTHROPIC_MODEL".to_string(), Value::String(model.clone()));
            env.insert("ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(), Value::String(model.clone()));
            env.insert("ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(), Value::String(model.clone()));
            env.insert("ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(), Value::String(model.clone()));
        }

        serde_json::json!({ "env": env })
    }

    /// 转换为 Codex settings
    fn to_codex_settings(&self) -> Value {
        let model = self.models.first().cloned().unwrap_or_else(|| "gpt-4o".to_string());

        let config_toml = format!(
            r#"model_provider = "newapi"
model = "{}"
model_reasoning_effort = "high"
disable_response_storage = true

[model_providers.newapi]
name = "NewAPI"
base_url = "{}"
wire_api = "responses"
requires_openai_auth = true"#,
            model, self.base_url
        );

        serde_json::json!({
            "auth": {
                "OPENAI_API_KEY": self.api_key
            },
            "config": config_toml
        })
    }

    /// 转换为 OpenCode settings
    fn to_opencode_settings(&self) -> Value {
        let mut models_obj = serde_json::Map::new();
        for model in &self.models {
            models_obj.insert(
                model.clone(),
                serde_json::json!({ "name": model })
            );
        }

        serde_json::json!({
            "npm": "@ai-sdk/openai-compatible",
            "options": {
                "baseURL": self.base_url,
                "apiKey": self.api_key
            },
            "models": models_obj
        })
    }

    /// 转换为通用 settings
    fn to_generic_settings(&self) -> Value {
        serde_json::json!({
            "baseURL": self.base_url,
            "apiKey": self.api_key,
            "models": self.models
        })
    }
}
