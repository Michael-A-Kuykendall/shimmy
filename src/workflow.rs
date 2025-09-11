use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use crate::tools::{ToolCall, ToolRegistry};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub step_type: WorkflowStepType,
    pub depends_on: Vec<String>,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WorkflowStepType {
    #[serde(rename = "llm")]
    LLMGeneration {
        prompt: String,
        model: Option<String>,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    },
    #[serde(rename = "tool")]
    ToolCall {
        tool_name: String,
        arguments: serde_json::Value,
    },
    #[serde(rename = "data_transform")]
    DataTransform {
        operation: String, // "filter", "map", "reduce", "extract"
        expression: String, // JSONPath or simple operations
    },
    #[serde(rename = "conditional")]
    Conditional {
        condition: String,
        if_true: Box<WorkflowStep>,
        if_false: Option<Box<WorkflowStep>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub inputs: HashMap<String, serde_json::Value>,
    pub outputs: Vec<String>, // Step IDs whose results should be included in final output
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRequest {
    pub workflow: Workflow,
    pub context: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub workflow_id: String,
    pub success: bool,
    pub step_results: HashMap<String, StepResult>,
    pub outputs: HashMap<String, serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: String,
    pub success: bool,
    pub result: serde_json::Value,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

pub struct WorkflowEngine {
    tool_registry: ToolRegistry,
}

impl WorkflowEngine {
    pub fn new(tool_registry: ToolRegistry) -> Self {
        Self { tool_registry }
    }

    pub async fn execute_workflow(&self, request: WorkflowRequest) -> Result<WorkflowResult> {
        let start_time = std::time::Instant::now();
        let mut step_results = HashMap::new();
        let mut context = request.context;
        
        // Add workflow inputs to context
        for (key, value) in request.workflow.inputs {
            context.insert(key, value);
        }

        // Execute steps in dependency order
        let execution_order = self.calculate_execution_order(&request.workflow.steps)?;
        
        for step_id in execution_order {
            let step = request.workflow.steps.iter()
                .find(|s| s.id == step_id)
                .ok_or_else(|| anyhow::anyhow!("Step {} not found", step_id))?;

            let step_start = std::time::Instant::now();
            let step_result = match self.execute_step(step, &context, &step_results).await {
                Ok(result) => {
                    // Add step result to context for subsequent steps
                    context.insert(format!("step_{}", step.id), result.clone());
                    StepResult {
                        step_id: step.id.clone(),
                        success: true,
                        result,
                        error: None,
                        execution_time_ms: step_start.elapsed().as_millis() as u64,
                    }
                }
                Err(e) => StepResult {
                    step_id: step.id.clone(),
                    success: false,
                    result: serde_json::Value::Null,
                    error: Some(e.to_string()),
                    execution_time_ms: step_start.elapsed().as_millis() as u64,
                }
            };

            step_results.insert(step.id.clone(), step_result);
        }

        // Collect outputs
        let mut outputs = HashMap::new();
        for output_step_id in &request.workflow.outputs {
            if let Some(step_result) = step_results.get(output_step_id) {
                outputs.insert(output_step_id.clone(), step_result.result.clone());
            }
        }

        // Check if workflow succeeded (all steps succeeded)
        let success = step_results.values().all(|result| result.success);

        Ok(WorkflowResult {
            workflow_id: request.workflow.id,
            success,
            step_results,
            outputs,
            error: if success { None } else { Some("One or more steps failed".to_string()) },
            execution_time_ms: start_time.elapsed().as_millis() as u64,
        })
    }

    fn execute_step<'a>(
        &'a self,
        step: &'a WorkflowStep,
        context: &'a HashMap<String, serde_json::Value>,
        step_results: &'a HashMap<String, StepResult>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<serde_json::Value>> + Send + 'a>> {
        Box::pin(async move {
            match &step.step_type {
                WorkflowStepType::LLMGeneration { prompt, model, max_tokens, temperature } => {
                    // Substitute context variables in prompt
                    let resolved_prompt = self.substitute_variables(prompt, context)?;
                    
                    // Use the existing LLM generation logic
                    // This would integrate with the actual generation engine
                    let result = self.call_llm(
                        &resolved_prompt,
                        model.as_deref().unwrap_or("default"),
                        max_tokens.unwrap_or(512),
                        temperature.unwrap_or(0.7),
                    ).await?;
                    
                    Ok(serde_json::json!({
                        "text": result,
                        "type": "llm_generation"
                    }))
                }
                
                WorkflowStepType::ToolCall { tool_name, arguments } => {
                    // Substitute context variables in arguments
                    let resolved_args = self.substitute_variables_in_json(arguments, context)?;
                    
                    let tool_call = ToolCall {
                        name: tool_name.clone(),
                        arguments: resolved_args,
                    };
                    
                    let tool_result = self.tool_registry.execute_tool(&tool_call)?;
                    
                    if tool_result.success {
                        Ok(tool_result.result)
                    } else {
                        Err(anyhow::anyhow!("Tool execution failed: {:?}", tool_result.error))
                    }
                }
                
                WorkflowStepType::DataTransform { operation, expression } => {
                    self.execute_data_transform(operation, expression, context, step_results)
                }
                
                WorkflowStepType::Conditional { condition, if_true, if_false } => {
                    let condition_result = self.evaluate_condition(condition, context)?;
                    
                    if condition_result {
                        self.execute_step(if_true, context, step_results).await
                    } else if let Some(false_step) = if_false {
                        self.execute_step(false_step, context, step_results).await
                    } else {
                        Ok(serde_json::json!({ "skipped": true }))
                    }
                }
            }
        })
    }

    pub fn calculate_execution_order(&self, steps: &[WorkflowStep]) -> Result<Vec<String>> {
        let mut order = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut temp_visited = std::collections::HashSet::new();

        for step in steps {
            if !visited.contains(&step.id) {
                self.visit_step(&step.id, steps, &mut visited, &mut temp_visited, &mut order)?;
            }
        }

        Ok(order)
    }

    fn visit_step(
        &self,
        step_id: &str,
        steps: &[WorkflowStep],
        visited: &mut std::collections::HashSet<String>,
        temp_visited: &mut std::collections::HashSet<String>,
        order: &mut Vec<String>,
    ) -> Result<()> {
        if temp_visited.contains(step_id) {
            return Err(anyhow::anyhow!("Circular dependency detected involving step {}", step_id));
        }

        if visited.contains(step_id) {
            return Ok(());
        }

        temp_visited.insert(step_id.to_string());

        let step = steps.iter()
            .find(|s| s.id == step_id)
            .ok_or_else(|| anyhow::anyhow!("Step {} not found", step_id))?;

        for dep in &step.depends_on {
            self.visit_step(dep, steps, visited, temp_visited, order)?;
        }

        temp_visited.remove(step_id);
        visited.insert(step_id.to_string());
        order.push(step_id.to_string());

        Ok(())
    }

    pub fn substitute_variables(&self, text: &str, context: &HashMap<String, serde_json::Value>) -> Result<String> {
        let mut result = text.to_string();
        
        // Simple variable substitution: {{variable_name}}
        for (key, value) in context {
            let placeholder = format!("{{{{{}}}}}", key);
            let replacement = match value {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            result = result.replace(&placeholder, &replacement);
        }
        
        Ok(result)
    }

    fn substitute_variables_in_json(
        &self,
        json: &serde_json::Value,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value> {
        match json {
            serde_json::Value::String(s) => {
                Ok(serde_json::Value::String(self.substitute_variables(s, context)?))
            }
            serde_json::Value::Object(obj) => {
                let mut new_obj = serde_json::Map::new();
                for (key, value) in obj {
                    new_obj.insert(key.clone(), self.substitute_variables_in_json(value, context)?);
                }
                Ok(serde_json::Value::Object(new_obj))
            }
            serde_json::Value::Array(arr) => {
                let new_arr: Result<Vec<_>> = arr.iter()
                    .map(|item| self.substitute_variables_in_json(item, context))
                    .collect();
                Ok(serde_json::Value::Array(new_arr?))
            }
            other => Ok(other.clone()),
        }
    }

    fn execute_data_transform(
        &self,
        operation: &str,
        expression: &str,
        context: &HashMap<String, serde_json::Value>,
        step_results: &HashMap<String, StepResult>,
    ) -> Result<serde_json::Value> {
        match operation {
            "extract" => {
                // Simple JSONPath-like extraction
                if let Some(value) = context.get(expression) {
                    Ok(value.clone())
                } else if expression.starts_with("step_") {
                    // Extract from step result
                    let step_id = expression.strip_prefix("step_").unwrap();
                    if let Some(step_result) = step_results.get(step_id) {
                        Ok(step_result.result.clone())
                    } else {
                        Err(anyhow::anyhow!("Step {} not found", step_id))
                    }
                } else {
                    Err(anyhow::anyhow!("Variable {} not found", expression))
                }
            }
            "filter" => {
                // Simple filtering - would be expanded with a proper expression evaluator
                Ok(serde_json::json!({ "filtered": true, "expression": expression }))
            }
            _ => Err(anyhow::anyhow!("Unsupported data transform operation: {}", operation)),
        }
    }

    fn evaluate_condition(
        &self,
        condition: &str,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<bool> {
        // Simple condition evaluation - would be expanded with a proper expression evaluator
        if condition.contains("==") {
            let parts: Vec<&str> = condition.split("==").collect();
            if parts.len() == 2 {
                let left = parts[0].trim();
                let right = parts[1].trim();
                
                let left_value = context.get(left);
                let right_str = right.trim_matches('"');
                
                match left_value {
                    Some(serde_json::Value::String(s)) => Ok(s == right_str),
                    Some(serde_json::Value::Bool(b)) => Ok(b.to_string() == right_str),
                    _ => Ok(false),
                }
            } else {
                Ok(false)
            }
        } else {
            // Default to true for unsupported conditions
            Ok(true)
        }
    }

    async fn call_llm(
        &self,
        prompt: &str,
        model: &str,
        max_tokens: u32,
        temperature: f32,
    ) -> Result<String> {
        // This would integrate with the actual LLM generation system
        // For now, return a placeholder
        Ok(format!("LLM response to: {} (model: {}, max_tokens: {}, temp: {})", 
                  prompt, model, max_tokens, temperature))
    }
}

// Tests for workflow functionality
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_variables_empty_template() {
        let engine = WorkflowEngine::new(ToolRegistry::new());
        let variables = HashMap::new();
        let result = engine.substitute_variables("", &variables).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_substitute_variables_no_variables() {
        let engine = WorkflowEngine::new(ToolRegistry::new());
        let variables = HashMap::new();
        let template = "Hello World";
        let result = engine.substitute_variables(template, &variables).unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_calculate_execution_order_empty() {
        let engine = WorkflowEngine::new(ToolRegistry::new());
        let steps = vec![];
        let result = engine.calculate_execution_order(&steps).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_calculate_execution_order_circular_dependency() {
        let engine = WorkflowEngine::new(ToolRegistry::new());
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
        let result = engine.calculate_execution_order(&steps);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Circular dependency"));
    }
}