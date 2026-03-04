use secrecy::SecretString;

use crate::config::helpers::{optional_env, parse_optional_env};
use crate::error::ConfigError;
use crate::settings::Settings;

/// Which LLM backend to use.
///
/// Defaults to `Ollama` to ensure local processing over external APIs.
/// Users can override with `LLM_BACKEND` env var to use their own API keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LlmBackend {
    /// Local Ollama instance (default)
    #[default]
    Ollama,
    /// Direct OpenAI API
    OpenAi,
    /// Direct Anthropic API
    Anthropic,

    /// Any OpenAI-compatible endpoint (e.g. vLLM, LiteLLM, Together)
    OpenAiCompatible,
    /// Tinfoil private inference
    Tinfoil,
}

impl std::str::FromStr for LlmBackend {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "openai" | "open_ai" => Ok(Self::OpenAi),
            "anthropic" | "claude" => Ok(Self::Anthropic),
            "ollama" => Ok(Self::Ollama),
            "openai_compatible" | "openai-compatible" | "compatible" => Ok(Self::OpenAiCompatible),
            "tinfoil" => Ok(Self::Tinfoil),
            _ => Err(format!(
                "invalid LLM backend '{}', expected one of: openai, anthropic, ollama, openai_compatible, tinfoil",
                s
            )),
        }
    }
}

impl std::fmt::Display for LlmBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenAi => write!(f, "openai"),
            Self::Anthropic => write!(f, "anthropic"),
            Self::Ollama => write!(f, "ollama"),
            Self::OpenAiCompatible => write!(f, "openai_compatible"),
            Self::Tinfoil => write!(f, "tinfoil"),
        }
    }
}

/// Configuration for direct OpenAI API access.
#[derive(Debug, Clone)]
pub struct OpenAiDirectConfig {
    pub api_key: SecretString,
    pub model: String,
}

/// Configuration for direct Anthropic API access.
#[derive(Debug, Clone)]
pub struct AnthropicDirectConfig {
    pub api_key: SecretString,
    pub model: String,
}

/// Configuration for local Ollama.
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model: String,
}

/// Configuration for any OpenAI-compatible endpoint.
///
/// Supports: vLLM, LM Studio, Ollama, LiteLLM, OpenRouter, etc.
#[derive(Debug, Clone)]
pub struct OpenAiCompatibleConfig {
    /// Base URL for the API (e.g., http://localhost:8000/v1)
    pub base_url: String,
    /// API key (optional for local servers)
    pub api_key: Option<SecretString>,
    /// Model name to use
    pub model: String,
}

/// Configuration for Tinfoil private inference.
#[derive(Debug, Clone)]
pub struct TinfoilConfig {
    pub api_key: SecretString,
    pub model: String,
}

/// LLM provider configuration.
///
/// Default backend is Ollama. Users can switch to other providers
/// by setting `LLM_BACKEND` (e.g. `openai`, `anthropic`, `ollama`, `openai_compatible`).
#[derive(Debug, Clone)]
pub struct LlmConfig {
    /// Which backend to use (default: Ollama)
    pub backend: LlmBackend,
    /// Direct OpenAI config (populated when backend=openai)
    pub openai: Option<OpenAiDirectConfig>,
    /// Direct Anthropic config (populated when backend=anthropic)
    pub anthropic: Option<AnthropicDirectConfig>,
    /// Ollama config (populated when backend=ollama)
    pub ollama: Option<OllamaConfig>,
    /// OpenAI-compatible config (populated when backend=openai_compatible)
    pub openai_compatible: Option<OpenAiCompatibleConfig>,
    /// Tinfoil config (populated when backend=tinfoil)
    pub tinfoil: Option<TinfoilConfig>,
}

impl LlmConfig {
    pub(crate) fn resolve(settings: &Settings) -> Result<Self, ConfigError> {
        // Determine backend: env var > settings > default (Ollama)
        let backend: LlmBackend = if let Some(b) = optional_env("LLM_BACKEND")? {
            b.parse().map_err(|e| ConfigError::InvalidValue {
                key: "LLM_BACKEND".to_string(),
                message: e,
            })?
        } else if let Some(ref b) = settings.llm_backend {
            match b.parse() {
                Ok(backend) => backend,
                Err(e) => {
                    tracing::warn!(
                        "Invalid llm_backend '{}' in settings: {}. Using default Ollama.",
                        b,
                        e
                    );
                    LlmBackend::Ollama
                }
            }
        } else {
            LlmBackend::Ollama
        };

        // Resolve config for each backend
        let openai = if backend == LlmBackend::OpenAi {
            let api_key = optional_env("OPENAI_API_KEY")?
                .map(SecretString::from)
                .ok_or_else(|| ConfigError::MissingRequired {
                    key: "OPENAI_API_KEY".to_string(),
                    hint: "Set OPENAI_API_KEY when LLM_BACKEND=openai".to_string(),
                })?;
            let model = optional_env("OPENAI_MODEL")?
                .or_else(|| settings.selected_model.clone())
                .unwrap_or_else(|| "gpt-4o".to_string());
            Some(OpenAiDirectConfig { api_key, model })
        } else {
            None
        };

        let anthropic = if backend == LlmBackend::Anthropic {
            let api_key = optional_env("ANTHROPIC_API_KEY")?
                .map(SecretString::from)
                .ok_or_else(|| ConfigError::MissingRequired {
                    key: "ANTHROPIC_API_KEY".to_string(),
                    hint: "Set ANTHROPIC_API_KEY when LLM_BACKEND=anthropic".to_string(),
                })?;
            let model = optional_env("ANTHROPIC_MODEL")?
                .or_else(|| settings.selected_model.clone())
                .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());
            Some(AnthropicDirectConfig { api_key, model })
        } else {
            None
        };

        let ollama = if backend == LlmBackend::Ollama {
            let base_url = optional_env("OLLAMA_BASE_URL")?
                .unwrap_or_else(|| "http://localhost:11434".to_string());
            let model = optional_env("OLLAMA_MODEL")?
                .or_else(|| settings.selected_model.clone())
                .unwrap_or_else(|| "llama3".to_string());
            Some(OllamaConfig { base_url, model })
        } else {
            None
        };

        let openai_compatible = if backend == LlmBackend::OpenAiCompatible {
            let base_url = optional_env("OPENAI_COMPATIBLE_BASE_URL")?
                .ok_or_else(|| ConfigError::MissingRequired {
                    key: "OPENAI_COMPATIBLE_BASE_URL".to_string(),
                    hint: "Set OPENAI_COMPATIBLE_BASE_URL when LLM_BACKEND=openai_compatible".to_string(),
                })?;
            let api_key = optional_env("OPENAI_COMPATIBLE_API_KEY")?.map(SecretString::from);
            let model = optional_env("OPENAI_COMPATIBLE_MODEL")?
                .or_else(|| settings.selected_model.clone())
                .unwrap_or_else(|| "gpt-4o".to_string());
            Some(OpenAiCompatibleConfig {
                base_url,
                api_key,
                model,
            })
        } else {
            None
        };

        let tinfoil = if backend == LlmBackend::Tinfoil {
            let api_key = optional_env("TINFOIL_API_KEY")?
                .map(SecretString::from)
                .ok_or_else(|| ConfigError::MissingRequired {
                    key: "TINFOIL_API_KEY".to_string(),
                    hint: "Set TINFOIL_API_KEY when LLM_BACKEND=tinfoil".to_string(),
                })?;
            let model = optional_env("TINFOIL_MODEL")?.unwrap_or_else(|| "kimi-k2-5".to_string());
            Some(TinfoilConfig { api_key, model })
        } else {
            None
        };

        Ok(Self {
            backend,
            openai,
            anthropic,
            ollama,
            openai_compatible,
            tinfoil,
        })
    }
}

fn normalize_openai_compatible_base_url(url: &str) -> String {
    let trimmed = url.trim().trim_end_matches('/');
    trimmed
        .strip_suffix("/chat/completions")
        .or_else(|| trimmed.strip_suffix("/completions"))
        .unwrap_or(trimmed)
        .to_string()
}

fn is_openrouter_base_url(url: &str) -> bool {
    normalize_openai_compatible_base_url(url)
        .to_lowercase()
        .contains("openrouter.ai/api/v1")
}
