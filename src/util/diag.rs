use axum::Json;
use serde::Serialize;
use sysinfo::System;

#[derive(Serialize)]
pub struct Diag {
    os: String,
    cores: usize,
    mem_total_mb: u64,
}

pub async fn diag_handler() -> Json<Diag> {
    let mut sys = System::new_all();
    sys.refresh_all();
    // Some sysinfo methods changed across versions; keep it minimal & portable.
    let os = std::env::consts::OS.to_string();
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(0);
    let mem_total_mb = sys.total_memory() / 1024; // KiB -> MiB
    Json(Diag {
        os,
        cores,
        mem_total_mb,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn diag_handler_returns_valid_fields() {
        let Json(diag) = diag_handler().await;
        assert!(!diag.os.is_empty(), "os must be populated");
        assert!(diag.cores >= 1, "cores must be >= 1 on this machine");
        assert!(diag.mem_total_mb > 0, "total memory must be non-zero");
    }
}
