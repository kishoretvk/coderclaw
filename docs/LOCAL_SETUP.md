# Local Development Setup Guide

This guide covers running TitanClaw locally without Docker, using SQLite (libSQL) as the database.

## Prerequisites

- **Rust** 1.75+ (install via [rustup](https://rustup.rs/))
- **SQLite3** development libraries (usually included with OS)
- **Ollama** or other LLM provider running locally or accessible

## Database Options

TitanClaw supports two database backends:

| Backend | Use Case | Default For |
|---------|----------|-------------|
| **libSQL/SQLite** | Local development | Local (no Docker) |
| **PostgreSQL** | Production/Docker | Docker deployments |

When building without Docker, libSQL is the default.

## Quick Start

### 1. Clone the Repository

```bash
git clone https://github.com/titanclaw/titanclaw.git
cd titanclaw
```

### 2. Configure Environment

Create a `.env` file in the project root:

```bash
# Database - uses SQLite by default
DATABASE_BACKEND=libsql

# LLM Provider - Ollama (local) is default
LLM_BACKEND=ollama
OLLAMA_BASE_URL=http://localhost:11434
SELECTED_MODEL=llama3
```

### 3. Run the Application

```bash
cargo run --release
```

On first run, you'll see the interactive onboarding wizard:

```
╔═══════════════════════════════════════════════════════════╗
║              TitanClaw Setup Wizard                       ║
╚═══════════════════════════════════════════════════════════╝

Step 1: Database Configuration
------------------------------
Which database backend would you like to use?

  [1] libSQL (SQLite) - Local file storage (recommended for development)
  [2] PostgreSQL - Requires external database server

Enter your choice [1]:
```

## Onboarding Wizard Walkthrough

### Step 1: Database Backend

Choose **libSQL (SQLite)** for local development:
- No external database required
- Data stored in local file (default: `~/.ironclaw/data.db`)
- Optionally sync with Turso (cloud SQLite)

### Step 2: Secrets Master Key

Enter a secure key (min 32 characters) for encrypting secrets:
- This key encrypts API keys and sensitive data
- Save it securely - you'll need it for backups
- Can be set via `SECRETS_MASTER_KEY` environment variable

### Step 3: LLM Provider

Select your LLM backend:

```
Which LLM provider would you like to use?

  [0] NEAR AI (deprecated)
  [1] Anthropic (Claude)
  [2] OpenAI (GPT)
  [3] Ollama (local models)
  [4] OpenRouter
  [5] OpenAI-compatible endpoint
```

For local development, **Ollama** is recommended (option 3).

### Step 4: Ollama Configuration

If you chose Ollama:

```
Enter the Ollama base URL [http://localhost:11434]:
Available models on your system:
- llama3
- mistral
- codellama
Which model would you like to use? [llama3]:
```

Make sure Ollama is running:
```bash
# Terminal 1
ollama serve

# Terminal 2 (to pull a model)
ollama pull llama3
```

### Step 5: Embeddings (Optional)

Enable embeddings for memory search functionality:

- Requires OpenAI API key, or
- Local embeddings (if available)

```
Enable embeddings? [y/N]: y
```

### Step 6: Channel Configuration

Configure how TitanClaw communicates:

```
Configure channels? [y/N]: y

Available channels:
- CLI (always available)
- Web Gateway (recommended for browser UI)
- HTTP Webhook
- WASM Channels

Enable Web Gateway? [Y]:
Enter port [3000]:
```

### Step 7: Summary

Review and confirm your configuration:

```
Configuration Summary:
----------------------
Database: libSQL (~/.ironclaw/data.db)
LLM Provider: Ollama (http://localhost:11434)
Model: llama3
Web Gateway: http://localhost:3000
Auth Token: <generated-token>

Save configuration? [Y]:
```

## Configuration Files

After onboarding, your settings are stored in:

- **Database**: `~/.ironclaw/data.db`
- **Config**: Database (not `.env`)
- **Logs**: `~/.ironclaw/logs/`

## LLM Provider Configuration

### Using Ollama (Recommended for Development)

```bash
# Start Ollama
ollama serve

# Pull models
ollama pull llama3
ollama pull codellama

# Configure TitanClaw
titanclaw config set llm_backend ollama
titanclaw config set ollama_base_url http://localhost:11434
titanclaw config set selected_model llama3
```

### Using OpenAI

```bash
# Set API key
export OPENAI_API_KEY=sk-your-key-here

# Configure TitanClaw
titanclaw config set llm_backend openai
titanclaw config set openai_api_key $OPENAI_API_KEY
titanclaw config set selected_model gpt-4o
```

### Using Anthropic

```bash
# Set API key
export ANTHROPIC_API_KEY=sk-ant-your-key-here

# Configure TitanClaw
titanclaw config set llm_backend anthropic
titanclaw config set anthropic_api_key $ANTHROPIC_API_KEY
titanclaw config set selected_model claude-sonnet-4-20250514
```

### Using LM Studio (OpenAI-Compatible)

```bash
# Start LM Studio
# It will typically listen on http://localhost:1234/v1

# Configure TitanClaw
titanclaw config set llm_backend openai_compatible
titanclaw config set openai_compatible_base_url http://localhost:1234/v1
titanclaw config set openai_compatible_model llama-3.2
```

### Using vLLM (OpenAI-Compatible)

```bash
# Start vLLM
# Typical endpoint: http://localhost:8000/v1

# Configure TitanClaw
titanclaw config set llm_backend openai_compatible
titanclaw config set openai_compatible_base_url http://localhost:8000/v1
titanclaw config set openai_compatible_model meta-llama/Llama-3.2-3B-Instruct
```

## Running TitanClaw

### Development Mode (with auto-reload)

```bash
cargo run
```

### Production Mode

```bash
cargo run --release
```

### Skip Onboarding (if pre-configured)

```bash
cargo run -- --no-onboard
```

### Access the Web UI

Open http://localhost:3000 in your browser.

The auth token is shown in the terminal on startup:
```
╔═══════════════════════════════════════════════════════════╗
║  Web UI: http://localhost:3000/                            ║
║  Token: YOUR_AUTH_TOKEN_HERE                               ║
╚═══════════════════════════════════════════════════════════╝
```

## Managing Configuration

### View all settings

```bash
titanclaw config list
```

### Get a specific setting

```bash
titanclaw config get llm_backend
titanclaw config get selected_model
```

### Set a value

```bash
titanclaw config set selected_model llama3
titanclaw config set llm_backend ollama
```

### Reset onboarding

```bash
titanclaw config set onboard_completed false
# Then restart to trigger wizard
```

## Common Issues

### "No such file or directory: ~/.ironclaw/data.db"

The database file doesn't exist. Run the onboarding wizard:

```bash
titanclaw run
```

### "Connection refused" to Ollama

Make sure Ollama is running:

```bash
ollama serve
# In another terminal:
ollama list
```

### "API key not found"

Set the environment variable before running:

```bash
export OPENAI_API_KEY=sk-your-key-here
cargo run
```

Or configure via CLI:

```bash
titanclaw config set openai_api_key sk-your-key-here
```

### Port 3000 already in use

Change the port:

```bash
titanclaw config set gateway_port 3001
cargo run
```

## Upgrading

When upgrading to a new version:

```bash
git pull
cargo build --release
# Database migrations run automatically on startup
```

## Getting Help

- Check logs: `tail -f ~/.ironclaw/logs/*.log`
- Run diagnostics: `titanclaw doctor`
- Reset everything: Delete `~/.ironclaw/` directory and re-run onboarding
