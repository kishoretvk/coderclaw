//! `ironclaw doctor` - active health diagnostics.
//!
//! Probes external dependencies and validates configuration to surface
//! problems before they bite during normal operation. Each check reports
//! pass/fail with actionable guidance on failures.

use std::path::PathBuf;

/// Run all diagnostic checks and print results.
pub async fn run_doctor_command() -> anyhow::Result<()> {
    println!("IronClaw Doctor");
    println!("===============\n");

    let mut passed = 0u32;
    let mut failed = 0u32;

    // ── Configuration checks ──────────────────────────────────

    // ⚠️ DEPRECATED: Near AI check - will be removed in future versions
    // Only run if explicitly using Near AI backend
    if should_check_nearai() {
        check(
            "NEAR AI session (DEPRECATED)",
            check_nearai_session().await,
            &mut passed,
            &mut failed,
        );
    } else {
        println!("  [skip] NEAR AI session: deprecated, using {}", get_llm_backend());
    }

    check(
        "Database backend",
        check_database().await,
        &mut passed,
        &mut failed,
    );

    check(
        "Workspace directory",
        check_workspace_dir(),
        &mut passed,
        &mut failed,
    );

    // ── External binary checks ────────────────────────────────

    check(
        "Docker",
        check_binary("docker", &["--version"]),
        &mut passed,
        &mut failed,
    );

    check(
        "cloudflared",
        check_binary("cloudflared", &["--version"]),
        &mut passed,
        &mut failed,
    );

    check(
        "ngrok",
        check_binary("ngrok", &["version"]),
        &mut passed,
        &mut failed,
    );

    check(
        "tailscale",
        check_binary("tailscale", &["version"]),
        &mut passed,
        &mut failed,
    );

    // ── Summary ───────────────────────────────────────────────

    println!();
    println!("  {passed} passed, {failed} failed");

    if failed > 0 {
        println!("\n  Some checks failed. This is normal if you don't use those features.");
    }

    Ok(())
}

// ── Individual checks ───────────────────────────────────────

fn check(name: &str, result: CheckResult, passed: &mut u32, failed: &mut u32) {
    match result {
        CheckResult::Pass(detail) => {
            *passed += 1;
            println!("  [pass] {name}: {detail}");
        }
        CheckResult::Fail(detail) => {
            *failed += 1;
            println!("  [FAIL] {name}: {detail}");
        }
        CheckResult::Skip(reason) => {
            println!("  [skip] {name}: {reason}");
        }
    }
}

enum CheckResult {
    Pass(String),
    Fail(String),
    Skip(String),
}

async fn check_nearai_session() -> CheckResult {
    // DEPRECATED: This check is for Near AI which is being deprecated
    // Check if session file exists
    let session_path = crate::llm::session::default_session_path();
    if !session_path.exists() {
        // Check for API key mode
        if std::env::var("NEARAI_API_KEY").is_ok() {
            return CheckResult::Pass("API key configured (DEPRECATED)".into());
        }
        return CheckResult::Skip(format!(
            "session file not found at {}. NEAR AI is deprecated, use LLM_BACKEND=ollama instead",
            session_path.display()
        ));
    }

    // Verify the session file is readable and non-empty
    match std::fs::read_to_string(&session_path) {
        Ok(content) if content.trim().is_empty() => {
            CheckResult::Fail("session file is empty".into())
        }
        Ok(_) => CheckResult::Pass(format!(
            "session found ({}) - DEPRECATED, consider switching to Ollama",
            session_path.display()
        )),
        Err(e) => CheckResult::Fail(format!("cannot read session file: {e}")),
    }
}

/// Check if Near AI backend is being used
fn should_check_nearai() -> bool {
    // Check if LLM_BACKEND is set to nearai
    if let Ok(backend) = std::env::var("LLM_BACKEND") {
        return backend.to_lowercase().contains("nearai");
    }
    // Also check if NEARAI_SESSION_TOKEN or NEARAI_API_KEY is set
    std::env::var("NEARAI_SESSION_TOKEN").is_ok() || std::env::var("NEARAI_API_KEY").is_ok()
}

/// Get current LLM backend for display
fn get_llm_backend() -> String {
    std::env::var("LLM_BACKEND").unwrap_or_else(|_| "ollama (default)".to_string())
}

async fn check_database() -> CheckResult {
    let backend = std::env::var("DATABASE_BACKEND")
        .ok()
        .unwrap_or_else(|| "postgres".into());

    match backend.as_str() {
        "libsql" | "turso" | "sqlite" => {
            let path = std::env::var("LIBSQL_PATH")
                .map(PathBuf::from)
                .unwrap_or_else(|_| crate::config::default_libsql_path());

            if path.exists() {
                CheckResult::Pass(format!("libSQL database exists ({})", path.display()))
            } else {
                CheckResult::Pass(format!(
                    "libSQL database not found at {} (will be created on first run)",
                    path.display()
                ))
            }
        }
        _ => {
            if std::env::var("DATABASE_URL").is_ok() {
                // Try to connect
                match try_pg_connect().await {
                    Ok(()) => CheckResult::Pass("PostgreSQL connected".into()),
                    Err(e) => CheckResult::Fail(format!("PostgreSQL connection failed: {e}")),
                }
            } else {
                CheckResult::Fail("DATABASE_URL not set".into())
            }
        }
    }
}

#[cfg(feature = "postgres")]
async fn try_pg_connect() -> Result<(), String> {
    let url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL not set".to_string())?;

    let config = deadpool_postgres::Config {
        url: Some(url),
        ..Default::default()
    };
    let pool = config
        .create_pool(
            Some(deadpool_postgres::Runtime::Tokio1),
            tokio_postgres::NoTls,
        )
        .map_err(|e| format!("pool error: {e}"))?;

    let client = tokio::time::timeout(std::time::Duration::from_secs(5), pool.get())
        .await
        .map_err(|_| "connection timeout (5s)".to_string())?
        .map_err(|e| format!("{e}"))?;

    client
        .execute("SELECT 1", &[])
        .await
        .map_err(|e| format!("{e}"))?;

    Ok(())
}

#[cfg(not(feature = "postgres"))]
async fn try_pg_connect() -> Result<(), String> {
    Err("postgres feature not compiled in".into())
}

fn check_workspace_dir() -> CheckResult {
    let dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".ironclaw");

    if dir.exists() {
        if dir.is_dir() {
            CheckResult::Pass(format!("{}", dir.display()))
        } else {
            CheckResult::Fail(format!("{} exists but is not a directory", dir.display()))
        }
    } else {
        CheckResult::Pass(format!("{} will be created on first run", dir.display()))
    }
}

fn check_binary(name: &str, args: &[&str]) -> CheckResult {
    match std::process::Command::new(name)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
    {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            let version = version.trim();
            // Some tools print version to stderr
            let version = if version.is_empty() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                stderr.trim().lines().next().unwrap_or("").to_string()
            } else {
                version.lines().next().unwrap_or("").to_string()
            };

            if output.status.success() {
                CheckResult::Pass(version)
            } else {
                CheckResult::Fail(format!("exited with {}", output.status))
            }
        }
        Err(_) => CheckResult::Skip(format!("{name} not found in PATH")),
    }
}

#[cfg(test)]
mod tests {
    use crate::cli::doctor::*;

    #[test]
    fn check_binary_finds_sh() {
        match check_binary("sh", &["-c", "echo ok"]) {
            CheckResult::Pass(_) => {}
            other => panic!("expected Pass for sh, got: {}", format_result(&other)),
        }
    }

    #[test]
    fn check_binary_skips_nonexistent() {
        match check_binary("__ironclaw_nonexistent_binary__", &["--version"]) {
            CheckResult::Skip(_) => {}
            other => panic!(
                "expected Skip for nonexistent binary, got: {}",
                format_result(&other)
            ),
        }
    }

    #[test]
    fn check_workspace_dir_does_not_panic() {
        let result = check_workspace_dir();
        match result {
            CheckResult::Pass(_) | CheckResult::Fail(_) | CheckResult::Skip(_) => {}
        }
    }

    #[tokio::test]
    async fn check_nearai_session_does_not_panic() {
        let result = check_nearai_session().await;
        match result {
            CheckResult::Pass(_) | CheckResult::Fail(_) | CheckResult::Skip(_) => {}
        }
    }

    fn format_result(r: &CheckResult) -> String {
        match r {
            CheckResult::Pass(s) => format!("Pass({s})"),
            CheckResult::Fail(s) => format!("Fail({s})"),
            CheckResult::Skip(s) => format!("Skip({s})"),
        }
    }
}
