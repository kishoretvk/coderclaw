# Docker Setup Guide

This guide covers running TitanClaw in Docker with PostgreSQL and Ollama.

## Prerequisites

- Docker Engine 20.10+
- Docker Compose vama running (for local L on the host machine2+
- Oll Quick Start

###LM inference)

## 1. Clone

```bash
 and Navigategit clone https://github.com/titanclaw/titanclaw.git
cd titanclaw
```

### the Services 2. Start

```bash
docker-compose up -d
```

This- **greSQL** withPost will start:
 port 5432 pgvector on
- **TitanClaw** application on port 3000

### 3. Access the Web UI

Open your browser to: http://localhost:3000

You'll need the token from the logs gateway:
```bash
docker logs titanclaw
```

Look for the line:
```
gateway   http://0.0.0.0token=:3000/?YOUR_TOKEN_HERE
```

## Configuration Variables

Create

### Environment a `.env` file (or edit `docker-compose.yml` directly):

| Variable | Description | Default |
|----------|-------------|---------ABASE_URL` ||
| `DAT PostgreSQL connection URL | `postgres://itanclaw@postgres:5432/ttitanclaw:titanclaw` |
| `LLM_BACKEND` | LLM provider | `ollama` |
| `OLLAMA_BASE_URL` | Ollama API endpoint | `http://host.docker.internal:11434` |
| `SELECTED_MODEL` | ( Model to use |none - must configure) |
| `GATEWAY_HOST` | Web server bind address | `0.0.| `GATE |
0.0`WAY_PORT` | Web server port | `3000` |
| `ONBOARD_COMPLETED` | Skip onboarding | `true` (pre-configured) LLM Providers

 |

### Using Different#### Ollama (Default - Local Models)

```yaml
environment:
  - LLM_BACKEND=ollama
  - OLLAMA_BASE_URL=http://host.docker.internal:11434
  - SELECTED_MODEL=llama3
```

Make sure Ollama is running on your host:
```bash
# On host machine
ollama serve
ollama pull llama3
```

#### OpenAI (Cloud)

```yaml
environment:
  - LLM_BACKEND=openai
  - OPENAI_API_KEY=sk-your-key-here
  - SELECTED_MODEL=gpt-4o
```

#### Anthropic (Cloud)

```yaml
environment:
  - LLM_BACKEND=anthropic
  - ANTHROPIC_API_KEY=sk-ant-your-key-here
  - SELECTED_MODEL=claude-sonnet-4-20250514
```

#### OpenAI-Compatible Endpoints

For LM Studio, vLLM, or other compatible APIs:

```yaml
environment:
  - LLM_BACKEND=openai_compatible
  - OPENAI_COMPATIBLE_BASE_URL=http://host.docker.internal:1234/v1
  - OPENAI_COMPATIBLE_MODEL=llama-3.2
  # Optional:
  # OPENAI_COMPATIBLE_API_KEY=sk-optional
```

#### OpenRouter (Unified Multi-Provider)

```yaml
environment:
  - LLM_BACKEND=openai_compatible
  - OPENAI_COMPATIBLE_BASE_URL=https://openrouter.ai/api/v1
  - OPENAI_COMPATIBLE_API_KEY=sk-or-your-key-here
  - OPENAI_COMPATIBLE_MODEL=anthropic/claude-sonnet-4
```

### Database Configuration

The Docker compose includes PostgreSQL by default. To customize:

```yaml
environment:
  - DATABASE_URL=postgres://user:password@host:5432/database
```

## Pre-Configuration (Skip Onboarding)

The provided `docker-compose.yml` includes pre-configured settings to skip the onboarding wizard:

```yaml
environment:
  - ONBOARD_COMPLETED=true
  - DATABASE_URL=postgres://titanclaw:titanclaw@postgres:5432/titanclaw
```

### Manually Configuring via CLI

If you need to reconfigure after startup:

```bash
# Enter the container
docker exec -it titanclaw /bin/bash

# Set configuration
titanclaw config set llm_backend ollama
titanclaw config set ollama_base_url http://host.docker.internal:11434
titanclaw config set selected_model llama3
titanclaw config set onboard_completed true
```

## Common Issues

### "Empty reply from server"

The web server is binding to `127.0.0.1` instead of `0.0.0.0`. Ensure `GATEWAY_HOST=0.0.0.0` is set in your environment.

### Cannot connect to Ollama

Ollama runs on your host machine. Use `host.docker.internal` to access it from the container:

```
OLLAMA_BASE_URL=http://host.docker.internal:11434
```

On Windows/macOS, this should work automatically. On Linux, you may need to add `--add-host=host.docker.internal:host-gateway` to your docker run command.

### Database connection failed

Ensure PostgreSQL is healthy before TitanClaw starts:

```bash
docker-compose up -d postgres
# Wait for "postgres" container to be healthy
docker-compose up -d titanclaw
```

## Health Check

```bash
curl http://localhost:3000/api/health
```

Expected response:
```json
{"status":"healthy"}
```

## Logs

View application logs:
```bash
docker-compose logs -f titanclaw
```

View database logs:
```bash
docker-compose logs -f postgres
```

## Stopping

```bash
docker-compose down
```

To also remove volumes (data):
```bash
docker-compose down -v
```
