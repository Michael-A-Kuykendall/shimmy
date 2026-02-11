//! Tests for analysis tools

#[cfg(test)]
mod tests {
    use super::super::analysis::{ProjectAnalysisTool, SyntaxCheckTool};
    use super::super::{Tool, ToolArgs};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_project_analysis_tool_name() {
        let tool = ProjectAnalysisTool;
        assert_eq!(tool.name(), "analyze_project");
    }

    #[tokio::test]
    async fn test_syntax_check_tool_name() {
        let tool = SyntaxCheckTool;
        assert_eq!(tool.name(), "syntax_check");
    }

    #[tokio::test]
    async fn test_project_analysis_requires_license() {
        let tool = ProjectAnalysisTool;
        assert!(tool.requires_license());
    }

    #[tokio::test]
    async fn test_syntax_check_requires_license() {
        let tool = SyntaxCheckTool;
        assert!(tool.requires_license());
    }
}
