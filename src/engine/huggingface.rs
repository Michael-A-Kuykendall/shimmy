use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::path::Path;
use std::process::Command;
use tokio::process::Command as TokioCommand;

use super::{GenOptions, ModelBackend, UniversalEngine, UniversalModel, UniversalModelSpec};

pub struct HuggingFaceEngine {
    python_path: String,
}

impl HuggingFaceEngine {
    pub fn new() -> Self {
        // Use the verified Python path from CLAUDE.md
        Self {
            python_path: "C:/Python311/python.exe".to_string(),
        }
    }
    
}

#[async_trait]
impl UniversalEngine for HuggingFaceEngine {
    async fn load(&self, spec: &UniversalModelSpec) -> Result<Box<dyn UniversalModel>> {
        match &spec.backend {
            ModelBackend::HuggingFace { base_model_id, peft_path, use_local } => {
                let model = HuggingFaceModel::load(
                    &self.python_path,
                    base_model_id,
                    peft_path.as_deref(),
                    *use_local,
                    &spec.device,
                ).await?;
                Ok(Box::new(model))
            }
            _ => Err(anyhow!("HuggingFaceEngine only supports HuggingFace backend")),
        }
    }
}

struct HuggingFaceModel {
    python_path: String,
    base_model_id: String,
    peft_path: Option<String>,
    device: String,
}

impl HuggingFaceModel {
    async fn load(
        python_path: &str,
        base_model_id: &str,
        peft_path: Option<&Path>,
        _use_local: bool,
        device: &str,
    ) -> Result<Self> {
        // Verify Python and transformers availability
        let output = Command::new(python_path)
            .args(["-c", "import torch, transformers, peft; print('OK')"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "Python dependencies missing. Install: pip install torch transformers peft\nError: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Verify model can be loaded (quick check)
        let verify_cmd = vec![
            "-c".to_string(),
            format!(
                r#"
import torch
from transformers import AutoTokenizer, AutoModelForCausalLM
from peft import PeftModel
import sys

try:
    print("Loading base model...", file=sys.stderr)
    model = AutoModelForCausalLM.from_pretrained('{}', torch_dtype=torch.float16)
    
    {}
    
    print("SUCCESS: Model loaded", file=sys.stderr)
    print("OK")
except Exception as e:
    print(f"ERROR: {{e}}", file=sys.stderr)
    sys.exit(1)
"#,
                base_model_id,
                if let Some(peft_path) = peft_path {
                    format!(
                        r#"print("Loading PEFT adapter...", file=sys.stderr)
    model = PeftModel.from_pretrained(model, '{}')
    print("PEFT adapter loaded", file=sys.stderr)"#,
                        peft_path.display()
                    )
                } else {
                    "".to_string()
                }
            ),
        ];

        let verify_output = Command::new(python_path)
            .args(&verify_cmd)
            .output()?;

        if !verify_output.status.success() {
            return Err(anyhow!(
                "Failed to load HuggingFace model '{}': {}",
                base_model_id,
                String::from_utf8_lossy(&verify_output.stderr)
            ));
        }

        Ok(HuggingFaceModel {
            python_path: python_path.to_string(),
            base_model_id: base_model_id.to_string(),
            peft_path: peft_path.map(|p| p.to_string_lossy().to_string()),
            device: device.to_string(),
        })
    }
}

#[async_trait]
impl UniversalModel for HuggingFaceModel {
    async fn generate(
        &self,
        prompt: &str,
        opts: GenOptions,
        on_token: Option<Box<dyn FnMut(String) + Send>>,
    ) -> Result<String> {
        let generation_script = format!(
            r#"
import torch
from transformers import AutoTokenizer, AutoModelForCausalLM, TextStreamer
from peft import PeftModel
import sys
import json

# Load model and tokenizer
print("Loading model...", file=sys.stderr)
tokenizer = AutoTokenizer.from_pretrained('{}')
model = AutoModelForCausalLM.from_pretrained(
    '{}',
    torch_dtype=torch.float16,
    device_map="auto" if '{}' == "cuda" else None
)

{}

# Set pad token if not present
if tokenizer.pad_token is None:
    tokenizer.pad_token = tokenizer.eos_token

# Generate
print("Generating...", file=sys.stderr)
inputs = tokenizer('''{}''', return_tensors="pt")
if '{}' == "cuda":
    inputs = {{k: v.cuda() for k, v in inputs.items()}}

with torch.no_grad():
    outputs = model.generate(
        **inputs,
        max_new_tokens={},
        temperature={},
        top_p={},
        top_k={},
        repetition_penalty={},
        pad_token_id=tokenizer.pad_token_id,
        do_sample=True if {} > 0.0 else False,
        {}
    )

# Decode output
generated_text = tokenizer.decode(outputs[0], skip_special_tokens=True)
# Remove input prompt from output
input_text = '''{}'''
if generated_text.startswith(input_text):
    generated_text = generated_text[len(input_text):].strip()

print(generated_text)
"#,
            self.base_model_id,
            self.base_model_id,
            self.device,
            if let Some(peft_path) = &self.peft_path {
                format!(
                    r#"print("Loading PEFT adapter...", file=sys.stderr)
model = PeftModel.from_pretrained(model, '{}')"#,
                    peft_path
                )
            } else {
                "".to_string()
            },
            prompt.replace("'", r"\'"),
            self.device,
            opts.max_tokens,
            opts.temperature,
            opts.top_p,
            opts.top_k,
            opts.repeat_penalty,
            opts.temperature,
            if let Some(seed) = opts.seed {
                format!("use_cache=False,\n        torch.manual_seed({})", seed)
            } else {
                "use_cache=True".to_string()
            },
            prompt.replace("'", r"\'")
        );

        let mut cmd = TokioCommand::new(&self.python_path);
        cmd.args(["-c", &generation_script]);

        let output = cmd.output().await?;

        if !output.status.success() {
            return Err(anyhow!(
                "HuggingFace generation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let result = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = result.lines().collect();
        
        // Find the actual generated text (after loading messages)
        let generated_text = lines
            .iter()
            .rev()
            .find(|line| !line.trim().is_empty())
            .unwrap_or(&"")
            .to_string();

        // Handle streaming callback if provided
        if let Some(mut callback) = on_token {
            for word in generated_text.split_whitespace() {
                callback(format!("{} ", word));
            }
        }

        Ok(generated_text)
    }
}