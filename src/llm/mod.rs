//! LLM integration for the agent.
//!
//! Supports multiple backends:
//! - **Ollama** (default): Local model inference - RECOMMENDED
//! - **OpenAI**: Direct API access with your own key
//! - **Anthropic**: Direct API access with your own key
//! - **OpenAI-compatible**: Any endpoint that speaks the OpenAI API

pub mod circuit_breaker;
pub mod costs;
pub mod failover;
mod provider;
mod reasoning;
pub mod response_cache;
mod retry;
mod rig_adapter;
pub mod session;

pub use circuit_breaker::{CircuitBreakerConfig, CircuitBreakerProvider};
pub use failover::{CooldownConfig, FailoverProvider};
pub use provider::{
    ChatMessage, CompletionRequest, CompletionResponse, FinishReason, LlmProvider, ModelMetadata,
    Role, ToolCall, ToolCompletionRequest, ToolCompletionResponse, ToolDefinition, ToolResult,
};
pub use reasoning::{
    ActionPlan, Reasoning, ReasoningContext, RespondOutput, RespondResult, TokenUsage,
    ToolSelection,
};
pub use response_cache::{CachedProvider, ResponseCacheConfig};
pub use rig_adapter::RigAdapter;
pub use session::{SessionConfig, SessionManager, create_session_manager};

use std::sync::Arc;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use rig::client::CompletionClient;
use secrecy::ExposeSecret;

use crate::config::{LlmBackend, LlmConfig};
use crate::error::LlmError;

/// Create an LLM provider based on configuration.
pub fn create_llm_provider(
    config: &LlmConfig,
    _session: Arc<SessionManager>,
) -> Result<Arc<dyn LlmProvider>, LlmError> {
    match config.backend {
        LlmBackend::NearAi => {
            // Near AI support has been removed
            Err(LlmError::AuthFailed {
                provider: "nearai".to_string(),
                reason: "NEAR AI backend has been removed. Please use Ollama, OpenAI, Anthropic, or OpenAI-compatible instead.".to_string(),
            })
        }
        LlmBackend::OpenAi => create_openai_provider(config),
        LlmBackend::Anthropic => create_anthropic_provider(config),
        LlmBackend::Ollama => create_ollama_provider(config),
        LlmBackend::OpenAiCompatible => create_openai_compatible_provider(config),
        LlmBackend::Tinfoil => create_tinfoil_provider(config),
    }
}

fn create_openai_provider(config: &LlmConfig) -> Result<Arc<dyn LlmProvider>, LlmError> {
    let oai = config.openai.as_ref().ok_or_else(|| LlmError::AuthFailed {
        provider: "openai".to_string(),
    })?;

    use rig::providers::openai;

    // Use CompletionsClient (Chat Completions API) instead of the default Client
    // (Responses API). The Responses API path in rig-core panics when tool results
    // are sent back because ironclaw doesn't thread `call_id` through its ToolCall
    // type. The Chat Completions API works correctly with the existing code.
    let client: openai::CompletionsClient = openai::Client::new(oai.api_key.expose_secret())
        .map_err(|e| LlmError::RequestFailed {
            provider: "openai".to_string(),
            reason: format!("Failed to create OpenAI client: {}", e),
        })?
        .completions_api();

    let model = client.completion_model(&oai.model);
    tracing::info!("Using OpenAI direct API (model: {})", oai.model);
    Ok(Arc::new(RigAdapter::new(model, &oai.model)))
}

fn create_anthropic_provider(config: &LlmConfig) -> Result<Arc<dyn LlmProvider>, LlmError> {
    let anth = config
        .anthropic
        .as_ref()
        .ok_or_else(|| LlmError::AuthFailed {
            provider: "anthropic".to_string(),
        })?;

    use rig::providers::anthropic;

    let client: anthropic::Client =
        anthropic::Client::new(anth.api_key.expose_secret()).map_err(|e| {
            LlmError::RequestFailed {
                provider: "anthropic".to_string(),
                reason: format!("Failed to create Anthropic client: {}", e),
            }
        })?;

    let model = client.completion_model(&anth.model);
    tracing::info!("Using Anthropic direct API (model: {})", anth.model);
    Ok(Arc::new(RigAdapter::new(model, &anth.model)))
}

fn create_ollama_provider(config: &LlmConfig) -> Result<Arc<dyn LlmProvider>, LlmError> {
    let oll = config.ollama.as_ref().ok_or_else(|| LlmError::AuthFailed {
        provider: "ollama".to_string(),
    })?;

    use rig::client::Nothing;
    use rig::providers::ollama;

    let client: ollama::Client = ollama::Client::builder()
        .base_url(&oll.base_url)
        .api_key(Nothing)
        .build()
        .map_err(|e| LlmError::RequestFailed {
            provider: "ollama".to_string(),
            reason: format!("Failed to create Ollama client: {}", e),
        })?;

    let model = client.completion_model(&oll.model);
    tracing::info!(
        "Using Ollama (base_url: {}, model: {})",
        oll.base_url,
        oll.model
    );
    Ok(Arc::new(RigAdapter::new(model, &oll.model)))
}

const TINFOIL_BASE_URL: &str = "https://inference.tinfoil.sh/v1";

fn create_tinfoil_provider(config: &LlmConfig) -> Result<Arc<dyn LlmProvider>, LlmError> {
    let tf = config
        .tinfoil
        .as_ref()
        .ok_or_else(|| LlmError::AuthFailed {
            provider: "tinfoil".to_string(),
        })?;

    use rig::providers::openai;

    let client: openai::Client = openai::Client::builder()
        .base_url(TINFOIL_BASE_URL)
        .api_key(tf.api_key.expose_secret())
        .build()
        .map_err(|e| LlmError::RequestFailed {
            provider: "tinfoil".to_string(),
            reason: format!("Failed to create Tinfoil client: {}", e),
        })?;

    // Tinfoil currently only supports the Chat Completions API and not the newer Responses API,
    // so we must explicitly select the completions API here (unlike other OpenAI-compatible providers).
    let client = client.completions_api();
    let model = client.completion_model(&tf.model);
    tracing::info!("Using Tinfoil private inference (model: {})", tf.model);
    Ok(Arc::new(RigAdapter::new(model, &tf.model)))
}

fn create_openai_compatible_provider(config: &LlmConfig) -> Result<Arc<dyn LlmProvider>, LlmError> {
    let compat = config
        .openai_compatible
        .as_ref()
        .ok_or_else(|| LlmError::AuthFailed {
            provider: "openai_compatible".to_string(),
        })?;

    use rig::providers::openai;

    let api_key = compat
        .api_key
        .as_ref()
        .map(|k| k.expose_secret().to_string());
    if is_openrouter_base_url(&compat.base_url) && api_key.is_none() {
        return Err(LlmError::AuthFailed {
            provider: "openai_compatible".to_string(),
        });
    }
    let api_key = api_key.unwrap_or_else(|| "no-key".to_string());

    let mut builder = openai::Client::builder()
        .base_url(&compat.base_url)
        .api_key(api_key);

    // OpenRouter app attribution headers so requests show the correct app
    // identity instead of "unknown" in OpenRouter logs/analytics.
    if is_openrouter_base_url(&compat.base_url) {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("x-title"),
            HeaderValue::from_static("TitanClaw"),
        );
        if let Ok(referer) = HeaderValue::from_str(openrouter_app_referer()) {
            headers.insert(HeaderName::from_static("http-referer"), referer);
        }
        builder = builder.http_headers(headers);
    }

    let client: openai::Client = builder.build().map_err(|e| LlmError::RequestFailed {
        provider: "openai_compatible".to_string(),
        reason: format!("Failed to create OpenAI-compatible client: {}", e),
    })?;

    // OpenAI-compatible providers (e.g. OpenRouter) are most reliable on Chat Completions.
    // This avoids Responses-API-specific assumptions such as required tool call IDs.
    let model = client.completions_api().completion_model(&compat.model);
    tracing::info!(
        "Using OpenAI-compatible endpoint via Chat Completions API (base_url: {}, model: {})",
        compat.base_url,
        compat.model
    );
    Ok(Arc::new(RigAdapter::new(model, &compat.model)))
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

fn openrouter_app_referer() -> &'static str {
    let repo = env!("CARGO_PKG_REPOSITORY");
    if repo.is_empty() {
        "https://github.com/PhantomReaper2025/titanclaw"
    } else {
        repo
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{LlmBackend, LlmConfig};

    fn test_llm_config() -> LlmConfig {
        LlmConfig {
            backend: LlmBackend::Ollama,
            openai: None,
            anthropic: None,
            ollama: None,
            openai_compatible: None,
            tinfoil: None,
        }
    }
}
