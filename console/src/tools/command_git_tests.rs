//! Tests for git-related command functionality

#[cfg(test)]
mod tests {
    use super::super::command::ShellCommandTool;
    use super::super::Tool;

    #[test]
    fn test_shell_command_tool_name() {
        let tool = ShellCommandTool;
        assert_eq!(tool.name(), "shell_command");
    }

    #[test]
    fn test_shell_command_description() {
        let tool = ShellCommandTool;
        assert!(tool.description().contains("shell"));
    }
}
