use std::process::Command;
use std::time::Duration;

// This is a guarded, heavy-weight integration test which starts the stack and
// runs the canonical stack verification. It is skipped by default and only
// runs when RUN_REAL_THEME_INTEGRATION=1 is set in the environment.

#[tokio::test]
async fn real_theme_orchestration_integration() {
    if std::env::var("RUN_REAL_THEME_INTEGRATION").ok().as_deref() != Some("1") {
        eprintln!("skipping heavy real-theme integration (set RUN_REAL_THEME_INTEGRATION=1 to enable)");
        return;
    }

    // Start the Rust orchestrator to run the full dev lifecycle + verification.
    // Try the release binary first; fall back to cargo run if it's not present.
    let start_cmd = "./target/release/shimmy dev 32bit --verify --no-build || cargo run --release --bin shimmy -- dev 32bit --verify --no-build";
    let mut child = Command::new("bash")
        .arg("-c")
        .arg(start_cmd)
        .spawn()
        .expect("failed to spawn orchestrator dev flow");

    // WAIT for orchestrator to complete verification and exit
    let status = child.wait().expect("failed waiting for orchestrator");
    assert!(status.success(), "orchestrator verification failed — check logs and artifacts");

    // Attempt best-effort cleanup: kill shimmy and node background processes
    let _ = Command::new("bash").arg("-c").arg("pkill -f shimmy || true").status();
    let _ = Command::new("bash").arg("-c").arg("pkill -f node || true").status();

    // Wait a moment for processes to die
    tokio::time::sleep(Duration::from_secs(2)).await;
}
