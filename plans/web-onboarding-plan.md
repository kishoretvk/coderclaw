# Web-Based Onboarding Landing Page - Production Implementation Plan

## Overview
Add a web-based onboarding/landing page that allows users to configure their LLM backend and database before accessing the main application. This provides a user-friendly alternative to the current CLI wizard.

## Target Deployment Scenarios

| Scenario | Database | Network Access | Users |
|----------|----------|----------------|-------|
| **Docker Production** | PostgreSQL (external or docker-compose) | External/Cloud | Multiple |
| **Docker Development** | PostgreSQL (docker-compose) | Localhost | Single |
| **Local Production** | libSQL/SQLite | Localhost | Single |
| **Local Development** | libSQL/SQLite | Localhost | Single |

---

## Architecture

### Route Structure

```
Public Routes (No Auth Required):
├── /                     → Redirect to /setup if not onboarded, else main app
├── /setup               → Onboarding wizard page (NEW)
├── /api/health          → Health check
├── /api/setup/status    → Get onboarding status (NEW)
├── /api/setup/models    → Fetch available models (NEW)
├── /api/setup/validate → Validate configuration (NEW)
└── /api/setup/save     → Save configuration (NEW)

Protected Routes (Auth Required):
├── /api/*              → All existing API endpoints
├── /app.js             → Main application JS
├── /style.css          → Styles
└── /ws                 → WebSocket for real-time
```

### Data Flow

```mermaid
sequenceDiagram
    participant User
    participant Browser
    participant Gateway
    participant Config
    participant Database
    participant Ollama/OpenAI

    Browser->>Gateway: GET /
    Gateway->>Database: Check onboard_completed
    Database-->>Gateway: false
    Gateway->>Browser: Redirect to /setup

    Browser->>Gateway: GET /setup
    Gateway-->>Browser: Serve setup HTML

    Note over Browser: User selects options
    
    Browser->>Gateway: POST /api/setup/save
    Gateway->>Config: Save all settings
    Config->>Database: Store in settings table
    Database-->>Gateway: Success
    Gateway-->>Browser: Redirect to /?token=xxx
```

---

## Implementation Details

### 1. Backend Changes

#### New Setup API Endpoints

**GET /api/setup/status**
```rust
// Returns current onboarding status
Response: {
    "completed": bool,
    "database_backend": Option<String>,  // "postgres" | "libsql"
    "llm_backend": Option<String>,       // "ollama" | "openai" | "anthropic" | "openai_compatible"
    "selected_model": Option<String>,
    "gateway_port": Option<u16>
}
```

**GET /api/setup/models**
```rust
// Fetch available models based on provider
Query params:
  - provider: "ollama" | "openai" | "anthropic" | "openai_compatible"
  - base_url: Option<String>  // for ollama/openai-compatible
  - api_key: Option<String>    // for openai-compatible

Response: {
    "models": Vec<ModelInfo>
}

ModelInfo: {
    "id": String,      // "llama3", "gpt-4o", etc.
    "name": String,    // Display name
    "provider": String // Origin provider
}
```

**POST /api/setup/validate**
```rust
// Validate configuration before saving
Request Body: {
    "database_backend": "postgres" | "libsql",
    "database_url": Option<String>,
    "llm_backend": "ollama" | "openai" | "anthropic" | "openai_compatible",
    "llm_base_url": Option<String>,
    "llm_api_key": Option<String>,
    "selected_model": String
}

Response: {
    "valid": bool,
    "errors": Vec<String>
}
```

**POST /api/setup/save**
```rust
// Save configuration and complete onboarding
Request Body: {
    // Database
    "database_backend": "postgres" | "libsql",
    "database_url": Option<String>,
    "libsql_path": Option<String>,
    "libsql_url": Option<String>,
    
    // LLM
    "llm_backend": "ollama" | "openai" | "anthropic" | "openai_compatible",
    "ollama_base_url": Option<String>,
    "openai_api_key": Option<String>,
    "anthropic_api_key": Option<String>,
    "openai_compatible_base_url": Option<String>,
    "openai_compatible_api_key": Option<String>,
    "selected_model": String,
    
    // Embeddings (optional)
    "embeddings_enabled": bool,
    "embeddings_provider": Option<String>,
    "embeddings_model": Option<String>,
    
    // Gateway
    "gateway_port": u16,
    "gateway_auth_token": Option<String>
}

Response: {
    "success": bool,
    "token": String,  // Auth token for main app
    "redirect_url": String
}
```

#### Modified Main Router

In `src/channels/web/server.rs`:

```rust
// Add before protected routes
let setup = Router::new()
    .route("/api/setup/status", get(setup_status_handler))
    .route("/api/setup/models", get(setup_models_handler))
    .route("/api/setup/validate", post(setup_validate_handler))
    .route("/api/setup/save", post(setup_save_handler));

// Modify index handler
async fn index_handler(State(state): State<Arc<GatewayState>>) -> impl IntoResponse {
    let settings = get_onboard_status(&state.store).await;
    
    if !settings.completed {
        return Redirect::to("/setup");
    }
    
    Html(include_str!("static/index.html"))
}

// Add setup page route
async fn setup_page_handler() -> Html<&'static str> {
    Html(include_str!("static/setup.html"))
}
```

### 2. Frontend Changes

#### New Setup Page (`static/setup.html`)

**Design Requirements:**
- Clean, modern UI matching existing brand
- Multi-step wizard with progress indicator
- Responsive design (mobile-friendly)
- Real-time validation feedback

**Step Flow:**

```mermaid
flowchart TD
    A[Welcome Screen] --> B[Database]
    B --> C[LLM Provider]
    C --> D{Provider Type}
    D -->|Ollama| E[Ollama URL]
    D -->|OpenAI| F[API Key]
    D -->|Anthropic| G[API Key]
    D -->|OpenAI-Compatible| H[URL + Key]
    E --> I[Select Model]
    F --> I
    G --> I
    H --> I
    I --> J[Review]
    J --> K[Save]
    K --> L[Redirect to Main App]
```

**Step 1: Welcome & Database**
- Welcome message with TitanClaw logo
- Database selection cards:
  - **PostgreSQL**: "Recommended for production, Docker deployments"
  - **SQLite/libSQL**: "Lightweight, local-only storage"
- Show current selection, allow change

**Step 2: LLM Provider**
- Provider cards with icons and descriptions:
  - **Ollama**: "Run AI models locally on your machine"
  - **OpenAI**: "GPT-4, GPT-4o, and more"
  - **Anthropic**: "Claude 3.5, Claude 4"
  - **OpenAI-Compatible**: "LM Studio, vLLM, Ollama API, others"
  - **OpenRouter**: "Unified access to 100+ models"
- Visual indicator for selected provider

**Step 3: Configuration (Dynamic)**
- **Ollama**:
  - Input: Base URL (default: `http://localhost:11434`)
  - Button: "Fetch Models" → calls `/api/setup/models`
  - Dropdown: Available models
  
- **OpenAI**:
  - Input: API Key (with reveal toggle)
  - Dropdown: Model selection (fetched from API)
  
- **Anthropic**:
  - Input: API Key (with reveal toggle)
  - Dropdown: Model selection (preset list)
  
- **OpenAI-Compatible**:
  - Input: Base URL
  - Input: API Key (optional for local)
  - Input: Model name
  
- **OpenRouter**:
  - Input: API Key
  - Dropdown: Model selection

**Step 4: Review & Save**
- Summary of all selections
- "Configure" button to save
- Loading state during save
- Redirect to main app on success

### 3. Settings Integration

The existing settings system in `src/settings.rs` should be leveraged:

```rust
// In setup_save_handler:
let settings = Settings {
    database_backend: Some(req.database_backend),
    database_url: req.database_url,
    libsql_path: req.libsql_path,
    llm_backend: Some(req.llm_backend),
    ollama_base_url: req.ollama_base_url,
    openai_api_key: req.openai_api_key,
    // ... etc
};

// Save to database
store.save_settings(&settings).await?;

// Set onboard completed
store.set_setting("onboard_completed", "true").await?;

// Generate or use provided auth token
let token = req.gateway_auth_token.unwrap_or_else(generate_token);
store.set_setting("gateway_auth_token", &token).await?;
```

---

## Database Schema (Existing)

The settings are already stored in the database:

```sql
-- settings table (existing)
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMP DEFAULT NOW()
);
```

---

## Environment Variable Defaults

### Production Defaults (Docker)

```yaml
# docker-compose.yml
environment:
  # Database
  - DATABASE_BACKEND=postgres
  - DATABASE_URL=postgres://titanclaw:titanclaw@postgres:5432/titanclaw
  
  # LLM
  - LLM_BACKEND=ollama
  - OLLAMA_BASE_URL=http://host.docker.internal:11434
  - SELECTED_MODEL=llama3
  
  # Gateway
  - GATEWAY_HOST=0.0.0.0
  - GATEWAY_PORT=3000
  
  # Security
  - ONBOARD_COMPLETED=true
```

### Development Defaults (Local)

```bash
# .env
DATABASE_BACKEND=libsql
LLM_BACKEND=ollama
OLLAMA_BASE_URL=http://localhost:11434
SELECTED_MODEL=llama3
GATEWAY_HOST=127.0.0.1
GATEWAY_PORT=3000
```

---

## Configuration Matrix

| Deployment | Database | GATEWAY_HOST | Notes |
|------------|----------|--------------|-------|
| Docker Production | PostgreSQL | 0.0.0.0 | Use external DB or linked container |
| Docker Dev | PostgreSQL | 0.0.0.0 | Use docker-compose linked DB |
| Local Production | libSQL | 127.0.0.1 | Single user, local only |
| Local Dev | libSQL | 127.0.0.1 | Development testing |

---

## Security Considerations

1. **API Keys**: Store encrypted in settings, not plain text
2. **Rate Limiting**: Apply to setup endpoints to prevent abuse
3. **Validation**: Sanitize all inputs before saving
4. **HTTPS**: In production, ensure HTTPS is enforced
5. **Token**: Generate secure random tokens, not predictable

---

## Implementation Phases

### Phase 1: Core Backend (Priority: Critical)
- [ ] Add setup API endpoints
- [ ] Connect to existing settings storage
- [ ] Basic validation logic

### Phase 2: Setup Page (Priority: Critical)
- [ ] Create static HTML page
- [ ] Implement multi-step wizard
- [ ] Add CSS styling
- [ ] Add JavaScript for API calls

### Phase 3: Model Fetching (Priority: High)
- [ ] Implement Ollama model fetch
- [ ] Implement OpenAI model fetch
- [ ] Add model caching

### Phase 4: Validation (Priority: High)
- [ ] Database connection validation
- [ ] LLM API validation
- [ ] Error handling and feedback

### Phase 5: Polish (Priority: Medium)
- [ ] Responsive design
- [ ] Loading states
- [ ] Error recovery
- [ ] Help text and tooltips

---

## Testing Plan

### Unit Tests
- API endpoint validation
- Settings serialization
- Model fetching mocks

### Integration Tests
- Full onboarding flow
- Database save/load
- Configuration persistence

### Manual Testing
- [ ] Docker production deployment
- [ ] Docker development deployment
- [ ] Local production (libSQL)
- [ ] Local development
- [ ] All LLM providers
- [ ] Mobile responsive design

---

## Files to Modify

| File | Changes |
|------|---------|
| `src/channels/web/server.rs` | Add routes, modify index |
| `src/channels/web/handlers/setup.rs` | NEW - Setup handlers |
| `src/channels/web/static/setup.html` | NEW - Setup page UI |
| `src/channels/web/static/setup.css` | NEW - Setup page styles |
| `src/channels/web/static/setup.js` | NEW - Setup page logic |
| `src/settings.rs` | Potentially add new fields |
| `src/db/` | Add setup store methods if needed |

---

## Rollout Strategy

1. **Development**: Deploy to dev environment, test all flows
2. **Staging**: Deploy to staging, verify database migrations
3. **Production**: 
   - For new installs: Show onboarding wizard
   - For existing: Continue using current config (ONBOARD_COMPLETED=true)
