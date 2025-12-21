use shimmy::orchestrator::OrchestratorConfig;
use std::net::TcpListener;
use std::thread;
use tempfile::TempDir;
use std::fs;

mod common;
use shimmy::orchestrator::test_shims::FakeSupervisor;

#[tokio::test]
async fn build_failure_bubbles_up() {
    // Discovery ok
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind failed");
    let addr = listener.local_addr().unwrap();
    let _handle = thread::spawn(move || {
        for stream in listener.incoming().take(1) {
            if let Ok(mut s) = stream {
                use std::io::{Write, Read};
                let mut buf = [0u8; 1024];
                let _ = s.read(&mut buf);
                let _ = write!(s, "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}", "{}");
                let _ = s.flush();
            }
        }
    });

    // Theme ok
    let tmp = TempDir::new().unwrap();
    let theme_dir = tmp.path().join("themes").join("my-test-theme");
    fs::create_dir_all(&theme_dir).unwrap();
    fs::write(theme_dir.join("index.html"), "<html>SHIMMY</html>").unwrap();

    let theme_listener = TcpListener::bind("127.0.0.1:0").expect("bind failed");
    let theme_addr = theme_listener.local_addr().unwrap();
    let theme_file = theme_dir.join("index.html");
    let _theme_handle = thread::spawn(move || {
        for stream in theme_listener.incoming().take(1) {
            if let Ok(mut s) = stream {
                use std::io::{Write, Read};
                let mut buf = [0u8; 1024];
                let _ = s.read(&mut buf);
                let body = std::fs::read_to_string(&theme_file).unwrap_or_else(|_| "".into());
                let _ = write!(s, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}", body.len(), body);
                let _ = s.flush();
            }
        }
    });

    let mut cfg = OrchestratorConfig::default();
    cfg.discovery_url = format!("http://127.0.0.1:{}/api/discovery", addr.port());
    cfg.theme_url = format!("http://127.0.0.1:{}", theme_addr.port());

    // Supervisor that fails on the first wait_for_exit (build)
    struct FailBuildSup { inner: FakeSupervisor, first: std::sync::Mutex<bool> }
    #[async_trait::async_trait]
    impl shimmy::orchestrator::supervisor::SupervisorTrait for FailBuildSup {
        async fn spawn_and_log(&self, cmd: &str, args: &[&str], log_name: &str) -> anyhow::Result<u32> {
            self.inner.spawn_and_log(cmd, args, log_name).await
        }
        async fn spawn_and_log_with_env(&self, cmd: &str, args: &[&str], env_pairs: &[(&str, &str)], log_name: &str) -> anyhow::Result<u32> {
            self.inner.spawn_and_log_with_env(cmd, args, env_pairs, log_name).await
        }
        async fn stop(&self, pid: u32) -> anyhow::Result<()> { self.inner.stop(pid).await }
        async fn wait_for_exit(&self, _pid: u32) -> anyhow::Result<Option<i32>> {
            let mut f = self.first.lock().unwrap();
            if *f {
                *f = false;
                Ok(Some(1))
            } else {
                Ok(Some(0))
            }
        }
        async fn try_wait(&self, _pid: u32) -> anyhow::Result<Option<Option<i32>>> {
            Ok(Some(Some(0)))
        }
        async fn list(&self) -> Vec<u32> { self.inner.list().await }
    }

    let sys_sup = FailBuildSup { inner: FakeSupervisor::new(), first: std::sync::Mutex::new(true) };
    let theme_sup = FakeSupervisor::new();

    // Run lifecycle - build should fail and produce an error
    let res = shimmy::orchestrator::lifecycle::run_lifecycle_with_supervisors(&cfg, "my-test-theme", false, false, &sys_sup, &theme_sup).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn discovery_timeout() {
    // point at a non-existent local port and force a short wait
    let mut cfg = OrchestratorConfig::default();
    cfg.discovery_url = "http://127.0.0.1:9/api/discovery".to_string();
    cfg.max_discovery_wait = Some(std::time::Duration::from_secs(2));
    cfg.theme_url = "http://127.0.0.1:1".to_string();

    let sup = FakeSupervisor::new();
    let res = shimmy::orchestrator::lifecycle::run_lifecycle_with_supervisors(&cfg, "nonexistent", false, true, &sup, &sup).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn theme_readiness_failure() {
    // Discovery ok
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind failed");
    let addr = listener.local_addr().unwrap();
    let _handle = thread::spawn(move || {
        for stream in listener.incoming().take(1) {
            if let Ok(mut s) = stream {
                use std::io::{Write, Read};
                let mut buf = [0u8; 1024];
                let _ = s.read(&mut buf);
                let _ = write!(s, "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}", "{}");
                let _ = s.flush();
            }
        }
    });

    // Theme server responds but doesn't include SHIMMY
    let tmp = TempDir::new().unwrap();
    let theme_dir = tmp.path().join("themes").join("my-test-theme");
    fs::create_dir_all(&theme_dir).unwrap();
    fs::write(theme_dir.join("index.html"), "<html>NOT_READY</html>").unwrap();

    let theme_listener = TcpListener::bind("127.0.0.1:0").expect("bind failed");
    let theme_addr = theme_listener.local_addr().unwrap();
    let theme_file = theme_dir.join("index.html");
    let _theme_handle = thread::spawn(move || {
        for stream in theme_listener.incoming().take(1) {
            if let Ok(mut s) = stream {
                use std::io::{Write, Read};
                let mut buf = [0u8; 1024];
                let _ = s.read(&mut buf);
                let body = std::fs::read_to_string(&theme_file).unwrap_or_else(|_| "".into());
                let _ = write!(s, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}", body.len(), body);
                let _ = s.flush();
            }
        }
    });

    let mut cfg = OrchestratorConfig::default();
    cfg.discovery_url = format!("http://127.0.0.1:{}/api/discovery", addr.port());
    cfg.theme_url = format!("http://127.0.0.1:{}", theme_addr.port());
    cfg.max_theme_wait = Some(std::time::Duration::from_secs(2));

    // run and expect error because theme never reports SHIMMY
    let sys_sup = FakeSupervisor::new();
    let theme_sup = FakeSupervisor::new();
    let res = shimmy::orchestrator::lifecycle::run_lifecycle_with_supervisors(&cfg, "my-test-theme", false, true, &sys_sup, &theme_sup).await;
    assert!(res.is_err());
}
