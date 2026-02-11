//! System test binary for shimmy console

use shimmy_console::{Config, ToolRegistry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Shimmy Console System Test");
    println!("==========================\n");

    // Test configuration
    println!("Testing configuration...");
    let config = Config::from_env();
    println!("  Backend URL: {}", config.backend_url);
    println!("  Discovery port: {}", config.discovery_port);
    println!("  Max context tokens: {}", config.max_context_tokens);
    println!("  ✓ Configuration loaded\n");

    // Test tool registry
    println!("Testing tool registry...");
    let registry = ToolRegistry::with_defaults();
    let tool_names: Vec<_> = registry.names().collect();
    println!("  Registered tools: {:?}", tool_names);
    println!("  ✓ Tool registry initialized\n");

    // Test individual tools
    println!("Testing tools...");
    
    if let Some(tool) = registry.get("system_info") {
        use shimmy_console::ToolArgs;
        let args = ToolArgs::new();
        match tool.execute(args).await {
            Ok(result) => {
                println!("  ✓ system_info tool works");
                if result.output.len() > 100 {
                    println!("    Output: {}...", &result.output[..100]);
                }
            }
            Err(e) => {
                println!("  ✗ system_info tool failed: {}", e);
            }
        }
    }

    if let Some(tool) = registry.get("list_files") {
        use shimmy_console::ToolArgs;
        let mut args = ToolArgs::new();
        args.args.insert("path".to_string(), serde_json::json!("."));
        match tool.execute(args).await {
            Ok(result) => {
                let lines: Vec<_> = result.output.lines().take(5).collect();
                println!("  ✓ list_files tool works");
                println!("    First 5 entries: {:?}", lines);
            }
            Err(e) => {
                println!("  ✗ list_files tool failed: {}", e);
            }
        }
    }

    println!("\n==========================");
    println!("System test complete!");

    Ok(())
}
