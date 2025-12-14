use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use async_trait::async_trait;
use uuid::Uuid;
use crate::error::{CoreError, Result};
use crate::tool::Tool;
use crate::Locale;
use std::sync::Arc;
use tokio::sync::RwLock;
use regex::Regex;

// --- Data Structures ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub id: String,
    pub tool_name: String,
    pub label: Option<String>,
    pub input_mappings: HashMap<String, String>, // Field -> Template "{{ node.output.field }}"
    pub static_inputs: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecutionState {
    pub status: NodeStatus,
    pub output: Option<Value>,
    pub error: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

impl Default for NodeExecutionState {
    fn default() -> Self {
        Self {
            status: NodeStatus::Pending,
            output: None,
            error: None,
            start_time: None,
            end_time: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstance {
    pub id: String,
    pub def: WorkflowDefinition,
    pub status: WorkflowStatus,
    pub node_states: HashMap<String, NodeExecutionState>,
    pub context: HashMap<String, Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// --- Engine Trait ---

#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    async fn validate(&self, def: &WorkflowDefinition) -> Result<()>;
    async fn start_workflow(&self, def: WorkflowDefinition) -> Result<String>;
    async fn get_status(&self, instance_id: &str) -> Result<WorkflowInstance>;
    async fn pause_workflow(&self, instance_id: &str) -> Result<()>;
    async fn stop_workflow(&self, instance_id: &str) -> Result<()>;
    // async fn get_logs(&self, instance_id: &str) -> Result<Vec<LogEntry>>; // Todo
}

// --- Engine Implementation --- 

/// Arc<Tool> 适配器，用于将 Arc<dyn Tool> 转换为 Box<dyn Tool>
struct ArcToolAdapter(Arc<dyn Tool>);

#[async_trait]
impl Tool for ArcToolAdapter {
    fn name(&self) -> &str {
        self.0.name()
    }
    
    fn display_name(&self, locale: Locale) -> String {
        self.0.display_name(locale)
    }
    
    fn description(&self, locale: Locale) -> String {
        self.0.description(locale)
    }
    
    fn user_guide(&self, locale: Locale) -> String {
        self.0.user_guide(locale)
    }
    
    fn input_schema(&self, locale: Locale) -> Value {
        self.0.input_schema(locale)
    }
    
    fn output_schema(&self, locale: Locale) -> Value {
        self.0.output_schema(locale)
    }
    
    async fn run(&self, input: Value) -> Result<Value> {
        self.0.run(input).await
    }
}

pub struct InMemoryWorkflowEngine {
    tools: Arc<HashMap<String, Box<dyn Tool>>>,
    instances: Arc<RwLock<HashMap<String, Arc<RwLock<WorkflowInstance>>>>>,
}

impl InMemoryWorkflowEngine {
    pub fn new(tools: HashMap<String, Box<dyn Tool>>) -> Self {
        Self {
            tools: Arc::new(tools),
            instances: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 创建工作流引擎实例，接受 Arc<Tool> 类型的工具
    pub fn new_with_arc(tools: HashMap<String, Arc<dyn Tool>>) -> Self {
        let mut box_tools: HashMap<String, Box<dyn Tool>> = HashMap::new();
        for (name, tool) in tools {
            // 使用适配器将 Arc<dyn Tool> 转换为 Box<dyn Tool>
            box_tools.insert(name, Box::new(ArcToolAdapter(tool)));
        }
        Self::new(box_tools)
    }

    async fn execute_workflow_loop(
        instance_lock: Arc<RwLock<WorkflowInstance>>,
        tools: Arc<HashMap<String, Box<dyn Tool>>>,
    ) {
        loop {
            // 1. Check workflow status
            {
                let inst = instance_lock.read().await;
                if matches!(inst.status, WorkflowStatus::Completed | WorkflowStatus::Failed(_) | WorkflowStatus::Paused) {
                    break;
                }
            }

            // 2. Find runnable nodes
            let mut runnable_nodes = Vec::new();
            let mut all_completed = true;
            
            // Scope for read lock
            {
                let inst = instance_lock.read().await;
                // let node_map: HashMap<String, &WorkflowNode> = inst.def.nodes.iter().map(|n| (n.id.clone(), n)).collect();
                
                // Build adjacency list for checking dependencies
                // Actually, we iterate nodes and check if their dependencies (incoming edges) are met
                
                for node in &inst.def.nodes {
                    let state = inst.node_states.get(&node.id).unwrap();
                    
                    if state.status == NodeStatus::Pending {
                        all_completed = false;
                        
                        // Check dependencies
                        let incoming_edges: Vec<&WorkflowEdge> = inst.def.edges.iter().filter(|e| e.to == node.id).collect();
                        let mut deps_met = true;
                        
                        for edge in incoming_edges {
                            let parent_state = inst.node_states.get(&edge.from).unwrap();
                            if parent_state.status != NodeStatus::Completed {
                                deps_met = false;
                                break;
                            }
                        }
                        
                        if deps_met {
                            runnable_nodes.push(node.clone());
                        }
                    } else if matches!(state.status, NodeStatus::Running) {
                        all_completed = false;
                        // Already running, skip
                    } else if matches!(state.status, NodeStatus::Failed(_)) {
                        all_completed = false;
                        // If any node failed, we might want to fail the workflow or handle error paths.
                        // For MVP, fail workflow.
                    }
                }

                // If any node failed, fail workflow
                let mut failure = None;
                for state in inst.node_states.values() {
                     if let NodeStatus::Failed(ref e) = state.status {
                         failure = Some(e.clone());
                         break;
                     }
                }
                
                if let Some(e) = failure {
                    drop(inst); // drop read lock
                    let mut w = instance_lock.write().await;
                    w.status = WorkflowStatus::Failed(format!("Node failed: {}", e));
                    w.updated_at = Utc::now();
                    return;
                }
            }

            if all_completed {
                let mut w = instance_lock.write().await;
                w.status = WorkflowStatus::Completed;
                w.updated_at = Utc::now();
                break;
            }

            if runnable_nodes.is_empty() {
                // No nodes runnable, but not all completed.
                // Could be:
                // 1. Running nodes (wait for them)
                // 2. Cycle or deadlock (should be caught by validation, but runtime check is good)
                // For now, simple sleep and retry. Better: use channels/notify.
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                continue;
            }

            // 3. Execute runnable nodes (Concurrent spawn)
            let mut handles = vec![];
            
            for node in runnable_nodes {
                let inst_lock_clone = instance_lock.clone();
                let tools_clone = tools.clone();
                
                // Mark as running
                {
                    let mut w = instance_lock.write().await;
                    if let Some(s) = w.node_states.get_mut(&node.id) {
                        s.status = NodeStatus::Running;
                        s.start_time = Some(Utc::now());
                    }
                }

                // Prepare input (needs read lock for context)
                let input_result = {
                    let r = instance_lock.read().await;
                    Self::resolve_inputs(&node, &r.context)
                };

                handles.push(tokio::spawn(async move {
                    let res = match input_result {
                        Ok(input) => {
                             if let Some(tool) = tools_clone.get(&node.tool_name) {
                                 tool.run(input).await
                             } else {
                                 Err(CoreError::ConfigError(format!("Tool not found: {}", node.tool_name)))
                             }
                        },
                        Err(e) => Err(e),
                    };

                    // Update state
                    let mut w = inst_lock_clone.write().await;
                    if let Some(s) = w.node_states.get_mut(&node.id) {
                        s.end_time = Some(Utc::now());
                        match res {
                            Ok(output) => {
                                s.status = NodeStatus::Completed;
                                s.output = Some(output.clone());
                                // Update context
                                w.context.insert(node.id.clone(), output);
                            }
                            Err(e) => {
                                s.status = NodeStatus::Failed(e.to_string());
                                s.error = Some(e.to_string());
                            }
                        }
                    }
                }));
            }
            
            // Wait for all spawned tasks in this batch to finish? 
            // Or just continue loop?
            // If we continue loop, we might re-schedule same nodes if status update is slow.
            // But we marked them as Running, so they won't be picked up again.
            // However, we want to pick up *children* as soon as *one* parent finishes.
            // So we shouldn't wait for ALL.
            // But for simplicity in MVP, let's wait for the batch. 
            // To make it truly async, we should just spawn and let the loop run.
            // The loop will sleep if nothing is Pending && deps met.
            // When a task finishes, it updates status to Completed. The loop wakes up (after sleep) and sees new runnable nodes.
            // So, just spawning is fine.
        }
    }

    fn resolve_inputs(node: &WorkflowNode, context: &HashMap<String, Value>) -> Result<Value> {
        let mut input = node.static_inputs.clone();
        
        // 1. Resolve values existing in static_inputs (recursively, for inline templates)
        Self::deep_resolve(&mut input, context)?;

        // 2. Apply top-level mappings (overwrite or insert)
        if let Value::Object(ref mut map) = input {
            for (key, template) in &node.input_mappings {
                 let val = Self::resolve_template(template, context)?;
                 map.insert(key.clone(), val);
            }
        } else {
             // If input is not an object, we can't apply mappings keyed by string.
             // But tools input should be an object.
        }
        
        Ok(input)
    }

    fn deep_resolve(val: &mut Value, context: &HashMap<String, Value>) -> Result<()> {
        match val {
            Value::Object(map) => {
                for v in map.values_mut() {
                    Self::deep_resolve(v, context)?;
                }
            }
            Value::String(s) => {
                 if s.contains("{{") && s.contains("}}") {
                     let resolved = Self::resolve_template(s, context)?;
                     *val = resolved;
                 }
            }
            Value::Array(arr) => {
                for v in arr {
                    Self::deep_resolve(v, context)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn resolve_template(template: &str, context: &HashMap<String, Value>) -> Result<Value> {
        // Regex to find {{ node.output.path }}
        // For MVP: assume full string is one template or simple substitution
        // Let's handle: "Some text {{ node.output.val }}" -> "Some text value"
        
        let re = Regex::new(r"\{\{\s*([a-zA-Z0-9_.]+)\s*\}\}").unwrap();
        
        // If the whole string is the template, return the Value directly (preserving type)
        if let Some(caps) = re.captures(template) {
            if caps.get(0).unwrap().as_str() == template {
                let path = caps.get(1).unwrap().as_str();
                return Self::get_value_by_path(path, context);
            }
        }
        
        // Otherwise, string replacement
        let mut result = template.to_string();
        for caps in re.captures_iter(template) {
            let full_match = caps.get(0).unwrap().as_str();
            let path = caps.get(1).unwrap().as_str();
            let val = Self::get_value_by_path(path, context)?;
            
            // Convert val to string for replacement
            let val_str = match val {
                Value::String(s) => s,
                _ => val.to_string(),
            };
            result = result.replace(full_match, &val_str);
        }
        
        Ok(Value::String(result))
    }

    fn get_value_by_path(path: &str, context: &HashMap<String, Value>) -> Result<Value> {
        // path: node_id.output.field.subfield
        let parts: Vec<&str> = path.split('.').collect();
        if parts.len() < 2 {
            return Err(CoreError::InvalidInput(format!("Invalid path format: {}", path)));
        }
        
        let node_id = parts[0];
        let _output_keyword = parts[1]; // should be "output"
        
        let node_val = context.get(node_id).ok_or_else(|| CoreError::InvalidInput(format!("Node output not found: {}", node_id)))?;
        
        let mut current = node_val;
        for part in &parts[2..] {
            current = current.get(part).ok_or_else(|| CoreError::InvalidInput(format!("Field not found: {} in {}", part, path)))?;
        }
        
        Ok(current.clone())
    }
}

#[async_trait]
impl WorkflowEngine for InMemoryWorkflowEngine {
    async fn validate(&self, def: &WorkflowDefinition) -> Result<()> {
        // 1. Check if tools exist
        for node in &def.nodes {
            if !self.tools.contains_key(&node.tool_name) {
                return Err(CoreError::ConfigError(format!("Tool not found: {}", node.tool_name)));
            }
        }
        // 2. Cycle detection (Todo)
        Ok(())
    }

    async fn start_workflow(&self, def: WorkflowDefinition) -> Result<String> {
        self.validate(&def).await?;
        
        let id = Uuid::new_v4().to_string();
        let mut node_states = HashMap::new();
        for node in &def.nodes {
            node_states.insert(node.id.clone(), NodeExecutionState::default());
        }

        let instance = WorkflowInstance {
            id: id.clone(),
            def,
            status: WorkflowStatus::Running,
            node_states,
            context: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let instance_lock = Arc::new(RwLock::new(instance));
        self.instances.write().await.insert(id.clone(), instance_lock.clone());

        // Spawn execution loop
        let tools = self.tools.clone();
        tokio::spawn(async move {
            Self::execute_workflow_loop(instance_lock, tools).await;
        });

        Ok(id)
    }

    async fn get_status(&self, instance_id: &str) -> Result<WorkflowInstance> {
        let map = self.instances.read().await;
        if let Some(inst_lock) = map.get(instance_id) {
            let inst = inst_lock.read().await;
            Ok(inst.clone())
        } else {
            Err(CoreError::InvalidInput("Workflow instance not found".to_string()))
        }
    }

    async fn pause_workflow(&self, instance_id: &str) -> Result<()> {
        let map = self.instances.read().await;
        if let Some(inst_lock) = map.get(instance_id) {
            let mut inst = inst_lock.write().await;
            inst.status = WorkflowStatus::Paused;
            Ok(())
        } else {
            Err(CoreError::InvalidInput("Workflow instance not found".to_string()))
        }
    }

    async fn stop_workflow(&self, instance_id: &str) -> Result<()> {
        let map = self.instances.read().await;
        if let Some(inst_lock) = map.get(instance_id) {
            let mut inst = inst_lock.write().await;
            inst.status = WorkflowStatus::Failed("Stopped by user".to_string());
            Ok(())
        } else {
            Err(CoreError::InvalidInput("Workflow instance not found".to_string()))
        }
    }
}
