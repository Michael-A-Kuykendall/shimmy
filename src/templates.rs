use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateFamily { ChatML, Llama3, OpenChat }

impl TemplateFamily {
    pub fn render(&self, system: Option<&str>, messages: &[(String, String)], input: Option<&str>) -> String {
        match self {
            TemplateFamily::ChatML => {
                let mut s = String::new();
                if let Some(sys) = system { s.push_str(&format!(\"<|im_start|>system\\
{}<|im_end|>\\
\", sys)); }
                for (role, content) in messages { s.push_str(&format!(\"<|im_start|>{}\\
{}<|im_end|>\\
\", role, content)); }
                if let Some(inp) = input { s.push_str(&format!(\"<|im_start|>user\\
{}<|im_end|>\\
<|im_start|>assistant\\
\", inp)); }
                s
            }
            TemplateFamily::Llama3 => {
                let mut s = String::new();
                if let Some(sys) = system { s.push_str(&format!(\"<|begin_of_text|><|start_header_id|>system<|end_header_id|>\\
{}<|eot_id|>\", sys)); }
                for (role, content) in messages { s.push_str(&format!(\"<|start_header_id|>{}<|end_header_id|>\\
{}<|eot_id|>\", role, content)); }
                if let Some(inp) = input { s.push_str(&format!(\"<|start_header_id|>user<|end_header_id|>\\
{}<|eot_id|><|start_header_id|>assistant<|end_header_id|>\\
\", inp)); }
                s
            }
            TemplateFamily::OpenChat => {
                let mut s = String::new();
                for (role, content) in messages { s.push_str(&format!(\"{}: {}\\
\", role, content)); }
                if let Some(inp) = input { s.push_str(&format!(\"user: {}\\
assistant: \", inp)); } else { s.push_str(\"assistant: \"); }
                s
            }
        }
    }
}

