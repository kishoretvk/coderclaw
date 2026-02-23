//! Setup API handlers for web-based onboarding.
//!
//! These endpoints allow users to configure their LLM provider and database
//! through a web interface instead of the CLI wizard.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use serde::{Deserialize, Serialize};

use crate::channels::web::server::GatewayState;

/// Response for setup status endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct SetupStatusResponse {
    pub completed: bool,
    pub database_backend: Option<String>,
    pub llm_backend: Option<String>,
    pub selected_model: Option<String>,
    pub gateway_port: Option<u16>,
}

/// Model info for dropdown selection
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
}

/// Request for fetching models
#[derive(Debug, Deserialize)]
pub struct ModelsQuery {
    pub provider: String,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
}

/// Response for models endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelsResponse {
    pub models: Vec<ModelInfo>,
}

/// Validation error
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

/// Response for validation endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateResponse {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

/// Request body for saving setup
#[derive(Debug, Deserialize)]
pub struct SetupSaveRequest {
    // Database
    #[serde(default)]
    pub database_backend: Option<String>,
    #[serde(default)]
    pub database_url: Option<String>,
    #[serde(default)]
    pub libsql_path: Option<String>,
    #[serde(default)]
    pub libsql_url: Option<String>,
    
    // LLM
    #[serde(default)]
    pub llm_backend: Option<String>,
    #[serde(default)]
    pub ollama_base_url: Option<String>,
    #[serde(default)]
    pub openai_api_key: Option<String>,
    #[serde(default)]
    pub anthropic_api_key: Option<String>,
    #[serde(default)]
    pub openai_compatible_base_url: Option<String>,
    #[serde(default)]
    pub openai_compatible_api_key: Option<String>,
    #[serde(default)]
    pub selected_model: Option<String>,
    
    // Embeddings
    #[serde(default)]
    pub embeddings_enabled: Option<bool>,
    #[serde(default)]
    pub embeddings_provider: Option<String>,
    #[serde(default)]
    pub embeddings_model: Option<String>,
    
    // Gateway
    #[serde(default)]
    pub gateway_port: Option<u16>,
    #[serde(default)]
    pub gateway_auth_token: Option<String>,
    
    // Security
    #[serde(default)]
    pub secrets_master_key: Option<String>,
}

/// Response for save endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct SetupSaveResponse {
    pub success: bool,
    pub token: Option<String>,
    pub redirect_url: Option<String>,
    pub error: Option<String>,
}

/// Get the current setup status
pub async fn setup_status_handler(
    State(state): State<Arc<GatewayState>>,
) -> Result<Json<SetupStatusResponse>, StatusCode> {
    let store = state
        .store
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    // Get key settings
    let onboard_completed = store
        .get_setting("default", "onboard_completed")
        .await
        .map_err(|e| {
            tracing::error!("Failed to get onboard_completed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .map(|r| r.value == "true")
        .unwrap_or(false);
    
    let database_backend = store
        .get_setting("default", "database_backend")
        .await
        .ok()
        .flatten()
        .map(|r| r.value);
    
    let llm_backend = store
        .get_setting("default", "llm_backend")
        .await
        .ok()
        .flatten()
        .map(|r| r.value);
    
    let selected_model = store
        .get_setting("default", "selected_model")
        .await
        .ok()
        .flatten()
        .map(|r| r.value);
    
    let gateway_port = store
        .get_setting("default", "gateway_port")
        .await
        .ok()
        .flatten()
        .and_then(|r| r.value.parse().ok());
    
    Ok(Json(SetupStatusResponse {
        completed: onboard_completed,
        database_backend,
        llm_backend,
        selected_model,
        gateway_port,
    }))
}

/// Fetch available models for a provider
pub async fn setup_models_handler(
    Query(query): Query<ModelsQuery>,
) -> Result<Json<ModelsResponse>, StatusCode> {
    let models = match query.provider.as_str() {
        "ollama" => {
            let base_url = query.base_url.as_deref().unwrap_or("http://localhost:11434");
            fetch_ollama_models(base_url).await
        }
        "openai" => {
            // For OpenAI, we'd need the API key to fetch models
            // For now, return common models
            get_openai_models()
        }
        "anthropic" => {
            // Anthropic doesn't have a public model listing API
            get_anthropic_models()
        }
        "openai_compatible" => {
            // Try to fetch from the endpoint, or return error
            if let Some(base_url) = &query.base_url {
                fetch_openai_compatible_models(base_url).await
            } else {
                Err(StatusCode::BAD_REQUEST)
            }
        }
        _ => Err(StatusCode::BAD_REQUEST),
    };
    
    match models {
        Ok(models) => Ok(Json(ModelsResponse { models })),
        Err(code) => Err(code),
    }
}

/// Validate configuration before saving
pub async fn setup_validate_handler(
    State(state): State<Arc<GatewayState>>,
    Json(req): Json<SetupSaveRequest>,
) -> Result<Json<ValidateResponse>, StatusCode> {
    let mut errors = Vec::new();
    
    // Validate database backend
    if let Some(ref backend) = req.database_backend {
        if backend != "postgres" && backend != "libsql" {
            errors.push(ValidationError {
                field: "database_backend".to_string(),
                message: "Must be 'postgres' or 'libsql'".to_string(),
            });
        }
        
        if backend == "postgres" && req.database_url.is_none() {
            errors.push(ValidationError {
                field: "database_url".to_string(),
                message: "Required for PostgreSQL".to_string(),
            });
        }
    }
    
    // Validate LLM backend
    if let Some(ref backend) = req.llm_backend {
        let valid_backends = ["ollama", "openai", "anthropic", "openai_compatible"];
        if !valid_backends.contains(&backend.as_str()) {
            errors.push(ValidationError {
                field: "llm_backend".to_string(),
                message: format!("Must be one of: {}", valid_backends.join(", ")),
            });
        }
        
        // Validate provider-specific requirements
        match backend.as_str() {
            "openai" => {
                if req.openai_api_key.is_none() || req.openai_api_key.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                    errors.push(ValidationError {
                        field: "openai_api_key".to_string(),
                        message: "Required for OpenAI".to_string(),
                    });
                }
            }
            "anthropic" => {
                if req.anthropic_api_key.is_none() || req.anthropic_api_key.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                    errors.push(ValidationError {
                        field: "anthropic_api_key".to_string(),
                        message: "Required for Anthropic".to_string(),
                    });
                }
            }
            "openai_compatible" => {
                if req.openai_compatible_base_url.is_none() {
                    errors.push(ValidationError {
                        field: "openai_compatible_base_url".to_string(),
                        message: "Required for OpenAI-compatible endpoint".to_string(),
                    });
                }
            }
            _ => {}
        }
    }
    
    // Validate model selection
    if req.selected_model.is_none() || req.selected_model.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
        errors.push(ValidationError {
            field: "selected_model".to_string(),
            message: "Please select a model".to_string(),
        });
    }
    
    // Validate secrets master key (min length)
    if let Some(ref key) = req.secrets_master_key {
        if key.len() < 32 {
            errors.push(ValidationError {
                field: "secrets_master_key".to_string(),
                message: "Must be at least 32 characters".to_string(),
            });
        }
    }
    
    Ok(Json(ValidateResponse {
        valid: errors.is_empty(),
        errors,
    }))
}

/// Save the setup configuration
pub async fn setup_save_handler(
    State(state): State<Arc<GatewayState>>,
    Json(req): Json<SetupSaveRequest>,
) -> Result<Json<SetupSaveResponse>, StatusCode> {
    let store = state
        .store
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    
    let user_id = "default";
    
    // Save database settings
    if let Some(ref backend) = req.database_backend {
        if let Err(e) = store.set_setting(user_id, "database_backend", backend).await {
            return Ok(Json(SetupSaveResponse {
                success: false,
                token: None,
                redirect_url: None,
                error: Some(format!("Failed to save database_backend: {}", e)),
            }));
        }
    }
    
    if let Some(ref url) = req.database_url {
        if let Err(e) = store.set_setting(user_id, "database_url", url).await {
            return Ok(Json(SetupSaveResponse {
                success: false,
                token: None,
                redirect_url: None,
                error: Some(format!("Failed to save database_url: {}", e)),
            }));
        }
    }
    
    if let Some(ref path) = req.libsql_path {
        if let Err(e) = store.set_setting(user_id, "libsql_path", path).await {
            return Ok(Json(SetupSaveResponse {
                success: false,
                token: None,
                redirect_url: None,
                error: Some(format!("Failed to save libsql_path: {}", e)),
            }));
        }
    }
    
    // Save LLM settings
    if let Some(ref backend) = req.llm_backend {
        if let Err(e) = store.set_setting(user_id, "llm_backend", backend).await {
            return Ok(Json(SetupSaveResponse {
                success: false,
                token: None,
                redirect_url: None,
                error: Some(format!("Failed to save llm_backend: {}", e)),
            }));
        }
    }
    
    if let Some(ref url) = req.ollama_base_url {
        if let Err(e) = store.set_setting(user_id, "ollama_base_url", url).await {
            return Ok(Json(SetupSaveResponse {
                success: false,
                token: None,
                redirect_url: None,
                error: Some(format!("Failed to save ollama_base_url: {}", e)),
            }));
        }
    }
    
    if let Some(ref key) = req.openai_api_key {
        if !key.is_empty() {
            if let Err(e) = store.set_setting(user_id, "openai_api_key", key).await {
                return Ok(Json(SetupSaveResponse {
                    success: false,
                    token: None,
                    redirect_url: None,
                    error: Some(format!("Failed to save openai_api_key: {}", e)),
                }));
            }
        }
    }
    
    if let Some(ref key) = req.anthropic_api_key {
        if !key.is_empty() {
            if let Err(e) = store.set_setting(user_id, "anthropic_api_key", key).await {
                return Ok(Json(SetupSaveResponse {
                    success: false,
                    token: None,
                    redirect_url: None,
                    error: Some(format!("Failed to save anthropic_api_key: {}", e)),
                }));
            }
        }
    }
    
    if let Some(ref url) = req.openai_compatible_base_url {
        if let Err(e) = store.set_setting(user_id, "openai_compatible_base_url", url).await {
            return Ok(Json(SetupSaveResponse {
                success: false,
                token: None,
                redirect_url: None,
                error: Some(format!("Failed to save openai_compatible_base_url: {}", e)),
            }));
        }
    }
    
    if let Some(ref key) = req.openai_compatible_api_key {
        if !key.is_empty() {
            if let Err(e) = store.set_setting(user_id, "openai_compatible_api_key", key).await {
                return Ok(Json(SetupSaveResponse {
                    success: false,
                    token: None,
                    redirect_url: None,
                    error: Some(format!("Failed to save openai_compatible_api_key: {}", e)),
                }));
            }
        }
    }
    
    if let Some(ref model) = req.selected_model {
        if let Err(e) = store.set_setting(user_id, "selected_model", model).await {
            return Ok(Json(SetupSaveResponse {
                success: false,
                token: None,
                redirect_url: None,
                error: Some(format!("Failed to save selected_model: {}", e)),
            }));
        }
    }
    
    // Save gateway settings
    if let Some(port) = req.gateway_port {
        if let Err(e) = store.set_setting(user_id, "gateway_port", &port.to_string()).await {
            return Ok(Json(SetupSaveResponse {
                success: false,
                token: None,
                redirect_url: None,
                error: Some(format!("Failed to save gateway_port: {}", e)),
            }));
        }
    }
    
    // Generate or use provided auth token
    let token = req.gateway_auth_token.unwrap_or_else(|| {
        use uuid::Uuid;
        Uuid::new_v4().to_string()
    });
    
    if let Err(e) = store.set_setting(user_id, "gateway_auth_token", &token).await {
        return Ok(Json(SetupSaveResponse {
            success: false,
            token: None,
            redirect_url: None,
            error: Some(format!("Failed to save gateway_auth_token: {}", e)),
        }));
    }
    
    // Mark onboarding as complete
    if let Err(e) = store.set_setting(user_id, "onboard_completed", "true").await {
        return Ok(Json(SetupSaveResponse {
            success: false,
            token: None,
            redirect_url: None,
            error: Some(format!("Failed to save onboard_completed: {}", e)),
        }));
    }
    
    // Return success with token
    Ok(Json(SetupSaveResponse {
        success: true,
        token: Some(token),
        redirect_url: Some(format!("/?token={}", token)),
        error: None,
    }))
}

// Helper functions for fetching models

async fn fetch_ollama_models(base_url: &str) -> Result<Vec<ModelInfo>, StatusCode> {
    // Try to fetch from Ollama API
    let client = reqwest::Client::new();
    let url = format!("{}/api/tags", base_url.trim_end_matches('/'));
    
    match client.get(&url).timeout(std::time::Duration::from_secs(5)).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                #[derive(Deserialize)]
                struct OllamaResponse {
                    models: Vec<OllamaModel>,
                }
                #[derive(Deserialize)]
                struct OllamaModel {
                    name: String,
                }
                
                if let Ok(data) = resp.json::<OllamaResponse>().await {
                    let models = data.models.into_iter().map(|m| {
                        // Extract model name without tags (e.g., "llama3:latest" -> "llama3")
                        let id = m.name.split(':').next().unwrap_or(&m.name).to_string();
                        ModelInfo {
                            id: id.clone(),
                            name: m.name,
                            provider: "ollama".to_string(),
                        }
                    }).collect();
                    
                    return Ok(models);
                }
            }
            // Fallback to default models
            Ok(get_default_ollama_models())
        }
        Err(_) => {
            // Fallback to default models if Ollama is not available
            Ok(get_default_ollama_models())
        }
    }
}

fn get_default_ollama_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo { id: "llama3".to_string(), name: "Llama 3".to_string(), provider: "ollama".to_string() },
        ModelInfo { id: "llama3.1".to_string(), name: "Llama 3.1".to_string(), provider: "ollama".to_string() },
        ModelInfo { id: "mistral".to_string(), name: "Mistral".to_string(), provider: "ollama".to_string() },
        ModelInfo { id: "codellama".to_string(), name: "CodeLlama".to_string(), provider: "ollama".to_string() },
        ModelInfo { id: "phi3".to_string(), name: "Phi-3".to_string(), provider: "ollama".to_string() },
        ModelInfo { id: "qwen2".to_string(), name: "Qwen 2".to_string(), provider: "ollama".to_string() },
    ]
}

fn get_openai_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo { id: "gpt-4o".to_string(), name: "GPT-4o".to_string(), provider: "openai".to_string() },
        ModelInfo { id: "gpt-4o-mini".to_string(), name: "GPT-4o Mini".to_string(), provider: "openai".to_string() },
        ModelInfo { id: "gpt-4-turbo".to_string(), name: "GPT-4 Turbo".to_string(), provider: "openai".to_string() },
        ModelInfo { id: "gpt-4".to_string(), name: "GPT-4".to_string(), provider: "openai".to_string() },
        ModelInfo { id: "gpt-3.5-turbo".to_string(), name: "GPT-3.5 Turbo".to_string(), provider: "openai".to_string() },
        ModelInfo { id: "o1".to_string(), name: "O1 (Reasoning)".to_string(), provider: "openai".to_string() },
        ModelInfo { id: "o1-mini".to_string(), name: "O1 Mini (Reasoning)".to_string(), provider: "openai".to_string() },
    ]
}

fn get_anthropic_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo { id: "claude-sonnet-4-20250514".to_string(), name: "Claude Sonnet 4".to_string(), provider: "anthropic".to_string() },
        ModelInfo { id: "claude-3-5-sonnet-20241022".to_string(), name: "Claude 3.5 Sonnet".to_string(), provider: "anthropic".to_string() },
        ModelInfo { id: "claude-3-5-haiku-20241022".to_string(), name: "Claude 3.5 Haiku".to_string(), provider: "anthropic".to_string() },
        ModelInfo { id: "claude-3-opus-20240229".to_string(), name: "Claude 3 Opus".to_string(), provider: "anthropic".to_string() },
        ModelInfo { id: "claude-3-sonnet-20240229".to_string(), name: "Claude 3 Sonnet".to_string(), provider: "anthropic".to_string() },
        ModelInfo { id: "claude-3-haiku-20240307".to_string(), name: "Claude 3 Haiku".to_string(), provider: "anthropic".to_string() },
    ]
}

async fn fetch_openai_compatible_models(base_url: &str) -> Result<Vec<ModelInfo>, StatusCode> {
    // Try to fetch models from the compatible endpoint
    let client = reqwest::Client::new();
    let url = format!("{}/models", base_url.trim_end_matches('/'));
    
    match client.get(&url).timeout(std::time::Duration::from_secs(5)).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                #[derive(Deserialize)]
                struct CompatibleResponse {
                    data: Vec<CompatibleModel>,
                }
                #[derive(Deserialize)]
                struct CompatibleModel {
                    id: String,
                }
                
                if let Ok(data) = resp.json::<CompatibleResponse>().await {
                    let models = data.data.into_iter().map(|m| {
                        ModelInfo {
                            id: m.id.clone(),
                            name: m.id,
                            provider: "openai_compatible".to_string(),
                        }
                    }).collect();
                    
                    return Ok(models);
                }
            }
            Err(StatusCode::BAD_REQUEST)
        }
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

/// Serve the setup page HTML
pub async fn setup_page_handler() -> Html<&'static str> {
    Html(include_str!("../../static/setup.html"))
}

/// Serve the setup page JavaScript
pub async fn setup_js_handler() -> impl IntoResponse {
    (
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("application/javascript"),
        include_str!("../../static/setup.js"),
    )
}
