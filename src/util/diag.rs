use axum::Json;
use serde::Serialize;
use sysinfo::{System, SystemExt};

#[derive(Serialize)]
pub struct Diag { os: String, cores: usize, mem_total_gb: f32 }

pub async fn diag_handler() -> Json<Diag> {
    let mut sys = System::new_all();
    sys.refresh_all();
    Json(Diag {
        os: format!(\"{} {}\", sys.name().unwrap_or_default(), sys.kernel_version().unwrap_or_default()),
        cores: sys.physical_core_count().unwrap_or(0),
        mem_total_gb: (sys.total_memory() as f32)/(1024.0*1024.0),
    })
}

