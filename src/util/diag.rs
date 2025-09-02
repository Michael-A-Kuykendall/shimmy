use axum::Json;
use serde::Serialize;
use sysinfo::System;

#[derive(Serialize)]
pub struct Diag { os: String, cores: usize, mem_total_mb: u64 }

pub async fn diag_handler() -> Json<Diag> {
    let mut sys = System::new_all();
    sys.refresh_all();
    // Some sysinfo methods changed across versions; keep it minimal & portable.
    let os = std::env::consts::OS.to_string();
    let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0);
    let mem_total_mb = sys.total_memory() / 1024; // KiB -> MiB
    Json(Diag { os, cores, mem_total_mb })
}
