use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{self, Write};

use crate::tools::{build_default_registry, ToolArgs};

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:11435";
const MAX_TOOL_ROUNDS: usize = 10;

// ── OpenAI-compatible message types ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".into(),
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }
    fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".into(),
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }
    fn tool_result(call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: "tool".into(),
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: Some(call_id.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String, // JSON string
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: AssistantMessage,
}

#[derive(Debug, Deserialize)]
struct AssistantMessage {
    role: String,
    content: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
}

// ── Main entry point ──────────────────────────────────────────────────────────

pub async fn execute_chat(model: Option<String>, _session: Option<String>) -> Result<()> {
    let base_url = std::env::var("SHIMMY_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.into());
    let model =
        model.unwrap_or_else(|| std::env::var("SHIMMY_MODEL").unwrap_or_else(|_| "default".into()));

    let registry = build_default_registry();
    let client = reqwest::Client::new();

    // System prompt with project context
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".into());
    let system = format!(
        "You are a local AI development assistant. \
        The user's working directory is: {}\n\
        You have access to tools for reading/writing files, running git commands, \
        executing shell commands, and analyzing projects. \
        Use tools when needed to help the user.",
        cwd
    );

    let mut messages: Vec<Message> = vec![Message::system(system)];

    println!("shimmy chat — model: {} @ {}", model, base_url);
    println!("Type your message, or 'exit' to quit.\n");

    loop {
        // Read user input
        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_string();

        if input.is_empty() {
            continue;
        }
        if input == "exit" || input == "quit" {
            break;
        }

        messages.push(Message::user(&input));

        // Agentic loop: send → tool calls → send results → repeat until done
        let mut rounds = 0;
        loop {
            rounds += 1;
            if rounds > MAX_TOOL_ROUNDS {
                eprintln!("[max tool rounds reached]");
                break;
            }

            let tools = registry.to_openai_tools();
            let body = serde_json::json!({
                "model": model,
                "messages": messages,
                "tools": tools,
                "tool_choice": "auto"
            });

            let resp = client
                .post(format!("{}/v1/chat/completions", base_url))
                .json(&body)
                .send()
                .await
                .map_err(|e| anyhow!("Request failed: {}. Is shimmy running?", e))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                return Err(anyhow!("API error {}: {}", status, text));
            }

            let chat_resp: ChatResponse = resp
                .json()
                .await
                .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

            let choice = chat_resp
                .choices
                .into_iter()
                .next()
                .ok_or_else(|| anyhow!("Empty response from model"))?;

            let assistant_msg = choice.message;

            // Push assistant message into history
            messages.push(Message {
                role: assistant_msg.role.clone(),
                content: assistant_msg.content.clone(),
                tool_calls: assistant_msg.tool_calls.clone(),
                tool_call_id: None,
            });

            // If model wants to call tools
            if let Some(tool_calls) = assistant_msg.tool_calls {
                for call in &tool_calls {
                    let tool_name = &call.function.name;
                    let args_value: Value = serde_json::from_str(&call.function.arguments)
                        .unwrap_or(Value::Object(Default::default()));
                    let args_map: HashMap<String, Value> = match args_value {
                        Value::Object(m) => m.into_iter().collect(),
                        _ => HashMap::new(),
                    };
                    let tool_args = ToolArgs::new(args_map);

                    print!("[{}] ", tool_name);
                    io::stdout().flush()?;

                    let result = if let Some(tool) = registry.get(tool_name) {
                        match tool.execute(tool_args).await {
                            Ok(r) => r.output,
                            Err(e) => format!("Tool error: {}", e),
                        }
                    } else {
                        format!("Unknown tool: {}", tool_name)
                    };

                    println!("✓");
                    messages.push(Message::tool_result(&call.id, result));
                }
                // Loop to send tool results back
                continue;
            }

            // No tool calls — print the response and break inner loop
            if let Some(content) = assistant_msg.content {
                println!("\n{}\n", content);
            }
            break;
        }
    }

    println!("Goodbye.");
    Ok(())
}
