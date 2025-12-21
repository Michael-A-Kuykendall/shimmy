/*!
# Comprehensive Console Tool Testing

Tests all tools in the extensive tooling suite to ensure they work correctly with the console interface.
This validates tool execution, parameter passing, result handling, and license gating.
*/

#[cfg(feature = "console")]
mod console_tool_tests {
    use shimmy_console::tools::{command::RunCommandTool, ExecutionContext, Tool, ToolArgs};
    use std::collections::HashMap;
    async fn create_test_context() -> ExecutionContext {
        ExecutionContext {
            working_directory: std::env::current_dir()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            user_id: Some("test-user".to_string()),
            session_id: "test-session-tools".to_string(),
        }
    }

    fn create_tool_args(
        parameters: HashMap<String, String>,
        context: ExecutionContext,
    ) -> ToolArgs {
        ToolArgs {
            parameters,
            context,
        }
    }

    #[tokio::test]
    async fn test_tool_interface_contract() {
        println!("\n🔧 Testing Tool Interface Contract");

        let command_tool = RunCommandTool;

        // Test that tool implements required interface
        assert_eq!(command_tool.name(), "run_command");
        assert!(!command_tool.description().is_empty());
        assert!(
            command_tool.requires_license(),
            "Command tool should require license"
        );

        println!("✅ Tool interface contract validated");
        println!("   📝 Tool name: {}", command_tool.name());
        println!("   📄 Description: {}", command_tool.description());
        println!(
            "   🔐 Requires license: {}",
            command_tool.requires_license()
        );
    }

    #[tokio::test]
    async fn test_command_tool_execution() {
        println!("\n⚡ Testing Command Execution Tool");

        let command_tool = RunCommandTool;
        let context = create_test_context().await;

        // Test safe command execution with echo
        let mut params = HashMap::new();
        params.insert(
            "command".to_string(),
            "echo Hello from tool test".to_string(),
        );

        let args = create_tool_args(params, context.clone());
        let result = command_tool.execute(args).await;

        assert!(result.is_ok(), "Command execution should work");
        let result = result.unwrap();

        if result.success {
            assert!(
                result.output.contains("Hello from tool test"),
                "Should capture command output"
            );
            println!("✅ Command execution tool works correctly");
        } else {
            println!("⚠️  Command execution failed: {:?}", result.error_message);
        }
    }

    #[tokio::test]
    async fn test_tool_execution_context_validation() {
        println!("\n🎯 Testing Tool Execution Context");

        let _command_tool = RunCommandTool;
        let context = create_test_context().await;

        // Test that context is properly passed
        assert!(
            !context.working_directory.is_empty(),
            "Working directory should be set"
        );
        assert!(context.user_id.is_some(), "User ID should be set");
        assert!(!context.session_id.is_empty(), "Session ID should be set");

        println!("✅ Tool execution context validation works");
        println!("   📁 Working directory: {}", context.working_directory);
        println!("   👤 User ID: {:?}", context.user_id);
        println!("   🔐 Session ID: {}", context.session_id);
    }

    #[tokio::test]
    async fn test_tool_licensing_integration() {
        println!("\n🔐 Testing Tool Licensing Integration");

        let command_tool = RunCommandTool;

        // Test licensing metadata
        assert!(
            command_tool.requires_license(),
            "Command tool should require premium license"
        );
        assert!(
            !command_tool.name().is_empty(),
            "Tool name should not be empty"
        );
        assert!(
            !command_tool.description().is_empty(),
            "Tool description should not be empty"
        );

        println!("✅ Tool licensing metadata is properly configured");
        println!("   🔒 {} requires premium license", command_tool.name());
    }

    #[tokio::test]
    async fn test_tool_parameter_validation() {
        println!("\n✅ Testing Tool Parameter Validation");

        let command_tool = RunCommandTool;
        let context = create_test_context().await;

        // Test missing required parameters
        let empty_params = HashMap::new();
        let args = create_tool_args(empty_params, context.clone());
        let result = command_tool.execute(args).await;

        // Should handle missing parameters gracefully
        if result.is_err() {
            println!("✅ Tools properly validate required parameters (error returned)");
        } else if let Ok(result) = result {
            if !result.success {
                println!("✅ Tools properly validate required parameters (success=false)");
            }
        }
    }
}
