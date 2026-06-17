//! Pure-Rust text embeddings via a candle-transformers BERT sentence-transformer.
//!
//! Implements the standard sentence-transformers recipe: run a BERT encoder,
//! mean-pool the last hidden state over the sequence dimension *using the
//! attention mask* (so padding tokens are ignored), then L2-normalize each
//! row. The default model is `sentence-transformers/all-MiniLM-L6-v2` (384-dim).
//!
//! This module is gated behind the `embeddings` cargo feature. candle is
//! pure-Rust, keeping shimmy's post-v2.0 no-C++ philosophy intact.

use std::sync::Arc;

use anyhow::{Context, Result};
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use tokenizers::Tokenizer;

/// Default sentence-transformer model id (overridable via `SHIMMY_EMBED_MODEL`).
pub const DEFAULT_EMBED_MODEL: &str = "sentence-transformers/all-MiniLM-L6-v2";

/// A loaded BERT sentence-transformer ready to produce embeddings.
pub struct Embedder {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
    dim: usize,
}

impl Embedder {
    /// Hidden size (embedding dimensionality) of the loaded model.
    #[allow(dead_code)] // used by the network integration test and external callers
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// Resolve config/tokenizer/weights from the HF hub and build the model on CPU.
    ///
    /// Prefers `model.safetensors`; falls back to `pytorch_model.bin` only when
    /// safetensors is unavailable.
    pub async fn load(model_id: &str) -> Result<Embedder> {
        use hf_hub::api::tokio::Api;

        let api = Api::new().context("failed to construct HF hub API client")?;
        let repo = api.model(model_id.to_string());

        let config_path = repo
            .get("config.json")
            .await
            .with_context(|| format!("failed to fetch config.json for '{model_id}'"))?;
        let tokenizer_path = repo
            .get("tokenizer.json")
            .await
            .with_context(|| format!("failed to fetch tokenizer.json for '{model_id}'"))?;

        let config_bytes =
            std::fs::read(&config_path).context("failed to read downloaded config.json")?;
        let config: Config =
            serde_json::from_slice(&config_bytes).context("failed to parse BERT config.json")?;
        let dim = config.hidden_size;

        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow::anyhow!("failed to load tokenizer.json: {e}"))?;

        let device = Device::Cpu;

        // Prefer safetensors; fall back to pytorch_model.bin.
        let vb = match repo.get("model.safetensors").await {
            Ok(weights_path) => {
                // SAFETY: the file is a trusted, freshly-downloaded safetensors
                // checkpoint; mmap lifetime is owned by the resulting VarBuilder.
                unsafe {
                    VarBuilder::from_mmaped_safetensors(&[weights_path], DTYPE, &device)
                        .context("failed to mmap model.safetensors")?
                }
            }
            Err(_) => {
                let weights_path = repo.get("pytorch_model.bin").await.with_context(|| {
                    format!("no model.safetensors or pytorch_model.bin for '{model_id}'")
                })?;
                VarBuilder::from_pth(&weights_path, DTYPE, &device)
                    .context("failed to load pytorch_model.bin")?
            }
        };

        let model = BertModel::load(vb, &config).context("failed to build BERT model")?;

        Ok(Embedder {
            model,
            tokenizer,
            device,
            dim,
        })
    }

    /// Embed a batch of texts. Synchronous candle work — callers should wrap
    /// this in `tokio::task::spawn_blocking`.
    pub fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Tokenize with padding to the batch's max length so every row has the
        // same sequence length; the attention mask records the real tokens.
        let mut tokenizer = self.tokenizer.clone();
        tokenizer
            .with_padding(Some(tokenizers::PaddingParams {
                strategy: tokenizers::PaddingStrategy::BatchLongest,
                ..Default::default()
            }))
            .with_truncation(None)
            .map_err(|e| anyhow::anyhow!("failed to configure tokenizer truncation: {e}"))?;

        let encodings = tokenizer
            .encode_batch(texts.to_vec(), true)
            .map_err(|e| anyhow::anyhow!("tokenization failed: {e}"))?;

        let batch = encodings.len();
        let seq = encodings.first().map(|e| e.get_ids().len()).unwrap_or(0);

        let mut ids: Vec<u32> = Vec::with_capacity(batch * seq);
        let mut mask: Vec<u32> = Vec::with_capacity(batch * seq);
        for enc in &encodings {
            ids.extend_from_slice(enc.get_ids());
            mask.extend_from_slice(enc.get_attention_mask());
        }

        let input_ids = Tensor::from_vec(ids, (batch, seq), &self.device)?;
        let attention_mask = Tensor::from_vec(mask, (batch, seq), &self.device)?;
        // BERT requires token_type_ids; for single-sentence inputs they are all zero.
        let token_type_ids = input_ids.zeros_like()?;

        // last_hidden_state: [batch, seq, hidden]
        let hidden = self
            .model
            .forward(&input_ids, &token_type_ids, Some(&attention_mask))?;

        let hidden_vals: Vec<f32> = hidden.flatten_all()?.to_vec1()?;
        let mask_f32: Vec<f32> = attention_mask.to_dtype(DTYPE)?.flatten_all()?.to_vec1()?;

        let mut out = Vec::with_capacity(batch);
        for b in 0..batch {
            let hidden_off = b * seq * self.dim;
            let mask_off = b * seq;
            let mut pooled = mean_pool_masked(
                &hidden_vals[hidden_off..hidden_off + seq * self.dim],
                &mask_f32[mask_off..mask_off + seq],
                seq,
                self.dim,
            );
            l2_normalize(&mut pooled);
            out.push(pooled);
        }

        Ok(out)
    }
}

/// Mean-pool a `[seq, dim]` hidden-state block over the sequence dimension,
/// weighting each token by its attention mask so padding rows are ignored:
/// `sum(hidden * mask) / sum(mask)`. Returns a `dim`-length vector.
fn mean_pool_masked(hidden: &[f32], mask: &[f32], seq: usize, dim: usize) -> Vec<f32> {
    let mut acc = vec![0.0f32; dim];
    let mut mask_sum = 0.0f32;
    for s in 0..seq {
        let m = mask[s];
        if m == 0.0 {
            continue;
        }
        mask_sum += m;
        let row = &hidden[s * dim..s * dim + dim];
        for (a, &h) in acc.iter_mut().zip(row.iter()) {
            *a += h * m;
        }
    }
    // Guard against an all-padding (mask_sum == 0) row producing NaNs.
    let denom = if mask_sum > 0.0 { mask_sum } else { 1.0 };
    for a in acc.iter_mut() {
        *a /= denom;
    }
    acc
}

/// L2-normalize a vector in place. A zero vector is left unchanged (no NaN).
fn l2_normalize(v: &mut [f32]) {
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

// Module-level lazily-initialized embedder singleton. Initialized on first
// embeddings request; subsequent requests reuse the loaded model.
static EMBEDDER: tokio::sync::OnceCell<Arc<Embedder>> = tokio::sync::OnceCell::const_new();

/// Resolve the configured default model id, honoring `SHIMMY_EMBED_MODEL`.
pub fn default_model_id() -> String {
    std::env::var("SHIMMY_EMBED_MODEL").unwrap_or_else(|_| DEFAULT_EMBED_MODEL.to_string())
}

/// Get (or lazily initialize) the process-wide embedder. The `model_id` only
/// selects the model on first init; later calls reuse the already-loaded model
/// regardless of the requested id (v1 supports a single configured model).
pub async fn get_or_init_embedder(model_id: &str) -> Result<Arc<Embedder>> {
    let embedder = EMBEDDER
        .get_or_try_init(|| async {
            let embedder = Embedder::load(model_id).await?;
            Ok::<_, anyhow::Error>(Arc::new(embedder))
        })
        .await?;
    Ok(Arc::clone(embedder))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mean_pool_ignores_padding() {
        // seq=3, dim=2. Rows: [1,1], [3,3], [99,99]. Mask keeps first two.
        let hidden = vec![1.0, 1.0, 3.0, 3.0, 99.0, 99.0];
        let mask = vec![1.0, 1.0, 0.0];
        let pooled = mean_pool_masked(&hidden, &mask, 3, 2);
        // mean of [1,3] = 2 for each dim; padding row (99) must be ignored.
        assert_eq!(pooled, vec![2.0, 2.0]);
    }

    #[test]
    fn mean_pool_all_masked_is_safe() {
        let hidden = vec![5.0, 5.0, 5.0, 5.0];
        let mask = vec![0.0, 0.0];
        let pooled = mean_pool_masked(&hidden, &mask, 2, 2);
        assert!(pooled.iter().all(|x| x.is_finite()));
        assert_eq!(pooled, vec![0.0, 0.0]);
    }

    #[test]
    fn l2_normalize_unit_norm() {
        let mut v = vec![3.0, 4.0];
        l2_normalize(&mut v);
        let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
        assert!((v[0] - 0.6).abs() < 1e-6);
        assert!((v[1] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn l2_normalize_zero_vector_is_safe() {
        let mut v = vec![0.0, 0.0, 0.0];
        l2_normalize(&mut v);
        assert!(v.iter().all(|x| x.is_finite()));
        assert_eq!(v, vec![0.0, 0.0, 0.0]);
    }

    fn cosine(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    #[tokio::test]
    #[ignore = "downloads model from HF hub"]
    async fn real_model_embeds_and_ranks_similarity() {
        let embedder = Embedder::load(DEFAULT_EMBED_MODEL).await.unwrap();
        assert_eq!(embedder.dim(), 384);

        let texts = vec![
            "A cat sat on the mat.".to_string(),
            "A kitten rested on the rug.".to_string(),
            "Quarterly tax filings are due in April.".to_string(),
        ];
        let embs = embedder.embed_batch(&texts).unwrap();
        assert_eq!(embs.len(), 3);
        for e in &embs {
            assert_eq!(e.len(), 384);
            assert!(e.iter().all(|x| x.is_finite()));
        }

        let sim_similar = cosine(&embs[0], &embs[1]);
        let sim_dissimilar = cosine(&embs[0], &embs[2]);
        assert!(
            sim_similar > sim_dissimilar,
            "similar pair {sim_similar} should exceed dissimilar pair {sim_dissimilar}"
        );
    }
}
