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
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_from_env() -> Self {
        Self {
            ch_api_key: std::env::var("CH_API_KEY").ok(),
            ai_provider: AiProviderConfig::None,
            kanon2_api_key: None,
            ocr_mode: OcrMode::default(),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load_from_env() -> Self {
        // In WASM, environment variables are not available.
        // All configuration is set via the settings panel.
        Self {
            ch_api_key: None,
            ai_provider: AiProviderConfig::None,
            kanon2_api_key: None,
            ocr_mode: OcrMode::NativeOnly,
        }
    }

    pub fn has_ch_key(&self) -> bool {
        self.ch_api_key.as_ref().map_or(false, |k| !k.is_empty())
    }

    pub fn has_ai(&self) -> bool {
        !matches!(self.ai_provider, AiProviderConfig::None)
    }
}
