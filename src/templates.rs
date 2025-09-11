use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateFamily { ChatML, Llama3, OpenChat }

impl TemplateFamily {
    pub fn render(&self, system: Option<&str>, messages: &[(String, String)], input: Option<&str>) -> String {
        match self {
            TemplateFamily::ChatML => {
                let mut s = String::new();
                if let Some(sys) = system { s.push_str(&format!("<|im_start|>system\n{}<|im_end|>\n", sys)); }
                for (role, content) in messages { s.push_str(&format!("<|im_start|>{}\n{}<|im_end|>\n", role, content)); }
                if let Some(inp) = input { s.push_str(&format!("<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n", inp)); }
                s
            }
            TemplateFamily::Llama3 => {
                let mut s = String::new();
                if let Some(sys) = system { s.push_str(&format!("<|begin_of_text|><|start_header_id|>system<|end_header_id|>\n{}<|eot_id|>", sys)); }
                for (role, content) in messages { s.push_str(&format!("<|start_header_id|>{}<|end_header_id|>\n{}<|eot_id|>", role, content)); }
                if let Some(inp) = input { s.push_str(&format!("<|start_header_id|>user<|end_header_id|>\n{}<|eot_id|><|start_header_id|>assistant<|end_header_id|>\n", inp)); }
                s
            }
            TemplateFamily::OpenChat => {
                let mut s = String::new();
                for (role, content) in messages { s.push_str(&format!("{}: {}\n", role, content)); }
                if let Some(inp) = input { s.push_str(&format!("user: {}\nassistant: ", inp)); } else { s.push_str("assistant: "); }
                s
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chatml_render() {
        let template = TemplateFamily::ChatML;
        let messages = vec![("user".to_string(), "Hello".to_string())];
        let result = template.render(None, &messages, None);
        assert!(result.contains("<|im_start|>user"));
        assert!(result.contains("Hello"));
        assert!(result.contains("<|im_end|>"));
    }
    
    #[test]
    fn test_llama3_render() {
        let template = TemplateFamily::Llama3;
        let messages = vec![("user".to_string(), "Test".to_string())];
        let result = template.render(None, &messages, None);
        assert!(result.contains("<|start_header_id|>user<|end_header_id|>"));
        assert!(result.contains("Test"));
        assert!(result.contains("<|eot_id|>"));
    }
    
    #[test]
    fn test_openchat_render() {
        let template = TemplateFamily::OpenChat;
        let messages = vec![("user".to_string(), "Hi".to_string())];
        let result = template.render(None, &messages, None);
        assert!(result.contains("user: Hi"));
        assert!(result.contains("assistant: "));
    }
}
