use std::env;

/// Runtime configuration loaded from .env and settings panel.
#[derive(Clone, Debug)]
pub struct AppConfig {
    pub ch_api_key: Option<String>,
    pub ai_provider: AiProviderConfig,
    pub kanon2_api_key: Option<String>,
    pub ocr_mode: OcrMode,
}

#[derive(Clone, Debug)]
pub enum AiProviderConfig {
    Anthropic { api_key: String, model: String },
    OpenAi { api_key: String, model: String },
    Custom { api_key: String, model: String, base_url: String },
    None,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OcrMode {
    /// Only attempt native PDF text extraction
    NativeOnly,
    /// Native + ocrs ML fallback
    WithOcrs,
    /// Native + Docling sidecar fallback
    WithDocling,
}

impl Default for OcrMode {
    fn default() -> Self {
        Self::NativeOnly
    }
}

impl AppConfig {
    pub fn load_from_env() -> Self {
        // Load AI provider from env if available (injected by sandbox container)
        let ai_provider = if let Ok(key) = env::var("ANTHROPIC_API_KEY") {
            if !key.is_empty() {
                AiProviderConfig::Anthropic {
                    api_key: key,
                    model: env::var("ANTHROPIC_MODEL")
                        .unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string()),
                }
            } else {
                AiProviderConfig::None
            }
        } else if let Ok(key) = env::var("OPENAI_API_KEY") {
            if !key.is_empty() {
                AiProviderConfig::OpenAi {
                    api_key: key,
                    model: env::var("OPENAI_MODEL")
                        .unwrap_or_else(|_| "gpt-4o".to_string()),
                }
            } else {
                AiProviderConfig::None
            }
        } else {
            AiProviderConfig::None
        };

        Self {
            ch_api_key: env::var("CH_API_KEY").ok(),
            ai_provider,
            kanon2_api_key: env::var("KANON2_API_KEY").ok(),
            ocr_mode: OcrMode::default(),
        }
    }

    pub fn has_ch_key(&self) -> bool {
        self.ch_api_key.as_ref().map_or(false, |k| !k.is_empty())
    }

    pub fn has_ai(&self) -> bool {
        !matches!(self.ai_provider, AiProviderConfig::None)
    }
}
