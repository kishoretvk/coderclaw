# TitanClaw Setup Guide

Choose your deployment type:

## Quick Start

### Option 1: Docker (Recommended for Production)

```bash
# Clone and navigate to project
git clone https://github.com/titanclaw/titanclaw.git
cd titanclaw

# Start with defaults (PostgreSQL + Ollama)
docker-compose up -d
```

Access at **http://localhost:3000**

### Option 2: Local Development

```bash
# Clone and navigate to project
git clone https://github.com/titanclaw/titanclaw.git
cd titanclaw

# Create .env file
cp .env.example .env
# Edit .env with your settings

# Run with SQLite (libSQL)
cargo run --release
```

Access at **http://localhost:3000**

---

## Deployment Comparison

| Feature | Docker | Local |
|---------|--------|-------|
| **Database** | PostgreSQL | SQLite (libSQL) |
| **LLM Default** | Ollama | Ollama |
| **Auto-update** | Yes | Manual |
| **Portability** | High | Medium |
| **Multi-user** | Yes | No |

---

## Docker Setup (PostgreSQL)

### Prerequisites
- Docker 20.10+
- Docker Compose v2+

### Quick Start

```bash
docker-compose up -d
```

This starts:
- PostgreSQL with pgvector (port 5432)
- TitanClaw application (port 3000)

### Configuration

Edit `docker-compose.yml` or set environment variables:

```yaml
services:
  titanclaw:
    environment:
      # Database
      - DATABASE_URL=postgres://titanclaw:titanclaw@postgres:5432/titanclaw
      
      # LLM Provider (see LLM Configuration section below)
      - LLM_BACKEND=ollama
      - OLLAMA_BASE_URL=http://host.docker.internal:11434
      - SELECTED_MODEL=llama3
      
      # Web Server
      - GATEWAY_HOST=0.0.0.0
      - GATEWAY_PORT=3000
      
      # Skip onboarding (pre-configured)
      - ONBOARD_COMPLETED=true
```

### Accessing the Web UI

1. Check logs for auth token:
```bash
docker logs titanclaw
```

2. Look for the gateway URL:
```
gateway   http://0.0.0.0:3000/?token=YOUR_TOKEN_HERE
```

3. Open http://localhost:3000 and enter the token

### Ollama Setup for Docker

On your host machine:

```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Start Ollama service
ollama serve

# Pull models
ollama pull llama3
```

**Important:** Use `host.docker.internal` to access Ollama from Docker:
```
OLLAMA_BASE_URL=http://host.docker.internal:11434
```

---

## Local Setup (SQLite/libSQL)

### Prerequisites
- Rust 1.75+
- SQLite3 development libraries
- Ollama (optional, for local models)

### Quick Start

```bash
# Create environment file
cp .env.example .env

# Edit .env with your configuration
nano .env

# Run the application
cargo run --release
```

### Configuration (.env)

```bash
# Database - uses SQLite by default
DATABASE_BACKEND=libsql

# LLM Provider (see LLM Configuration section below)
LLM_BACKEND=ollama
OLLAMA_BASE_URL=http://localhost:11434
SELECTED_MODEL=llama3

# Web Server
GATEWAY_HOST=127.0.0.1
GATEWAY_PORT=3000
```

### First Run (Onboarding Wizard)

On first run, the CLI wizard guides you through:

1. **Database**: Choose libSQL (default for local)
2. **Master Key**: Enter encryption key (min 32 chars)
3. **LLM Provider**: Select your provider
4. **Model**: Choose or enter model name
5. **Channels**: Enable web gateway
6. **Review**: Confirm and save

---

## LLM Provider Configuration

### Ollama (Local Models) - Recommended for Development

```bash
# Start Ollama
ollama serve

# Pull models
ollama pull llama3
ollama pull codellama

# TitanClaw configuration
LLM_BACKEND=ollama
OLLAMA_BASE_URL=http://localhost:11434
SELECTED_MODEL=llama3
```

### OpenAI (Cloud)

```bash
# Get API key from https://platform.openai.com/api-keys
LLM_BACKEND=openai
OPENAI_API_KEY=sk-your-key-here
SELECTED_MODEL=gpt-4o
```

Or via CLI:
```bash
titanclaw config set llm_backend openai
titanclaw config set openai_api_key sk-your-key-here
titanclaw config set selected_model gpt-4o
```

### Anthropic (Cloud)

```bash
# Get API key from https://console.anthropic.com/
LLM_BACKEND=anthropic
ANTHROPIC_API_KEY=sk-ant-your-key-here
SELECTED_MODEL=claude-sonnet-4-20250514
```

### OpenAI-Compatible (LM Studio, vLLM, etc.)

For LM Studio:
```bash
# Start LM Studio, it typically listens on port 1234
LLM_BACKEND=openai_compatible
OPENAI_COMPATIBLE_BASE_URL=http://localhost:1234/v1
OPENAI_COMPATIBLE_MODEL=llama-3.2
```

For vLLM:
```bash
# Start vLLM with OpenAI API server
# Typical endpoint: http://localhost:8000/v1
LLM_BACKEND=openai_compatible
OPENAI_COMPATIBLE_BASE_URL=http://localhost:8000/v1
OPENAI_COMPATIBLE_MODEL=meta-llama/Llama-3.2-3B-Instruct
```

### OpenRouter (Unified Multi-Provider)

```bash
# Get API key from https://openrouter.ai/keys
LLM_BACKEND=openai_compatible
OPENAI_COMPATIBLE_BASE_URL=https://openrouter.ai/api/v1
OPENAI_COMPATIBLE_API_KEY=sk-or-your-key-here
OPENAI_COMPATIBLE_MODEL=anthropic/claude-sonnet-4-20250514
```

---

## Environment Variables Reference

### Database

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_BACKEND` | `postgres` or `libsql` | Based on build |
| `DATABASE_URL` | PostgreSQL connection string | - |
| `DATABASE_POOL_SIZE` | Connection pool size | 10 |

### LLM

| Variable | Description | Default |
|----------|-------------|---------|
| `LLM_BACKEND` | Provider: `ollama`, `openai`, `anthropic`, `openai_compatible` | `ollama` |
| `SELECTED_MODEL` | Model ID to use | - |
| `OLLAMA_BASE_URL` | Ollama API endpoint | `http://localhost:11434` |
| `OPENAI_API_KEY` | OpenAI API key | - |
| `ANTHROPIC_API_KEY` | Anthropic API key | - |
| `OPENAI_COMPATIBLE_BASE_URL` | Custom endpoint URL | - |
| `OPENAI_COMPATIBLE_API_KEY` | Custom endpoint API key | (optional) |

### Web Gateway

| Variable | Description | Default |
|----------|-------------|---------|
| `GATEWAY_HOST` | Bind address | `127.0.0.1` (local) / `0.0.0.0` (Docker) |
| `GATEWAY_PORT` | HTTP port | `3000` |
| `GATEWAY_AUTH_TOKEN` | Authentication token | (auto-generated) |
| `GATEWAY_ENABLED` | Enable gateway | `true` |

### Other

| Variable | Description | Default |
|----------|-------------|---------|
| `ONBOARD_COMPLETED` | Skip onboarding wizard | `false` |
| `RUST_LOG` | Log level | `info` |
| `SECRETS_MASTER_KEY` | Encryption key | (required) |

---

## Managing Configuration

### View Settings

```bash
titanclaw config list
```

### Get Specific Setting

```bash
titanclaw config get llm_backend
titanclaw config get selected_model
```

### Set a Value

```bash
titanclaw config set selected_model llama3
titanclaw config set llm_backend ollama
```

### Reset Onboarding

```bash
titanclaw config set onboard_completed false
# Restart to trigger wizard
```

---

## Troubleshooting

### "Empty reply from server"

Web server is binding to localhost only. Set:
```bash
GATEWAY_HOST=0.0.0.0
```

### "Connection refused" to Ollama

Ensure Ollama is running:
```bash
ollama serve
ollama list
```

### Database errors

For SQLite:
- Check file permissions on `~/.ironclaw/`
- Delete `~/.ironclaw/data.db` and re-run onboarding

For PostgreSQL:
- Check DATABASE_URL is correct
- Verify PostgreSQL is running and accessible

### Port already in use

Change the port:
```bash
GATEWAY_PORT=3001
```

---

## Production Considerations

### Security
- Change the default auth token
- Use HTTPS in production (reverse proxy)
- Secure your API keys
- Set a strong `SECRETS_MASTER_KEY`

### Performance
- Increase `DATABASE_POOL_SIZE` for multi-user
- Use PostgreSQL for production
- Configure appropriate model cache

### Backups
- Regularly backup `~/.ironclaw/` directory
- For PostgreSQL, use standard database backups
- Export settings: `titanclaw config export`

---

## Next Steps

After setup:
1. Access the Web UI at http://localhost:3000
2. Enter your auth token
3. Start chatting with the AI assistant
4. Explore the Memory, Jobs, and Routines tabs
