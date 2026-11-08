use airframe::runtime::gpu::GpuRuntime;
use sha2::{Digest, Sha256};
use std::io::Read;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<_> = std::env::args().collect();
    let contract: serde_json::Value = serde_json::from_slice(&std::fs::read(&args[2])?)?;
    let path = std::path::Path::new(&args[1]);
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut block = vec![0; 1024 * 1024];
    loop {
        let n = file.read(&mut block)?;
        if n == 0 {
            break;
        }
        hasher.update(&block[..n]);
    }
    let hash = format!("{:x}", hasher.finalize());
    assert_eq!(Some(hash.as_str()), contract["model_sha256"].as_str());
    GpuRuntime::diagnose_buffer_upload(path, std::path::Path::new(&args[3])).await?;
    let mut result: serde_json::Value = serde_json::from_slice(&std::fs::read(&args[3])?)?;
    result["model_sha256"] = serde_json::json!(hash);
    std::fs::write(&args[3], serde_json::to_vec_pretty(&result)?)?;
    println!("UPLOAD_OBSERVATIONS_COMPLETE");
    Ok(())
}
