# TitanClaw Updates: Near AI Removal & Docker Enhancement

## Executive Summary

This document outlines the comprehensive plan to:
1. **Remove Near AI integration** - Deprecate and remove all Near AI-related code
2. **Enhance Docker compatibility** - Improve Docker deployment experience

---

## Part 1: Near AI Integration Analysis

### Current State

Near AI was the original default backend for TitanClaw. The integration spans multiple components:

| Component | File(s) | Integration Type |
|-----------|---------|------------------|
| LLM Backend Enum | [`src/config/llm.rs:19`](src/config/llm.rs:19) | `LlmBackend::NearAi` variant |
| LLM Provider | [`src/llm/nearai.rs`](src/llm/nearai.rs) | Full provider implementation |
| Chat Provider | [`src/llm/nearai_chat.rs`](src/llm/nearai_chat.rs) | Alternative chat API |
| Session Manager | [`src/llm/session.rs`](src/llm/session.rs) | Auth token management |
| Settings | [`src/settings.rs`](src/settings.rs) | Backend & embeddings config |
| Setup Wizard | [`src/setup/wizard.rs`](src/setup/wizard.rs) | Interactive setup flow |
| CLI Doctor | [`src/cli/doctor.rs`](src/cli/doctor.rs) | Health checks |
| Main/App | [`src/main.rs`](src/main.rs), [`src/app.rs`](src/app.rs) | Runtime initialization |

### Reference Count (119 instances)

```
src/settings.rs           - 14 references
src/setup/wizard.rs      - 17 references  
src/main.rs              - 28 references
src/llm/nearai.rs        - 24 references
src/llm/session.rs       - 26 references
src/config/llm.rs        - 8 references
src/app.rs               - 12 references
```

---

## Part 2: Near AI Removal Plan

### Phase 1: Deprecation (Completed ✅)

- [x] Add deprecation warnings to `src/llm/mod.rs` for `NearAiProvider` and `NearAiChatProvider`
- [x] Change default embeddings provider from "nearai" to "openai" in `src/settings.rs`
- [x] Update `.env.example` with deprecation notices

### Phase 1: Setup Wizard Updates (In Progress)

- [x] Update provider list to remove "NEAR AI" as primary option
- [x] Change default backend to "ollama" 
- [x] Simplify embeddings setup (remove Near AI option)
- [ ] Remove `setup_nearai()` method or mark deprecated
- [ ] Remove `fetch_nearai_models()` method

### Phase 3: Session Management

- [ ] Remove NearAi session token handling in `src/llm/session.rs`
- [ ] Remove `nearai.session_token` and `nearai.session` from DB migration patterns
- [ ] Update `src/bootstrap.rs` to remove session token storage

### Phase 4: Config & LLM Module

- [ ] Remove or mark deprecated `LlmBackend::NearAi` variant
- [ ] Remove NearAiConfig struct or simplify
- [ ] Remove `create_nearai_provider()` and `create_nearai_chat_provider()` functions
- [ ] Update error messages to remove "nearai" from valid options

### Phase 5: Main & App Cleanup

- [ ] Remove Near AI initialization in `src/main.rs`:
  - Session manager creation
  - NearAiEmbeddings initialization
  - Circuit breaker config
  - Response cache config
- [ ] Remove similar code from `src/app.rs`

### Phase 6: CLI & Observability

- [ ] Simplify `src/cli/doctor.rs` - remove Near AI health checks
- [ ] Update `src/safety/leak_detector.rs` - remove nearai_session pattern
- [ ] Update test cases in observability modules

### Phase 7: Sandbox Configuration

- [ ] Change default sandbox image from `ghcr.io/nearai/sandbox:latest` to `ghcr.io/titanclaw/sandbox:latest`

---

## Part 3: Docker Compatibility Analysis

### Current Docker Setup

| File | Purpose | Status |
|------|---------|--------|
| [`Dockerfile`](Dockerfile) | Multi-stage build for TitanClaw | ✅ Complete |
| [`docker-compose.yml`](docker-compose.yml) | Full stack with PostgreSQL | ✅ Complete |
| [`docker/sandbox.Dockerfile`](docker/sandbox.Dockerfile) | Custom sandbox image | ✅ Complete |
| [`docker/entrypoint.sh`](docker/entrypoint.sh) | Container startup | ✅ Complete |
| [`docker/healthcheck.sh`](docker/healthcheck.sh) | Health checks | ✅ Complete |

### Existing Features

1. **Multi-stage build** - Optimized image size
2. **Health checks** - Built-in HTTP health endpoint
3. **Non-root user** - Security best practice
4. **Volume mounts** - `/data` and `/config` persistence
5. **PostgreSQL** - With pgvector support
6. **Ollama integration** - Via `host.docker.internal`

### Docker Improvements Identified

1. **Default sandbox image** - Currently points to `ghcr.io/nearai/sandbox:latest` → needs to change to TitanClaw's own image
2. **Environment documentation** - Add comprehensive env var docs
3. **ARM64 support** - Consider multi-platform builds
4. **Production compose** - Add production-ready compose file

---

## Part 4: Implementation Roadmap

### Recommended Execution Order

```
┌─────────────────────────────────────────────────────────────────────┐
│                        IMPLEMENTATION FLOW                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Phase 1: Setup Wizard (High Impact, Low Risk)                      │
│  ├─ Remove Near AI from provider selection                         │
│  ├─ Simplify embeddings setup                                       │
│  └─ Change defaults to Ollama                                       │
│                                                                      │
│  Phase 2: Config & LLM Module (Medium Impact)                       │
│  ├─ Remove NearAi from LlmBackend enum                              │
│  ├─ Remove NearAiConfig                                            │
│  └─ Update create_llm_provider_with_config()                        │
│                                                                      │
│  Phase 3: Main & App (Medium Impact)                                 │
│  ├─ Remove session manager for Near AI                              │
│  ├─ Remove NearAiEmbeddings                                         │
│  └─ Simplify circuit breaker / cache config                         │
│                                                                      │
│  Phase 4: Session & Bootstrap (Low Impact)                          │
│  ├─ Remove session.rs Near AI references                            │
│  └─ Update bootstrap.rs                                             │
│                                                                      │
│  Phase 5: CLI & Utilities (Low Impact)                              │
│  ├─ Simplify doctor.rs                                              │
│  └─ Update leak_detector.rs                                         │
│                                                                      │
│  Phase 6: Docker (Enhancement)                                       │
│  ├─ Update default sandbox image                                    │
│  └─ Add documentation                                                │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Risk Assessment

| Phase | Risk Level | Mitigation |
|-------|------------|------------|
| Setup Wizard | Low | Interactive, users can still manually configure |
| Config/LLM | Medium | Keep deprecated enum variant to avoid breaking |
| Main/App | Medium | Ensure other backends work correctly |
| Session/Bootstrap | Low | Mostly dead code for new users |
| CLI/Utilities | Low | Remove optional checks |
| Docker | Low | Enhancement, no breaking changes |

---

## Part 5: File-by-File Changes

### Critical Files to Modify

#### 1. [`src/config/llm.rs`](src/config/llm.rs)
```rust
// Remove or deprecate:
- LlmBackend::NearAi variant (line 19)
- NearAiConfig struct (around line 110)
- nearai_api_key parsing (line 228)
- nearai config resolution
```

#### 2. [`src/llm/mod.rs`](src/llm/mod.rs)
```rust
// Keep but fully deprecate:
- pub use nearai::{ModelInfo, NearAiProvider};
- pub use nearai_chat::NearAiChatProvider;

// Remove from create_llm_provider_with_config():
- LlmBackend::NearAi case
```

#### 3. [`src/settings.rs`](src/settings.rs)
```rust
// Update:
- Default llm_backend from "nearai" to "ollama"
- Default embeddings.provider (already done)
```

#### 4. [`src/setup/wizard.rs`](src/setup/wizard.rs)
```rust
// Remove:
- setup_nearai() method
- fetch_nearai_models() method  
- Near AI from provider match
- Near AI from embeddings selection
```

#### 5. [`src/main.rs`](src/main.rs)
```rust
// Remove:
- Session manager for Near AI (lines 104-115)
- NearAiEmbeddings (lines 112-120)
- Near AI circuit breaker config
- Near AI response cache config
```

#### 6. [`src/app.rs`](src/app.rs)
```rust
// Similar removals as main.rs
```

#### 7. [`src/sandbox/config.rs`](src/config/sandbox.rs)
```rust
// Update line 37:
- image: "ghcr.io/nearai/sandbox:latest".to_string()
// To:
- image: "ghcr.io/titanclaw/sandbox:latest".to_string()
```

---

## Part 6: Docker Enhancement Details

### Recommended Sandbox Image Strategy

1. **Current**: `ghcr.io/nearai/sandbox:latest`
2. **Target**: `ghcr.io/titanclaw/sandbox:latest`

The TitanClaw team should build and maintain their own sandbox image with:
- Rust toolchain
- Node.js/npm
- Python
- Common CLI tools (git, curl, etc.)
- WASM support

### Environment Variables for Docker

```bash
# Required
DATABASE_URL=postgres://user:pass@host:5432/db

# LLM Configuration  
LLM_BACKEND=ollama                    # or openai, anthropic, openai_compatible
OLLAMA_BASE_URL=http://host.docker.internal:11434
SELECTED_MODEL=llama3.1:8b

# Security
SECRETS_MASTER_KEY=...               # 32 bytes hex

# Optional
EMBEDDINGS_PROVIDER=openai           # or disable
RUST_LOG=titanclaw=info
DATA_DIR=/data
CONFIG_DIR=/config
```

### Docker Compose Commands

```bash
# Development
docker-compose up -d

# With custom model
SELECTED_MODEL=phi3:14b docker-compose up -d

# Production (with persistence)
docker-compose -f docker-compose.prod.yml up -d

# View logs
docker-compose logs -f titanclaw

# Stop
docker-compose down
```

---

## Part 7: Testing Checklist

After implementing changes, verify:

- [ ] `cargo test` passes
- [ ] Docker build succeeds: `docker build --platform linux/amd64 -t titanclaw:test .`
- [ ] Docker Compose starts: `docker-compose up -d`
- [ ] Health endpoint responds: `curl http://localhost:3000/health`
- [ ] Ollama backend works (if configured)
- [ ] OpenAI backend works (if configured)
- [ ] PostgreSQL data persists after restart

---

## Appendix A: Search Patterns

To find all Near AI references:
```bash
# In Rust files
grep -rn "nearai" src/

# In config files  
grep -rn "nearai" --include="*.toml" --include="*.yaml" --include="*.yml"
```

---

## Appendix B: Rollback Plan

If issues arise:

1. **Keep deprecated code**: Instead of full removal, mark as `#[deprecated]`
2. **Feature flags**: Use Cargo features to conditionally compile Near AI
3. **Config fallback**: Keep NearAiConfig but don't auto-populate

---

## Document Info

| Attribute | Value |
|-----------|-------|
| Version | 1.0 |
| Created | 2026-02-22 |
| Last Updated | 2026-02-22 |
| Status | Planning Complete |
