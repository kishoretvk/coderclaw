# TitanClaw Updates: Near AI Removal & Docker Compatibility

**Document Version:** 2.1  
**Date:** 2026-02-22  
**Status:** Planning Document  
**Author:** TitanClaw Development Team

---

## Executive Summary

This document outlines a comprehensive plan to:
1. **Remove Near AI integration** from the TitanClaw codebase
2. **Enhance Docker compatibility** to enable running from Docker images locally
3. **Add OpenAI-Compatible client** for flexible LLM provider connections
4. **Add Welcome Page** as a new channel for user interaction
5. **Use PostgreSQL** for session management and all data dependencies

---

## Part 1: Near AI Integration Analysis

### 1.1 Identified Integration Points (COMPLETE LIST)

The following table summarizes ALL Near AI integration points discovered during comprehensive code analysis:

| Category | Files Affected | Integration Type | Complexity |
|----------|----------------|------------------|------------|
| **LLM Provider** | `src/llm/nearai.rs`, `src/llm/nearai_chat.rs` | Core LLM backend | High |
| **Session Management** | `src/llm/session.rs` | Authentication | High |
| **Embeddings** | `src/workspace/embeddings.rs` | Vector embeddings | Medium |
| **Configuration** | `src/config/llm.rs`, `src/config/sandbox.rs` | Settings | Medium |
| **Setup Wizard** | `src/setup/wizard.rs` | Onboarding | Medium |
| **CLI/Doctor** | `src/cli/doctor.rs` | Diagnostics | Low |
| **Observability** | `src/observability/*.rs` | Events/Logging | Low |
| **Safety** | `src/safety/leak_detector.rs` | Security patterns | Low |
| **Sandbox** | `docker/sandbox.Dockerfile` | Default image | Medium |

### 1.2 Additional Files with Near AI References (MISSED ITEMS)

The following files also contain Near AI references that were found in the comprehensive search:

| File | Type | References |
|------|------|------------|
| `.env.example` | Config | `NEARAI_SESSION_TOKEN`, `NEARAI_MODEL`, `NEARAI_BASE_URL`, `NEARAI_AUTH_URL` |
| `deploy/env.example` | Config | Same as above |
| `CLAUDE.md` | Documentation | Environment variables, architecture docs |
| `CHANGELOG.md` | Documentation | Version history references |
| `implementation_plan.md` | Documentation | Architecture decisions |
| `src/setup/README.md` | Documentation | Provider setup guide |
| `src/main.rs` | Code | `NEARAI_SESSION_TOKEN` migration, `NEARAI_API_KEY` check |
| `src/tools/builtin/shell.rs` | Test data | Fake session token in test fixtures |
| `tools-src/github/README.md` | Documentation | Example repo references (`owner: "nearai"`) |
| `README.md` | Documentation | Project references |
| `src/sandbox/config.rs` | Code | Credential mappings, allowlist |

### 1.3 Environment Variables to Remove

| Variable | Location Used | Purpose |
|----------|---------------|---------|
| `NEARAI_SESSION_TOKEN` | session.rs, main.rs, .env.example | OAuth session |
| `NEARAI_MODEL` | config/llm.rs, .env.example | Model selection |
| `NEARAI_BASE_URL` | config/llm.rs, .env.example | API endpoint |
| `NEARAI_AUTH_URL` | config/llm.rs, .env.example | Auth endpoint |
| `NEARAI_API_MODE` | config/llm.rs | responses/chat_completions |
| `NEARAI_API_KEY` | config/llm.rs, main.rs | API key auth |
| `NEARAI_CHEAP_MODEL` | llm/mod.rs | Cheap model for routing |
| `NEARAI_FALLBACK_MODEL` | config/llm.rs | Failover model |
| `NEARAI_MAX_RETRIES` | config/llm.rs | Retry config |
| `NEARAI_SESSION_PATH` | config/llm.rs, session.rs | Session file path |

### 1.4 Detailed Integration Breakdown

#### 1.4.1 LLM Backend (`src/llm/nearai.rs`, `src/llm/nearai_chat.rs`)

**Current State:**
- Two separate NEAR AI LLM providers: `NearAiProvider` and `NearAiChatProvider`
- Session-based authentication with token refresh
- Support for Responses API and Chat Completions API modes
- Circuit breaker, failover, and response caching features

**Files to Modify:**
- `src/llm/nearai.rs` - Mark as deprecated or remove
- `src/llm/nearai_chat.rs` - Mark as deprecated or remove  
- `src/llm/mod.rs` - Remove exports

#### 1.4.2 Session Management (`src/llm/session.rs`)

**Current State:**
- `SessionConfig` with NEAR AI-specific defaults (`https://private.near.ai`)
- Session token persistence to database (`nearai.session_token`)
- Token validation and refresh logic
- Hardcoded NEAR AI URLs throughout
- `NEARAI_SESSION_TOKEN` env var migration in `src/main.rs` line ~1765

**NEW APPROACH:** Replace with PostgreSQL-based session storage:
- Store session data in `sessions` table
- Use database for all state management
- Implement secure session tokens with PostgreSQL

**Files to Modify:**
- `src/llm/session.rs` - Replace with PostgreSQL session storage
- `src/bootstrap.rs` - Use DB session storage
- `src/config/llm.rs` - Remove session path from NearAiConfig

#### 1.4.3 Embeddings (`src/workspace/embeddings.rs`)

**Current State:**
- `NearAiEmbeddings` struct using session-based auth
- Couples embeddings to NEAR AI session
- Default provider set to "nearai" in settings

**NEW APPROACH:** Use OpenAI embeddings with PostgreSQL storage:
- Use `OpenAiEmbeddings` as primary
- Store embeddings in PostgreSQL with pgvector
- Add support for local Ollama embeddings

**Files to Modify:**
- `src/workspace/embeddings.rs` - Remove NearAiEmbeddings, enhance OpenAiEmbeddings
- `src/workspace/mod.rs` - Update exports
- `src/config/embeddings.rs` - Change default to OpenAI

#### 1.4.4 Configuration Defaults

**Current State in `src/config/sandbox.rs`:**
```rust
image: "ghcr.io/nearai/sandbox:latest".to_string(),
```

**Current State in `src/settings.rs`:**
```rust
fn default_embeddings_provider() -> String {
    "nearai".to_string()
}
```

**NEW APPROACH:**
- Default sandbox image: configurable via environment
- Default embeddings: OpenAI or Ollama
- Default LLM backend: Ollama (local) or OpenAI

**Files to Modify:**
- `src/config/sandbox.rs` - Make image configurable
- `src/settings.rs` - Change defaults to OpenAI/Ollama

#### 1.4.5 Sandbox Credential Mappings (`src/sandbox/config.rs`)

**Current State:**
```rust
CredentialMapping::bearer("NEARAI_API_KEY", "api.near.ai"),
```

**Also in allowlist:**
```rust
"api.near.ai".to_string(),
```

**Files to Modify:**
- Remove NEAR AI credential mapping
- Remove from network allowlist

#### 1.4.6 Test Fixtures (`src/tools/builtin/shell.rs`)

**Current State:**
```rust
("NEARAI_SESSION_TOKEN", "sess_fake_token_abc"),
```

**Files to Modify:**
- Update test fixtures

#### 1.4.7 CLI Doctor (`src/cli/doctor.rs`)

**Current State:**
- Checks for `NEARAI_API_KEY` environment variable
- Checks for nearai session file

**Files to Modify:**
- Remove Near AI-specific checks

#### 1.4.8 Observability & Safety

**Current State:**
- `ObserverEvent` hardcodes "nearai" provider in test/mock code
- Leak detector pattern for `nearai_session` tokens

**Files to Modify:**
- `src/observability/traits.rs` - Make provider generic
- `src/observability/log.rs` - Update mock events
- `src/safety/leak_detector.rs` - Keep for backwards compatibility (useful pattern)

---

## Part 2: Docker Compatibility Analysis

### 2.1 Current Docker Setup

#### Main Dockerfile (`Dockerfile`)

**Status:** ✅ Exists and Functional

```dockerfile
# Multi-stage build for cloud deployment
- Stage 1: Build with rust:1.92-slim-bookworm
- Stage 2: Runtime with debian:bookworm-slim
- Non-root user (ironclaw:1000)
- Exposes port 3000
```

**Issues Identified:**
- No volume mounts for persistence
- No environment file handling
- Missing health checks
- No support for database initialization

#### Docker Compose (`docker-compose.yml`)

**Status:** ⚠️ Partial - Only PostgreSQL

```yaml
services:
  postgres:
    image: pgvector/pgvector:pg16
    # Basic setup only
```

**NEW ARCHITECTURE - Complete Local Docker Stack:**
```yaml
services:
  titanclaw:
    build: .
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgres://ironclaw:ironclaw@postgres:5432/ironclaw
      - RUST_LOG=info
    volumes:
      - ./data:/data
      - ./config:/config
    depends_on:
      postgres:
        condition: service_healthy

  postgres:
    image: pgvector/pgvector:pg16
    environment:
      POSTGRES_DB: ironclaw
      POSTGRES_USER: ironclaw
      POSTGRES_PASSWORD: ironclaw
    volumes:
      - pgdata:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ironclaw"]
      interval: 5s
      timeout: 3s
      retries: 5
```

#### Sandbox Dockerfile (`docker/sandbox.Dockerfile`)

**Status:** ✅ Exists - For WASM sandbox environment

### 2.2 Local Docker Requirements

| Requirement | Current State | Priority | NEW Solution |
|-------------|---------------|----------|---------------|
| Multi-stage builds | ✅ Implemented | - | Keep |
| Non-root execution | ✅ Implemented | - | Keep |
| Health checks | ❌ Missing | High | Add to Dockerfile |
| Volume mounts for data | ❌ Missing | High | Add /data, /config volumes |
| Environment configuration | ⚠️ Partial | High | Use .env file |
| Database initialization | ❌ Missing | Medium | Add init scripts |
| Entrypoint scripts | ❌ Missing | Medium | Create entrypoint.sh |
| Signal handling | ❌ Missing | Medium | Add trap in entrypoint |
| PostgreSQL for sessions | ❌ Using Near AI | High | **NEW: Use PostgreSQL** |

---

## Part 3: OpenAI-Compatible Client Enhancement

### 3.1 Current State

The codebase already has `OpenAiCompatibleConfig` in [`src/config/llm.rs`](src/config/llm.rs):
```rust
pub struct OpenAiCompatibleConfig {
    pub base_url: String,
    pub api_key: Option<SecretString>,
    pub model: String,
}
```

### 3.2 Enhancement Plan

**Add Dynamic OpenAI-Compatible Provider Registration:**

| Feature | Description |
|---------|-------------|
| **Provider Registry** | Dynamic list of OpenAI-compatible endpoints |
| **Custom Models** | Allow users to specify custom model lists |
| **Auth Modes** | Support API key, Bearer token, custom headers |
| **Connection Testing** | Validate endpoint connectivity |

**New Configuration Structure:**
```rust
pub struct OpenAiCompatibleProviderConfig {
    pub name: String,           // Display name (e.g., "LiteLLM", "vLLM")
    pub base_url: String,       // API endpoint
    pub api_key: Option<SecretString>,
    pub models: Vec<String>,    // Available models
    pub auth_header: Option<String>,  // Custom auth header
}
```

**Environment Variable Support:**
```
OPENAI_COMPATIBLE_PROVIDERS='[{"name":"vllm","base_url":"http://localhost:8000/v1","models":["llama-3"]}]'
```

**Files to Modify:**
- [`src/config/llm.rs`](src/config/llm.rs) - Add provider registry
- [`src/llm/mod.rs`](src/llm/mod.rs) - Add provider creation
- [`src/settings.rs`](src/settings.rs) - Add provider settings

---

## Part 4: Welcome Page (New Channel)

### 4.1 Overview

Add a web-based welcome page as a new channel for:
- User onboarding and welcome experience
- Prompt input interface
- LLM provider configuration
- Channel mode selection

### 4.2 Features

| Feature | Description |
|---------|-------------|
| **Welcome Screen** | Initial greeting, feature highlights |
| **Quick Prompt** | Direct prompt input without authentication |
| **Provider Setup** | Configure LLM backend (OpenAI, Anthropic, Ollama, OpenAI-Compatible) |
| **Channel Selection** | Choose interaction mode (CLI, Web, API) |
| **Settings Panel** | Configure sandbox, embeddings, model preferences |

### 4.3 Architecture

```
src/channels/web/welcome/
├── mod.rs           # Channel registration
├── handler.rs       # HTTP handlers
├── templates/        # HTML templates
│   ├── welcome.html
│   └── settings.html
└── static/          # CSS, JS assets
```

**Endpoint Design:**
| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Welcome page (if not authenticated) |
| `/api/welcome/config` | GET | Get current configuration |
| `/api/welcome/config` | POST | Save configuration |
| `/api/welcome/providers` | GET | List available LLM providers |
| `/api/chat` | POST | Send prompt (quick chat) |

### 4.4 Files to Create/Modify

| File | Action |
|------|--------|
| `src/channels/web/welcome/mod.rs` | Create |
| `src/channels/web/welcome/handler.rs` | Create |
| `src/channels/web/server.rs` | Modify - register welcome routes |
| `src/channels/web/static/welcome.css` | Create |
| `src/channels/web/static/welcome.js` | Create |
| `src/channels/mod.rs` | Modify - add channel |

---

## Part 5: Implementation Plan

### Phase 1: Near AI Removal & PostgreSQL Migration (Weeks 1-2)

#### Task 1.1: Deprecate NEAR AI LLM Provider
- [ ] Add deprecation warnings to `src/llm/nearai.rs`
- [ ] Add deprecation warnings to `src/llm/nearai_chat.rs`
- [ ] Update `src/llm/mod.rs` to mark exports as deprecated
- [ ] Update default LLM backend to `Ollama` in `src/config/llm.rs`

#### Task 1.2: Replace Session Management with PostgreSQL
- [ ] Create new `sessions` table in PostgreSQL
- [ ] Refactor `SessionConfig` to use PostgreSQL
- [ ] Implement secure token generation and storage
- [ ] Update `src/bootstrap.rs` for DB session handling
- [ ] Remove `NEARAI_SESSION_TOKEN` migration code from `src/main.rs`

#### Task 1.3: Update Embeddings Configuration
- [ ] Change default embeddings provider to OpenAI
- [ ] Remove `NearAiEmbeddings` 
- [ ] Enhance `OpenAiEmbeddings` with better config
- [ ] Use pgvector for embedding storage

#### Task 1.4: Update Configuration Defaults
- [ ] Change default sandbox image to configurable
- [ ] Set default LLM backend to Ollama
- [ ] Update environment variable documentation

#### Task 1.5: Update Environment Files
- [ ] Update `.env.example` - Remove NEARAI vars, add new providers
- [ ] Update `deploy/env.example` - Same
- [ ] Update `CLAUDE.md` - Remove Near AI references
- [ ] Update `CHANGELOG.md` - Add deprecation notice
- [ ] Update `implementation_plan.md` - Update status

#### Task 1.6: Update Sandbox Configuration
- [ ] Remove `NEARAI_API_KEY` from credential mappings in `src/sandbox/config.rs`
- [ ] Remove `api.near.ai` from network allowlist
- [ ] Update default sandbox image

#### Task 1.7: Update CLI and Tests
- [ ] Update `src/cli/doctor.rs` - Remove Near AI checks
- [ ] Update test fixtures in `src/tools/builtin/shell.rs`
- [ ] Update documentation in `src/setup/README.md`

### Phase 2: Docker Compatibility (Weeks 2-3)

#### Task 2.1: Enhance Main Dockerfile
```dockerfile
# Proposed improvements:
- Add health check (curl to /health)
- Add volume mounts for /data, /config
- Add proper signal handling (SIGTERM, SIGINT)
- Add database initialization scripts
- Support for .env file loading
- Non-root user (titanclaw:1000)
```

#### Task 2.2: Create Complete Docker Compose
```yaml
services:
  titanclaw:
    build: .
    ports:
      - "3000:3000"
    environment:
      DATABASE_URL: postgres://ironclaw:ironclaw@postgres:5432/ironclaw
    volumes:
      - titanclaw_data:/data
      - titanclaw_config:/config
    depends_on:
      postgres:
        condition: service_healthy

  postgres:
    image: pgvector/pgvector:pg16
    environment:
      POSTGRES_DB: ironclaw
      POSTGRES_USER: ironclaw
      POSTGRES_PASSWORD: ironclaw
    volumes:
      - pgdata:/var/lib/postgresql/data
```

#### Task 2.3: Add Docker Configuration Files
- [ ] Create `docker/entrypoint.sh` - Container startup with migrations
- [ ] Create `docker/healthcheck.sh` - Health verification
- [ ] Create `docker/init-db.sql` - Database schema initialization
- [ ] Create `.dockerignore` - Exclude build artifacts

### Phase 3: OpenAI-Compatible Enhancement (Week 3)

#### Task 3.1: Add Provider Registry
- [ ] Create provider configuration structure
- [ ] Add environment variable parsing
- [ ] Implement dynamic provider registration

#### Task 3.2: Add Connection Testing
- [ ] Add endpoint connectivity check
- [ ] Add model list verification
- [ ] Add error handling for misconfiguration

### Phase 4: Welcome Page (Week 4)

#### Task 4.1: Create Welcome Page Channel
- [ ] Create `src/channels/web/welcome/` module
- [ ] Implement HTTP handlers
- [ ] Add HTML templates

#### Task 4.2: Integrate with Settings
- [ ] Connect welcome page to settings system
- [ ] Add provider configuration UI
- [ ] Add channel mode selection

#### Task 4.3: Add Quick Chat
- [ ] Implement `/api/chat` endpoint
- [ ] Add prompt input interface
- [ ] Connect to LLM providers

### Phase 5: Testing & Documentation (Week 5)

#### Task 5.1: Docker Testing
- [ ] Test image build
- [ ] Test container startup
- [ ] Test database connectivity
- [ ] Test volume persistence
- [ ] Test session management

#### Task 5.2: Integration Testing
- [ ] Test Welcome page functionality
- [ ] Test OpenAI-compatible providers
- [ ] Test channel switching

#### Task 5.3: Documentation Updates
- [ ] Update `README.md` with Docker instructions
- [ ] Create `DOCKER.md` deployment guide
- [ ] Update `AGENTS.md`
- [ ] Update `implementation_plan.md`

---

## Part 6: Risk Assessment & Mitigation

### 6.1 Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Breaking existing Near AI users | High | Low | Deprecation warnings, migration guide |
| Docker build failures | Medium | Medium | Test thoroughly, use multi-stage builds |
| Database migration issues | High | Low | Backup before upgrade, test migrations |
| LLM provider configuration | Medium | Medium | Clear documentation, default to Ollama |
| Welcome page security | High | Medium | Auth guards, input sanitization |
| Session security | High | Low | Use cryptographic tokens, secure storage |

### 6.2 Mitigation Strategies

1. **Deprecation Approach:** Use Rust's `#[deprecated]` attribute with clear messages
2. **Docker Testing:** Comprehensive CI/CD tests for Docker builds
3. **Database Backward Compatibility:** Keep migrations compatible
4. **Welcome Page:** Add authentication middleware, rate limiting
5. **Session Security:** Use cryptographic tokens, secure storage

---

## Part 7: Complete File Change Summary

### 7.1 Files to Remove/Deprecate

| File | Action | Reason |
|------|--------|--------|
| `src/llm/nearai.rs` | Deprecate | NEAR AI LLM provider |
| `src/llm/nearai_chat.rs` | Deprecate | NEAR AI chat provider |

### 7.2 Files to Modify (Complete List)

| File | Changes Required |
|------|------------------|
| `src/llm/mod.rs` | Remove NearAi exports, add compatible providers |
| `src/llm/session.rs` | Replace with PostgreSQL storage |
| `src/llm/nearai.rs` | Add deprecation warning |
| `src/llm/nearai_chat.rs` | Add deprecation warning |
| `src/workspace/embeddings.rs` | Remove NearAiEmbeddings, enhance OpenAI |
| `src/workspace/mod.rs` | Update exports |
| `src/config/llm.rs` | Add provider registry, change defaults, remove NEARAI env parsing |
| `src/config/sandbox.rs` | Make image configurable, remove NEARAI credentials |
| `src/config/embeddings.rs` | Change default to OpenAI |
| `src/settings.rs` | Update defaults |
| `src/setup/wizard.rs` | Remove NEAR AI, add new providers |
| `src/setup/README.md` | Update documentation |
| `src/cli/doctor.rs` | Remove NearAI checks |
| `src/bootstrap.rs` | Use DB session storage |
| `src/main.rs` | Remove NEARAI_SESSION_TOKEN migration, NEARAI_API_KEY check |
| `src/observability/traits.rs` | Make provider generic |
| `src/observability/log.rs` | Update mock events |
| `src/sandbox/config.rs` | Remove NEARAI credential mapping, remove from allowlist |
| `src/safety/leak_detector.rs` | Keep pattern (useful) |
| `src/tools/builtin/shell.rs` | Update test fixtures |
| `Dockerfile` | Add health checks, volumes, signals |
| `docker-compose.yml` | Add complete stack |
| `src/channels/web/server.rs` | Add welcome routes |
| `.env.example` | Remove NEARAI vars, add OpenAI/Ollama |
| `deploy/env.example` | Same updates |
| `CLAUDE.md` | Remove Near AI references |
| `CHANGELOG.md` | Add deprecation notice |
| `implementation_plan.md` | Update status |
| `README.md` | Update project description |

### 7.3 Files to Create

| File | Purpose |
|------|---------|
| `docker/entrypoint.sh` | Container startup script |
| `docker/healthcheck.sh` | Health check script |
| `docker/init-db.sql` | Database initialization |
| `src/channels/web/welcome/mod.rs` | Welcome channel |
| `src/channels/web/welcome/handler.rs` | HTTP handlers |
| `src/channels/web/static/welcome.html` | Welcome page |
| `src/channels/web/static/welcome.js` | Frontend JS |
| `src/channels/web/static/welcome.css` | Styling |
| `DOCKER.md` | Docker deployment guide |

---

## Part 8: Docker Image Architecture

### 8.1 Proposed Image Structure

```
ghcr.io/titanclaw/titanclaw:latest
├── /app
│   ├── titanclaw (binary)
│   └── migrations/
├── /data (volume mount)
│   ├── settings.json
│   ├── sessions.db
│   └── logs/
└── /config (volume mount)
    └── env (environment file)
```

### 8.2 Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | - | PostgreSQL connection string |
| `LLM_BACKEND` | No | `ollama` | LLM provider (ollama, openai, anthropic, openai_compatible) |
| `ANTHROPIC_API_KEY` | Conditional | - | Anthropic API key |
| `OPENAI_API_KEY` | Conditional | - | OpenAI API key |
| `OLLAMA_BASE_URL` | No | `http://localhost:11434` | Ollama endpoint |
| `OPENAI_COMPATIBLE_BASE_URL` | No | - | Custom OpenAI-compatible endpoint |
| `OPENAI_COMPATIBLE_API_KEY` | No | - | API key for compatible endpoint |
| `OPENAI_COMPATIBLE_MODEL` | No | - | Model name for compatible endpoint |
| `RUST_LOG` | No | `info` | Log level |
| `SANDBOX_IMAGE` | No | `ghcr.io/titanclaw/sandbox:latest` | Sandbox image |
| `POSTGRES_PASSWORD` | Yes | - | Database password (Docker) |

### 8.3 Volume Mounts

| Volume | Purpose |
|--------|---------|
| `/data` | Application data (settings, sessions) |
| `/config` | Configuration files |
| `/data/logs` | Application logs |

---

## Part 9: Welcome Page UI Design

### 9.1 Page Structure

```
┌─────────────────────────────────────────────────────┐
│  TitanClaw - Welcome                               │
├─────────────────────────────────────────────────────┤
│  ┌───────────────────────────────────────────────┐  │
│  │            Welcome to TitanClaw!              │  │
│  │                                               │  │
│  │  Your secure AI assistant                    │  │
│  └───────────────────────────────────────────────┘  │
│                                                     │
│  Quick Prompt                                       │
│  ┌─────────────────────────────────────────────┐   │
│  │ Enter your prompt here...                   │   │
│  └─────────────────────────────────────────────┘   │
│  [Send]                                             │
│                                                     │
│  ─────────────── OR ───────────────                │
│                                                     │
│  Provider Setup                                     │
│  ┌─────────────────────────────────────────────┐   │
│  │ Backend: [Ollama ▼]                         │   │
│  │ Model:   [llama-3 ▼]                        │   │
│  │ URL:     [http://localhost:11434]          │   │
│  │ [Test Connection]                          │   │
│  └─────────────────────────────────────────────┘   │
│                                                     │
│  Channel Selection                                  │
│  ○ Web Chat    ○ CLI    ○ API                      │
│                                                     │
│  [Get Started]                                      │
└─────────────────────────────────────────────────────┘
```

### 9.2 API Integration

```javascript
// Quick chat via welcome page
POST /api/chat
{
  "prompt": "Hello, help me write a function",
  "provider": "openai_compatible",
  "model": "gpt-4"
}

// Provider test
GET /api/welcome/test-connection?provider=ollama&url=http://localhost:11434
```

---

## Part 10: Backward Compatibility

### 10.1 Settings Migration

Existing `settings.json` files with `"llm_backend": "nearai"` will continue to work but will show deprecation warnings. Users should be guided to:
1. Set `LLM_BACKEND` environment variable to new provider
2. Use Welcome page to reconfigure

### 10.2 Database Schema

**New Session Table:**
```sql
CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL,
    token_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    metadata JSONB
);
```

### 10.3 API Compatibility

All existing HTTP APIs remain unchanged. New endpoints:
- `/api/welcome/*` - Welcome page API
- `/api/chat` - Quick chat endpoint

---

## Conclusion

This updated plan provides a comprehensive approach to:

1. **Remove Near AI dependencies** with PostgreSQL-based session management
2. **Enhance Docker compatibility** for local containerized deployments
3. **Add OpenAI-Compatible client** for flexible LLM provider connections
4. **Add Welcome Page** as a new web-based channel for interaction
5. **Maintain backward compatibility** where possible

The implementation follows a phased approach:
- **Phase 1**: Near AI removal + PostgreSQL migration (2 weeks)
- **Phase 2**: Docker enhancements (1 week)
- **Phase 3**: OpenAI-Compatible enhancements (1 week)
- **Phase 4**: Welcome page (1 week)
- **Phase 5**: Testing and documentation (1 week)

---

## Appendix A: Complete Code Reference List

### Environment Variables Found
- `NEARAI_SESSION_TOKEN`
- `NEARAI_MODEL`
- `NEARAI_BASE_URL`
- `NEARAI_AUTH_URL`
- `NEARAI_API_MODE`
- `NEARAI_API_KEY`
- `NEARAI_CHEAP_MODEL`
- `NEARAI_FALLBACK_MODEL`
- `NEARAI_MAX_RETRIES`
- `NEARAI_SESSION_PATH`

### Files with Near AI References (ALL)
1. `.env.example`
2. `deploy/env.example`
3. `CLAUDE.md`
4. `CHANGELOG.md`
5. `implementation_plan.md`
6. `src/setup/README.md`
7. `src/main.rs`
8. `src/tools/builtin/shell.rs`
9. `tools-src/github/README.md`
10. `README.md`
11. `src/sandbox/config.rs`
12. `src/llm/nearai.rs`
13. `src/llm/nearai_chat.rs`
14. `src/llm/session.rs`
15. `src/llm/mod.rs`
16. `src/config/llm.rs`
17. `src/config/sandbox.rs`
18. `src/settings.rs`
19. `src/workspace/embeddings.rs`
20. `src/setup/wizard.rs`
21. `src/cli/doctor.rs`
22. `src/bootstrap.rs`
23. `src/observability/traits.rs`
24. `src/observability/log.rs`
25. `src/safety/leak_detector.rs`

### Total References Found: 200+

---

## Appendix B: Alternative Approaches Considered

### Option B.1: Feature Flag Approach
Keep NEAR AI code behind a `nearai` feature flag:
```toml
[features]
default = ["postgres", "libsql"]
nearai = []
```

**Pros:** Complete removal from default builds  
**Cons:** More complex CI/CD, potential runtime errors

### Option B.2: Stub Implementation
Replace NEAR AI with stub that returns errors:
```rust
#[deprecated(note = "NEAR AI support has been removed")]
```

**Pros:** Minimal code changes  
**Cons:** Larger binary, confusing errors

**Selected Approach:** Deprecation (Option B.2) combined with Phase 1-5 implementation for clean removal.

---

*Document End - Version 2.1*
