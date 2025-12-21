use std::path::PathBuf;
use std::time::Duration;
use crate::orchestrator::supervisor::SupervisorTrait;

/// Theme manager - scaffolding only.
#[derive(Debug, Clone)]
pub struct ThemeManager {
    pub workspace_root: PathBuf,
}

impl ThemeManager {
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self { workspace_root: workspace_root.into() }
    }

    /// Resolve the theme directory name against the repository layout.
    pub fn resolve_theme_dir(&self, name: &str) -> Option<PathBuf> {
        let cand1 = self.workspace_root.join("themes").join(name);
        if cand1.is_dir() {
            return Some(cand1);
        }
        let cand2 = self.workspace_root.join("theme-generator").join("themes").join(name);
        if cand2.is_dir() {
            return Some(cand2);
        }
        None
    }

    /// Start the theme dev server using the provided Supervisor.
    /// This spawns `npm run dev` in the theme directory and returns the PID.
    pub async fn start_theme_async(&self, supervisor: &dyn SupervisorTrait, name: &str, theme_url: &str, max_wait: Option<Duration>) -> anyhow::Result<u32> {
        let dir = self.resolve_theme_dir(name).ok_or_else(|| anyhow::anyhow!("theme not found"))?;
        println!("Starting theme in: {}", dir.display());

        // In test-mode we may want to avoid spawning external binaries like
        // `npm`. Honor SHIMMY_ORCH_SKIP_THEMES=1 to skip spawning and only
        // poll for theme readiness.
        let skip_themes = std::env::var("SHIMMY_ORCH_SKIP_THEMES").ok().map(|v| v == "1").unwrap_or(false);
        let pid = if skip_themes {
            println!("[orchestrator] SHIMMY_ORCH_SKIP_THEMES=1 — skipping theme spawn (test mode)");
            // Use 0 as a sentinel PID to indicate we did not spawn a process.
            0
        } else {
            // Spawn theme using the single, deterministic approach:
            // 1) Allow override via ORCH_THEME_RUNNER env var (developer/deployment override).
            // 2) Otherwise prefer pnpm -> yarn -> npm in that order if available.
            // 3) Use a platform shell wrapper so the user's PATH / shims are respected.

            // Resolve final runner string (e.g. "pnpm" or "npm")
            let runner = std::env::var("ORCH_THEME_RUNNER").ok().unwrap_or_else(|| {
                // prefer pnpm, then yarn, then npm deterministically
                if which::which("pnpm").is_ok() { "pnpm".to_string() }
                else if which::which("yarn").is_ok() { "yarn".to_string() }
                else { "npm".to_string() }
            });

            // Determine platform shell command + args to ensure consistent spawns
            #[cfg(target_os = "windows")]
            let (shell_cmd, shell_args) = ("cmd", vec!["/C".to_string(), format!("{} run dev", runner)]);

            #[cfg(not(target_os = "windows"))]
            let (shell_cmd, shell_args) = ("sh", vec!["-lc".to_string(), format!("{} run dev", runner)]);

            // The supervisor spawns commands from the cwd of the process, so change directory.
            // Perform spawn inside a block so we can restore cwd reliably.
            let pid = {
                let prev_cwd = std::env::current_dir()?;
                std::env::set_current_dir(&dir)?;

                // Convert args into slice of &str for supervisor API
                let shell_args_refs: Vec<&str> = shell_args.iter().map(|s| s.as_str()).collect();

                // Spawn and return PID (propagate any error)
                let pid = supervisor.spawn_and_log(shell_cmd, &shell_args_refs[..], "theme").await?;

                // restore cwd before returning
                std::env::set_current_dir(prev_cwd)?;
                pid
            };
            pid
        };

        // Poll theme URL until SHIMMY appears; do blocking network I/O in a
        // spawn_blocking task so we don't accidentally drop blocking clients
        // inside the async runtime (which can cause tokio to panic).
        let theme_url = theme_url.to_string();
        // Clone the theme name so the blocking closure owns everything it needs (no short-lived borrows)
        let theme_name_owned = name.to_string();
        let result_pid = tokio::task::spawn_blocking(move || -> anyhow::Result<u32> {
            let client = reqwest::blocking::Client::builder().timeout(std::time::Duration::from_secs(5)).build()?;
            let start = std::time::Instant::now();
            // If max_wait is Some(d) then limit; otherwise wait indefinitely
            loop {
                if let Some(max) = max_wait {
                    if start.elapsed() >= max {
                        anyhow::bail!("theme failed to become ready within {:?}", max)
                    }
                }
                match client.get(&theme_url).send() {
                    Ok(resp) => {
                        println!("[orchestrator] theme probe status={} for {}", resp.status(), &theme_url);
                        if let Ok(body) = resp.text() {
                            println!("[orchestrator] theme probe body_len={} for {}", body.len(), &theme_url);
                            // Consider the theme ready when the body contains a visible
                            // theme marker. We accept either a `SHIMMY` marker or
                            // the theme name (case-insensitive). This makes checks
                            // robust for generated themes like `shimmy-default`.
                            let body_l = body.to_lowercase();
                            let theme_name_l = theme_name_owned.to_lowercase();
                            if body_l.contains("shimmy") || body_l.contains(&theme_name_l) {
                                return Ok(pid);
                            }
                        }
                    }
                    Err(e) => {
                        println!("[orchestrator] theme probe err={:?} for {}", e, &theme_url);
                    }
                }
                std::thread::sleep(Duration::from_secs(1));
            }
        }).await??;

        Ok(result_pid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn resolve_theme_dir_works() {
        let tmp = TempDir::new().unwrap();
        let themes = tmp.path().join("themes");
        std::fs::create_dir_all(&themes.join("my-theme")).unwrap();

        let m = ThemeManager::new(tmp.path());
        let p = m.resolve_theme_dir("my-theme");
        assert!(p.is_some());
    }
}
