use airframe::runtime::gpu::GpuRuntime;
use airframe_observe::quant_formula;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::{Read, Seek, SeekFrom};
fn hash(values: &[f32]) -> String {
    let mut hash = Sha256::new();
    for value in values {
        hash.update(value.to_le_bytes());
    }
    format!("{:x}", hash.finalize())
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<_> = std::env::args().collect();
    let contract: Value = serde_json::from_slice(&std::fs::read(&args[2])?)?;
    let model = std::path::Path::new(&args[1]);
    let runtime = GpuRuntime::load_with_context(model, Some(2048)).await?;
    assert_eq!(
        runtime.model_identity().model_sha256,
        contract["model_sha256"]
    );
    let manifest = shimmytok::manifest::load_manifest(model)?;
    let tensor = manifest
        .tensor("token_embd.weight")
        .ok_or("missing embedding")?;
    let kind = tensor.ggml_type();
    assert_eq!(kind, 12);
    let width = runtime.spec().n_embd;
    assert_eq!(width, 2048);
    let bytes = quant_formula::block_bytes(kind).ok_or("unknown quantization")?;
    let elements = quant_formula::block_elems(kind).ok_or("unknown quantization")?;
    let row_size = width / elements * bytes;
    assert_eq!(row_size, 1152);
    let mut file = std::fs::File::open(model)?;
    let mut rows = Vec::new();
    for row in contract["rows"].as_array().ok_or("missing rows")? {
        let token = row["token_id"].as_u64().ok_or("invalid token")? as u32;
        file.seek(SeekFrom::Start(
            tensor.absolute_offset() + u64::from(token) * row_size as u64,
        ))?;
        let mut raw = vec![0_u8; row_size];
        file.read_exact(&mut raw)?;
        let expected: Vec<f32> = (0..width)
            .map(|i| {
                quant_formula::dequant_elem(
                    kind,
                    &raw[(i / elements) * bytes..(i / elements + 1) * bytes],
                    i % elements,
                )
                .unwrap()
            })
            .collect();
        assert_eq!(
            hash(&expected),
            row["f32_sha256"].as_str().ok_or("missing reference SHA")?,
            "independent frozen FP32 row mismatch"
        );
        let actual = runtime.dequant_token_embd(token);
        assert_eq!(actual.len(), expected.len());
        let maximum = actual
            .iter()
            .zip(&expected)
            .map(|(a, b)| f64::from(*a) - f64::from(*b))
            .map(f64::abs)
            .fold(0.0_f64, f64::max);
        let rms = (actual
            .iter()
            .zip(&expected)
            .map(|(a, b)| (f64::from(*a) - f64::from(*b)).powi(2))
            .sum::<f64>()
            / width as f64)
            .sqrt();
        let pass = maximum <= 0.00002 && actual.iter().all(|v| v.is_finite());
        rows.push(json!({"token_id":token,"max_abs":maximum,"rms":rms,"pass":pass,"actual_sha256":hash(&actual),"reference_sha256":hash(&expected),"actual_zero_count":actual.iter().filter(|v|**v==0.0).count(),"reference_zero_count":expected.iter().filter(|v|**v==0.0).count(),"actual_first4":&actual[..4],"reference_first4":&expected[..4]}));
        std::fs::write(
            &args[3],
            serde_json::to_vec_pretty(
                &json!({"complete":false,"model_sha256":runtime.model_identity().model_sha256,"rows":rows}),
            )?,
        )?;
    }
    assert_eq!(rows.len(), 33);
    let pass = rows.iter().all(|r| r["pass"] == true);
    std::fs::write(
        &args[3],
        serde_json::to_vec_pretty(
            &json!({"complete":true,"model_sha256":runtime.model_identity().model_sha256,"status":if pass{"PASS"}else{"FAIL"},"rows":rows}),
        )?,
    )?;
    if !pass {
        return Err("METAL_EMBEDDING_DIFFERENTIAL_FAILURE".into());
    }
    println!("EMBEDDING_ROWS_PASS 33");
    Ok(())
}
