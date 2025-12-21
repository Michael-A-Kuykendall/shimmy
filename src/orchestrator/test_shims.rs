use async_trait::async_trait;
use crate::orchestrator::supervisor::SupervisorTrait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct FakeSupervisor {
    inner: Arc<Mutex<HashMap<u32, Option<i32>>>>,
    next: Arc<Mutex<u32>>,
    // record last spawn invocation for assertions in tests
    pub last_cmd: Arc<Mutex<Option<String>>>,
    pub last_args: Arc<Mutex<Vec<String>>>,
}

impl FakeSupervisor {
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(HashMap::new())), next: Arc::new(Mutex::new(100)), last_cmd: Arc::new(Mutex::new(None)), last_args: Arc::new(Mutex::new(vec![])) }
    }
}

#[async_trait]
impl SupervisorTrait for FakeSupervisor {
    async fn spawn_and_log(&self, cmd: &str, args: &[&str], _log_name: &str) -> anyhow::Result<u32> {
        let mut n = self.next.lock().await;
        *n += 1;
        let pid = *n;
        self.inner.lock().await.insert(pid, Some(0));
        // record command and args for assertions
        let mut last_cmd = self.last_cmd.lock().await;
        *last_cmd = Some(cmd.to_string());
        let mut last_args = self.last_args.lock().await;
        last_args.clear();
        for a in args { last_args.push(a.to_string()); }
        Ok(pid)
    }

    async fn spawn_and_log_with_env(&self, cmd: &str, args: &[&str], env_pairs: &[(&str, &str)], _log_name: &str) -> anyhow::Result<u32> {
        let mut n = self.next.lock().await;
        *n += 1;
        let pid = *n;
        self.inner.lock().await.insert(pid, Some(0));

        // record command and args for assertions
        let mut last_cmd = self.last_cmd.lock().await;
        *last_cmd = Some(cmd.to_string());
        let mut last_args = self.last_args.lock().await;
        last_args.clear();
        for a in args { last_args.push(a.to_string()); }
        for (k, v) in env_pairs { last_args.push(format!("{}={}", k, v)); }

        Ok(pid)
    }

    async fn stop(&self, _pid: u32) -> anyhow::Result<()> { Ok(()) }

    async fn wait_for_exit(&self, pid: u32) -> anyhow::Result<Option<i32>> {
        let mut inner = self.inner.lock().await;
        Ok(inner.remove(&pid).unwrap_or(Some(0)))
    }

    async fn try_wait(&self, pid: u32) -> anyhow::Result<Option<Option<i32>>> {
        // In fake supervisor, processes never exit on their own
        let inner = self.inner.lock().await;
        if inner.contains_key(&pid) {
            Ok(None) // still running
        } else {
            Ok(Some(Some(0))) // not tracked = already exited
        }
    }

    async fn list(&self) -> Vec<u32> {
        self.inner.lock().await.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fake_spawn_records_cmd() {
        let sup = FakeSupervisor::new();
        let pid = sup.spawn_and_log("echo", &["hello"], "log").await.unwrap();
        assert!(pid > 100);
        let cmd = sup.last_cmd.lock().await.clone();
        assert_eq!(cmd.unwrap(), "echo");
    }

    #[tokio::test]
    async fn fake_spawn_with_env_records_env_pairs() {
        let sup = FakeSupervisor::new();
        let pid = sup.spawn_and_log_with_env("node", &["validator.js"], &[("SHIMMY_DISCOVERY_URL", "http://127.0.0.1:11430")], "validator").await.unwrap();
        assert!(pid > 100);
        let args = sup.last_args.lock().await.clone();
        // Expect the env pair to have been appended to the recorded args
        assert!(args.iter().any(|a| a.contains("SHIMMY_DISCOVERY_URL=http://127.0.0.1:11430")));
    }
}
