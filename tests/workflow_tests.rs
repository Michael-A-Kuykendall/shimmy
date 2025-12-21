// PUNCH-generated tests for workflow module
use shimmy::tools::ToolRegistry;
use shimmy::workflow::{WorkflowEngine, WorkflowStep, WorkflowStepType};
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    // Rule: rust_result_err - Functions returning Result need Err case tests
    #[test]
    fn execute_workflow_error_case() {
        // Test error case handling with invalid workflow
        let engine = WorkflowEngine::new(ToolRegistry::new());
        let request = shimmy::workflow::WorkflowRequest {
            workflow: shimmy::workflow::Workflow {
                id: "test".to_string(),
                name: "test".to_string(),
                description: "test".to_string(),
                steps: vec![], // Empty workflow should cause error
                inputs: HashMap::new(),
                outputs: vec!["nonexistent".to_string()], // Reference non-existent step
            },
            context: HashMap::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(engine.execute_workflow(request));
        // Empty workflow might succeed but requesting output from non-existent step should fail
        if let Ok(workflow_result) = result {
            assert!(
                !workflow_result.success,
                "Workflow should fail with non-existent output step"
            );
        }
    }

    // Rule: rust_result_err - Functions returning Result need Err case tests
    #[test]
    fn calculate_execution_order_error_case() {
        let engine = WorkflowEngine::new(ToolRegistry::new());
        // Test circular dependencies - same as in the main module tests
        let steps = vec![
            WorkflowStep {
                id: "step1".to_string(),
                step_type: WorkflowStepType::DataTransform {
                    operation: "extract".to_string(),
                    expression: "test".to_string(),
                },
                depends_on: vec!["step2".to_string()],
                parameters: serde_json::Value::Null,
            },
            WorkflowStep {
                id: "step2".to_string(),
                step_type: WorkflowStepType::DataTransform {
                    operation: "extract".to_string(),
                    expression: "test".to_string(),
                },
                depends_on: vec!["step1".to_string()],
                parameters: serde_json::Value::Null,
            },
        ];
        // Execute via public API: a circular dependency should cause execute_workflow to fail
        let workflow = shimmy::workflow::Workflow {
            id: "wf_circ".to_string(),
            name: "circular".to_string(),
            description: "circular deps".to_string(),
            steps: steps.clone(),
            inputs: HashMap::new(),
            outputs: vec!["step1".to_string()],
        };

        let request = shimmy::workflow::WorkflowRequest {
            workflow,
            context: HashMap::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(engine.execute_workflow(request));
        assert!(result.is_err(), "Execution should fail due to circular dependency");
    }

    // Rule: rust_result_err - Functions returning Result need Err case tests
    #[test]
    fn substitute_variables_error_case() {
        let engine = WorkflowEngine::new(ToolRegistry::new());
        // Test undefined variables via execute_workflow public API
        let step = WorkflowStep {
            id: "step1".to_string(),
            step_type: WorkflowStepType::LLMGeneration {
                prompt: "Hello {{undefined_var}}".to_string(),
                model: None,
                max_tokens: None,
                temperature: None,
            },
            depends_on: vec![],
            parameters: serde_json::Value::Null,
        };

        let workflow = shimmy::workflow::Workflow {
            id: "wf2".to_string(),
            name: "substitute".to_string(),
            description: "substitute vars".to_string(),
            steps: vec![step.clone()],
            inputs: HashMap::new(),
            outputs: vec![step.id.clone()],
        };

        let request = shimmy::workflow::WorkflowRequest { workflow, context: HashMap::new() };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(engine.execute_workflow(request)).unwrap();

        let output_text = result
            .outputs
            .get("step1")
            .and_then(|v| v.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("");

        assert!(output_text.contains("{{undefined_var}}"));
    }

    // Rule: rust_empty_str - Functions accepting &str need empty string tests
    #[test]
    fn substitute_variables_empty_template() {
        let engine = WorkflowEngine::new(ToolRegistry::new());
        // Empty template should not cause the workflow to error
        let step = WorkflowStep {
            id: "step_empty".to_string(),
            step_type: WorkflowStepType::LLMGeneration {
                prompt: "".to_string(),
                model: None,
                max_tokens: None,
                temperature: None,
            },
            depends_on: vec![],
            parameters: serde_json::Value::Null,
        };

        let workflow = shimmy::workflow::Workflow {
            id: "wf_empty".to_string(),
            name: "empty".to_string(),
            description: "empty prompt".to_string(),
            steps: vec![step.clone()],
            inputs: HashMap::new(),
            outputs: vec![step.id.clone()],
        };

        let request = shimmy::workflow::WorkflowRequest { workflow, context: HashMap::new() };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(engine.execute_workflow(request));
        assert!(result.is_ok(), "Empty prompt should not cause error");
    }

    #[test]
    fn substitute_variables_no_variables() {
        let engine = WorkflowEngine::new(ToolRegistry::new());
        // Template without variables should work through the public API
        let step = WorkflowStep {
            id: "step_nv".to_string(),
            step_type: WorkflowStepType::LLMGeneration {
                prompt: "Hello World".to_string(),
                model: None,
                max_tokens: None,
                temperature: None,
            },
            depends_on: vec![],
            parameters: serde_json::Value::Null,
        };

        let workflow = shimmy::workflow::Workflow {
            id: "wf_nv".to_string(),
            name: "no_vars".to_string(),
            description: "no vars".to_string(),
            steps: vec![step.clone()],
            inputs: HashMap::new(),
            outputs: vec![step.id.clone()],
        };

        let request = shimmy::workflow::WorkflowRequest { workflow, context: HashMap::new() };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(engine.execute_workflow(request));
        assert!(result.is_ok(), "Template without vars should succeed");
    }
}
