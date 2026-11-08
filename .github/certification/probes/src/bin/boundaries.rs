use airframe::runtime::gpu::GpuRuntime;
use airframe_observe::quant_formula;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::{Read, Seek, SeekFrom};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<_> = std::env::args().collect();
    let contract: Value = serde_json::from_slice(&std::fs::read(&args[2])?)?;
    let path = std::path::Path::new(&args[1]);
    let runtime = GpuRuntime::load_with_context(path, Some(2048)).await?;
    assert_eq!(
        runtime.model_identity().model_sha256,
        contract["model_sha256"]
    );
    let manifest = shimmytok::manifest::load_manifest(path)?;
    let tensor = manifest
        .tensor("token_embd.weight")
        .ok_or("missing embedding")?;
    assert_eq!(tensor.absolute_offset(), runtime.embd_weight_offset());
    assert_eq!(tensor.row_bytes(), runtime.row_bytes());
    assert_eq!(tensor.ggml_type(), 12);
    let mut file = std::fs::File::open(path)?;
    let mut results = Vec::new();
    for token in [11_u32, 151644] {
        let record = contract["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["token_id"] == token)
            .ok_or("missing frozen row")?;
        let mut raw = vec![0_u8; runtime.row_bytes() as usize];
        file.seek(SeekFrom::Start(
            tensor.absolute_offset() + u64::from(token) * runtime.row_bytes(),
        ))?;
        file.read_exact(&mut raw)?;
        let expected: Vec<f32> = (0..runtime.spec().n_embd)
            .map(|i| {
                quant_formula::dequant_elem(12, &raw[(i / 256) * 144..(i / 256 + 1) * 144], i % 256)
                    .unwrap()
            })
            .collect();
        let hash = format!(
            "{:x}",
            Sha256::digest(
                expected
                    .iter()
                    .flat_map(|v| v.to_le_bytes())
                    .collect::<Vec<_>>()
            )
        );
        assert_eq!(
            Some(hash.as_str()),
            record["f32_sha256"].as_str(),
            "frozen independent reference changed"
        );
        let mut observation = runtime.diagnose_embedding_storage(token, &raw, &expected);
        observation["reference_sha256"] = json!(hash);
        observation["raw_cpu_sha256"] = json!(format!("{:x}", Sha256::digest(&raw)));
        results.push(observation);
        std::fs::write(
            &args[3],
            serde_json::to_vec_pretty(
                &json!({"complete":false,"model_sha256":runtime.model_identity().model_sha256,"rows":results}),
            )?,
        )?;
    }
    std::fs::write(
        &args[3],
        serde_json::to_vec_pretty(
            &json!({"complete":true,"model_sha256":runtime.model_identity().model_sha256,"rows":results}),
        )?,
    )?;
    println!("BOUNDARY_OBSERVATIONS_COMPLETE");
    Ok(())
}
