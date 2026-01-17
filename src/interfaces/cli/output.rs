//! Output formatting for CLI interface

use crate::core::{ExecutionStatus, ToolInfo};
use crate::workflow::{NodeExecutionState, WorkflowExecution};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub use crate::interfaces::cli::commands::OutputFormat;

/// Trait for formatting CLI output
pub trait OutputFormatter {
    fn format_workflow_status(&self, execution: &WorkflowExecution) -> String;
    fn format_workflow_list(&self, executions: &[WorkflowExecution]) -> String;
    fn format_tool_list(&self, tools: &[ToolInfo]) -> String;
    fn format_tool_info(&self, tool: &ToolInfo) -> String;
    fn format_execution_result(&self, result: &ExecutionResult) -> String;
    fn format_error(&self, error: &str) -> String;
    fn format_success(&self, message: &str) -> String;
    fn format_section(&self, title: &str) -> String;
}

/// Execution result for formatting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub workflow_id: String,
    pub workflow_name: String,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<Value>,
    pub error: Option<String>,
}

/// Table formatter (default)
pub struct TableFormatter;

impl OutputFormatter for TableFormatter {
    fn format_workflow_status(&self, execution: &WorkflowExecution) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!(
            "Workflow: {} ({})\n",
            execution.workflow_name, execution.id
        ));
        output.push_str(&format!("Status: {:?}\n", execution.status));
        output.push_str(&format!(
            "Started: {}\n",
            execution.started_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        if let Some(completed_at) = execution.completed_at {
            output.push_str(&format!(
                "Completed: {}\n",
                completed_at.format("%Y-%m-%d %H:%M:%S UTC")
            ));
            let duration = completed_at.signed_duration_since(execution.started_at);
            output.push_str(&format!("Duration: {}s\n", duration.num_seconds()));
        }

        if let Some(current_node) = &execution.current_node {
            output.push_str(&format!("Current Node: {}\n", current_node));
        }

        // Node status table
        if !execution.node_states.is_empty() {
            output.push_str("\nNode Status:\n");
            output.push_str("┌─────────────────────────────────┬─────────────┬─────────────────────┬─────────────────────┐\n");
            output.push_str("│ Node ID                         │ Status      │ Started At          │ Completed At        │\n");
            output.push_str("├─────────────────────────────────┼─────────────┼─────────────────────┼─────────────────────┤\n");

            for (node_id, node_state) in &execution.node_states {
                let started = node_state
                    .started_at
                    .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "-".to_string());
                let completed = node_state
                    .completed_at
                    .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "-".to_string());

                output.push_str(&format!(
                    "│ {:<31} │ {:<11} │ {:<19} │ {:<19} │\n",
                    truncate_string(node_id, 31),
                    format!("{:?}", node_state.status),
                    started,
                    completed
                ));
            }

            output.push_str("└─────────────────────────────────┴─────────────┴─────────────────────┴─────────────────────┘\n");
        }

        output
    }

    fn format_workflow_list(&self, executions: &[WorkflowExecution]) -> String {
        if executions.is_empty() {
            return "No workflows found.\n".to_string();
        }

        let mut output = String::new();
        output.push_str("┌─────────────────────────────────────┬─────────────┬─────────────────────┬──────────┐\n");
        output.push_str("│ Workflow ID                         │ Status      │ Started At          │ Progress │\n");
        output.push_str("├─────────────────────────────────────┼─────────────┼─────────────────────┼──────────┤\n");

        for execution in executions {
            let progress = calculate_progress(&execution.node_states);
            let started = execution.started_at.format("%Y-%m-%d %H:%M:%S").to_string();

            output.push_str(&format!(
                "│ {:<35} │ {:<11} │ {:<19} │ {:<8} │\n",
                truncate_string(&execution.id.to_string(), 35),
                format!("{:?}", execution.status),
                started,
                progress
            ));
        }

        output.push_str("└─────────────────────────────────────┴─────────────┴─────────────────────┴──────────┘\n");
        output
    }

    fn format_tool_list(&self, tools: &[ToolInfo]) -> String {
        if tools.is_empty() {
            return "No tools found.\n".to_string();
        }

        let mut output = String::new();
        output.push_str("┌─────────────────────────────────┬─────────┬─────────────────────────────────────────────────┐\n");
        output.push_str("│ Tool Name                       │ Version │ Description                                     │\n");
        output.push_str("├─────────────────────────────────┼─────────┼─────────────────────────────────────────────────┤\n");

        for tool in tools {
            output.push_str(&format!(
                "│ {:<31} │ {:<7} │ {:<47} │\n",
                truncate_string(&tool.name, 31),
                truncate_string(&tool.version, 7),
                truncate_string(&tool.description, 47)
            ));
        }

        output.push_str("└─────────────────────────────────┴─────────┴─────────────────────────────────────────────────┘\n");
        output
    }

    fn format_tool_info(&self, tool: &ToolInfo) -> String {
        let mut output = String::new();
        output.push_str(&format!("Tool: {} v{}\n", tool.name, tool.version));
        output.push_str(&format!("Description: {}\n", tool.description));

        if let Some(category) = &tool.category {
            output.push_str(&format!("Category: {}\n", category));
        }

        if !tool.tags.is_empty() {
            output.push_str(&format!("Tags: {}\n", tool.tags.join(", ")));
        }

        output
    }

    fn format_execution_result(&self, result: &ExecutionResult) -> String {
        let mut output = String::new();
        output.push_str(&format!(
            "Workflow: {} ({})\n",
            result.workflow_name, result.workflow_id
        ));
        output.push_str(&format!("Status: {:?}\n", result.status));
        output.push_str(&format!(
            "Started: {}\n",
            result.started_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        if let Some(completed_at) = result.completed_at {
            output.push_str(&format!(
                "Completed: {}\n",
                completed_at.format("%Y-%m-%d %H:%M:%S UTC")
            ));
            let duration = completed_at.signed_duration_since(result.started_at);
            output.push_str(&format!("Duration: {}s\n", duration.num_seconds()));
        }

        if let Some(error) = &result.error {
            output.push_str(&format!("Error: {}\n", error));
        }

        if let Some(result_value) = &result.result {
            output.push_str(&format!(
                "Result: {}\n",
                serde_json::to_string_pretty(result_value)
                    .unwrap_or_else(|_| "Invalid JSON".to_string())
            ));
        }

        output
    }

    fn format_error(&self, error: &str) -> String {
        format!("Error: {}\n", error)
    }

    fn format_success(&self, message: &str) -> String {
        format!("Success: {}\n", message)
    }

    fn format_section(&self, title: &str) -> String {
        let line = "=".repeat(title.len() + 4);
        format!("\n{}\n  {}  \n{}\n", line, title, line)
    }
}

/// JSON formatter
pub struct JsonFormatter;

impl OutputFormatter for JsonFormatter {
    fn format_workflow_status(&self, execution: &WorkflowExecution) -> String {
        serde_json::to_string_pretty(execution).unwrap_or_else(|_| "{}".to_string())
    }

    fn format_workflow_list(&self, executions: &[WorkflowExecution]) -> String {
        let json_obj = serde_json::json!({
            "workflows": executions
        });
        serde_json::to_string_pretty(&json_obj).unwrap_or_else(|_| "{}".to_string())
    }

    fn format_tool_list(&self, tools: &[ToolInfo]) -> String {
        let json_obj = serde_json::json!({
            "tools": tools
        });
        serde_json::to_string_pretty(&json_obj).unwrap_or_else(|_| "{}".to_string())
    }

    fn format_tool_info(&self, tool: &ToolInfo) -> String {
        serde_json::to_string_pretty(tool).unwrap_or_else(|_| "{}".to_string())
    }

    fn format_execution_result(&self, result: &ExecutionResult) -> String {
        serde_json::to_string_pretty(result).unwrap_or_else(|_| "{}".to_string())
    }

    fn format_error(&self, error: &str) -> String {
        let json_obj = serde_json::json!({
            "error": error
        });
        serde_json::to_string_pretty(&json_obj).unwrap_or_else(|_| "{}".to_string())
    }

    fn format_success(&self, message: &str) -> String {
        let json_obj = serde_json::json!({
            "success": message
        });
        serde_json::to_string_pretty(&json_obj).unwrap_or_else(|_| "{}".to_string())
    }

    fn format_section(&self, _title: &str) -> String {
        String::new()
    }
}

/// YAML formatter
pub struct YamlFormatter;

impl OutputFormatter for YamlFormatter {
    fn format_workflow_status(&self, execution: &WorkflowExecution) -> String {
        serde_yaml::to_string(execution).unwrap_or_else(|_| "---\n".to_string())
    }

    fn format_workflow_list(&self, executions: &[WorkflowExecution]) -> String {
        let yaml_obj = serde_json::json!({
            "workflows": executions
        });
        serde_yaml::to_string(&yaml_obj).unwrap_or_else(|_| "---\n".to_string())
    }

    fn format_tool_list(&self, tools: &[ToolInfo]) -> String {
        let yaml_obj = serde_json::json!({
            "tools": tools
        });
        serde_yaml::to_string(&yaml_obj).unwrap_or_else(|_| "---\n".to_string())
    }

    fn format_tool_info(&self, tool: &ToolInfo) -> String {
        serde_yaml::to_string(tool).unwrap_or_else(|_| "---\n".to_string())
    }

    fn format_execution_result(&self, result: &ExecutionResult) -> String {
        serde_yaml::to_string(result).unwrap_or_else(|_| "---\n".to_string())
    }

    fn format_error(&self, error: &str) -> String {
        let yaml_obj = serde_json::json!({
            "error": error
        });
        serde_yaml::to_string(&yaml_obj).unwrap_or_else(|_| "---\n".to_string())
    }

    fn format_success(&self, message: &str) -> String {
        let yaml_obj = serde_json::json!({
            "success": message
        });
        serde_yaml::to_string(&yaml_obj).unwrap_or_else(|_| "---\n".to_string())
    }

    fn format_section(&self, _title: &str) -> String {
        String::new()
    }
}

/// Text formatter (simple plain text)
pub struct TextFormatter;

impl OutputFormatter for TextFormatter {
    fn format_workflow_status(&self, execution: &WorkflowExecution) -> String {
        format!(
            "{} {} {:?}",
            execution.id, execution.workflow_name, execution.status
        )
    }

    fn format_workflow_list(&self, executions: &[WorkflowExecution]) -> String {
        executions
            .iter()
            .map(|e| format!("{} {} {:?}", e.id, e.workflow_name, e.status))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_tool_list(&self, tools: &[ToolInfo]) -> String {
        tools
            .iter()
            .map(|t| format!("{} {} {}", t.name, t.version, t.description))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_tool_info(&self, tool: &ToolInfo) -> String {
        format!("{} {} {}", tool.name, tool.version, tool.description)
    }

    fn format_execution_result(&self, result: &ExecutionResult) -> String {
        format!(
            "{} {} {:?}",
            result.workflow_id, result.workflow_name, result.status
        )
    }

    fn format_error(&self, error: &str) -> String {
        format!("ERROR: {}", error)
    }

    fn format_success(&self, message: &str) -> String {
        format!("SUCCESS: {}", message)
    }

    fn format_section(&self, title: &str) -> String {
        format!("\n=== {} ===\n", title)
    }
}

/// Create formatter based on output format
pub fn create_formatter(format: &OutputFormat) -> Box<dyn OutputFormatter> {
    match format {
        OutputFormat::Table => Box::new(TableFormatter),
        OutputFormat::Json => Box::new(JsonFormatter),
        OutputFormat::Yaml => Box::new(YamlFormatter),
        OutputFormat::Text => Box::new(TextFormatter),
    }
}

// Helper functions

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

fn calculate_progress(node_states: &HashMap<String, NodeExecutionState>) -> String {
    if node_states.is_empty() {
        return "0/0".to_string();
    }

    let completed = node_states
        .values()
        .filter(|state| state.status == ExecutionStatus::Completed)
        .count();
    let total = node_states.len();

    format!("{}/{}", completed, total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ExecutionStatus;
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_execution() -> WorkflowExecution {
        let mut node_states = HashMap::new();
        node_states.insert(
            "node1".to_string(),
            NodeExecutionState {
                status: ExecutionStatus::Completed,
                started_at: Some(Utc::now()),
                completed_at: Some(Utc::now()),
                result: None,
                error: None,
                retry_count: 0,
            },
        );
        node_states.insert(
            "node2".to_string(),
            NodeExecutionState {
                status: ExecutionStatus::Running,
                started_at: Some(Utc::now()),
                completed_at: None,
                result: None,
                error: None,
                retry_count: 0,
            },
        );

        WorkflowExecution {
            id: uuid::Uuid::new_v4(),
            workflow_name: "test-workflow".to_string(),
            status: ExecutionStatus::Running,
            started_at: Utc::now(),
            completed_at: None,
            current_node: Some("node2".to_string()),
            node_states,
            global_context: serde_json::Value::Null,
        }
    }

    fn create_test_tool() -> ToolInfo {
        ToolInfo {
            name: "test-tool".to_string(),
            version: "1.0.0".to_string(),
            description: "A test tool".to_string(),
            category: Some("testing".to_string()),
            tags: vec!["test".to_string(), "example".to_string()],
            parameters_schema: serde_json::json!({}),
            return_schema: serde_json::json!({}),
            plugin_name: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            dependencies: Vec::new(),
            version_requirements: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_table_formatter_workflow_status() {
        let formatter = TableFormatter;
        let execution = create_test_execution();
        let output = formatter.format_workflow_status(&execution);

        assert!(output.contains("test-workflow"));
        assert!(output.contains("Running"));
        assert!(output.contains("Node Status:"));
    }

    #[test]
    fn test_json_formatter_workflow_status() {
        let formatter = JsonFormatter;
        let execution = create_test_execution();
        let output = formatter.format_workflow_status(&execution);

        // Should be valid JSON
        let _: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(output.contains("test-workflow"));
    }

    #[test]
    fn test_yaml_formatter_workflow_status() {
        let formatter = YamlFormatter;
        let execution = create_test_execution();
        let output = formatter.format_workflow_status(&execution);

        // Should be valid YAML
        let _: serde_json::Value = serde_yaml::from_str(&output).unwrap();
        assert!(output.contains("test-workflow"));
    }

    #[test]
    fn test_table_formatter_tool_list() {
        let formatter = TableFormatter;
        let tools = vec![create_test_tool()];
        let output = formatter.format_tool_list(&tools);

        assert!(output.contains("test-tool"));
        assert!(output.contains("1.0.0"));
        assert!(output.contains("A test tool"));
    }

    #[test]
    fn test_progress_calculation() {
        let mut node_states = HashMap::new();
        node_states.insert(
            "node1".to_string(),
            NodeExecutionState {
                status: ExecutionStatus::Completed,
                started_at: None,
                completed_at: None,
                result: None,
                error: None,
                retry_count: 0,
            },
        );
        node_states.insert(
            "node2".to_string(),
            NodeExecutionState {
                status: ExecutionStatus::Running,
                started_at: None,
                completed_at: None,
                result: None,
                error: None,
                retry_count: 0,
            },
        );

        let progress = calculate_progress(&node_states);
        assert_eq!(progress, "1/2");
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(
            truncate_string("this is a very long string", 10),
            "this is..."
        );
        assert_eq!(truncate_string("exact", 5), "exact");
    }

    #[test]
    fn test_create_formatter() {
        let table_formatter = create_formatter(&OutputFormat::Table);
        let json_formatter = create_formatter(&OutputFormat::Json);
        let yaml_formatter = create_formatter(&OutputFormat::Yaml);
        let text_formatter = create_formatter(&OutputFormat::Text);

        let execution = create_test_execution();

        // Test that all formatters can format without panicking
        let _ = table_formatter.format_workflow_status(&execution);
        let _ = json_formatter.format_workflow_status(&execution);
        let _ = yaml_formatter.format_workflow_status(&execution);
        let _ = text_formatter.format_workflow_status(&execution);
    }
}
