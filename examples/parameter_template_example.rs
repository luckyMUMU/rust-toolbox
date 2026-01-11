//! Example demonstrating parameter template expansion functionality

use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use workflow_toolkit::{
    core::ExecutionContext,
    error::Result,
    tools::{
        AsyncFunctionExecutor, BasicTool, BasicToolRegistry, ParameterTemplate, TemplateContext,
        TemplateEngine, ToolRegistry,
    },
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== Parameter Template Expansion Example ===\n");

    // Create a template engine
    let engine = TemplateEngine::new()?;

    // Example 1: Basic variable substitution
    println!("1. Basic Variable Substitution:");
    basic_variable_substitution(&engine)?;

    // Example 2: Template functions
    println!("\n2. Template Functions:");
    template_functions(&engine)?;

    // Example 3: Nested property access
    println!("\n3. Nested Property Access:");
    nested_property_access(&engine)?;

    // Example 4: Parameter templates with tools
    println!("\n4. Parameter Templates with Tools:");
    parameter_templates_with_tools().await?;

    // Example 5: Complex template scenarios
    println!("\n5. Complex Template Scenarios:");
    complex_template_scenarios(&engine)?;

    println!("\n=== Example completed successfully ===");
    Ok(())
}

fn basic_variable_substitution(engine: &TemplateEngine) -> Result<()> {
    let mut context = TemplateContext::new();
    context.set_variable("name", json!("Alice"));
    context.set_variable("age", json!(30));
    context.set_variable("city", json!("New York"));

    let template = "Hello ${name}! You are ${age} years old and live in ${city}.";
    let result = engine.expand_string(template, &context)?;

    println!("  Template: {}", template);
    println!("  Result: {}", result);

    // Test with JSON object template
    let json_template = json!({
        "greeting": "Hello ${name}!",
        "info": {
            "age": "${age}",
            "location": "${city}",
            "full_info": "Name: ${name}, Age: ${age}, City: ${city}"
        }
    });

    let expanded_json = engine.expand(&json_template, &context)?;
    println!(
        "  JSON Template: {}",
        serde_json::to_string_pretty(&json_template)?
    );
    println!(
        "  Expanded JSON: {}",
        serde_json::to_string_pretty(&expanded_json)?
    );

    Ok(())
}

fn template_functions(engine: &TemplateEngine) -> Result<()> {
    let mut context = TemplateContext::new();
    context.set_variable("text", json!("hello world"));
    context.set_variable("items", json!(["apple", "banana", "cherry"]));
    context.set_variable("empty_value", json!(null));

    // Set environment variable for testing
    std::env::set_var("TEST_ENV_VAR", "environment_value");

    let examples = vec![
        ("${upper(text)}", "Convert to uppercase"),
        ("${lower('HELLO WORLD')}", "Convert to lowercase"),
        ("${join(items, ', ')}", "Join array with separator"),
        ("${split('a,b,c', ',')}", "Split string into array"),
        ("${length(text)}", "Get string length"),
        ("${length(items)}", "Get array length"),
        (
            "${default(empty_value, 'default_text')}",
            "Use default value",
        ),
        ("${env('TEST_ENV_VAR')}", "Get environment variable"),
        (
            "${format('Hello {0}, you have {1} items', 'Bob', 5)}",
            "Format string",
        ),
    ];

    for (template, description) in examples {
        let result = engine.expand_string(template, &context)?;
        println!("  {} -> {} ({})", template, result, description);
    }

    Ok(())
}

fn nested_property_access(engine: &TemplateEngine) -> Result<()> {
    let mut context = TemplateContext::new();
    context.set_variable(
        "config",
        json!({
            "database": {
                "host": "localhost",
                "port": 5432,
                "credentials": {
                    "username": "admin",
                    "password": "secret"
                }
            },
            "servers": ["web1", "web2", "web3"]
        }),
    );

    let examples = vec![
        ("${config.database.host}", "Access nested object property"),
        ("${config.database.port}", "Access nested number property"),
        (
            "${config.database.credentials.username}",
            "Access deeply nested property",
        ),
        ("${config.servers.0}", "Access array element by index"),
        ("${config.servers.2}", "Access another array element"),
    ];

    for (template, description) in examples {
        let result = engine.expand_string(template, &context)?;
        println!("  {} -> {} ({})", template, result, description);
    }

    // Complex template combining multiple features
    let complex_template = "postgresql://${config.database.credentials.username}:${config.database.credentials.password}@${config.database.host}:${config.database.port}/myapp";
    let result = engine.expand_string(complex_template, &context)?;
    println!("  Complex: {} -> {}", complex_template, result);

    Ok(())
}

async fn parameter_templates_with_tools() -> Result<()> {
    let mut registry = BasicToolRegistry::new();

    // Create a database connection tool with parameter templates
    let db_executor = Arc::new(AsyncFunctionExecutor::new(|params, _context| async move {
        println!("    Connecting to database with params: {}", params);
        Ok(json!({
            "status": "connected",
            "connection_string": params.get("connection_string").unwrap_or(&json!("unknown")),
            "pool_size": params.get("pool_size").unwrap_or(&json!(10))
        }))
    }));

    // Create parameter templates for the database tool
    let connection_template = ParameterTemplate::new(
        "database_connection".to_string(),
        json!({
            "connection_string": "postgresql://${username}:${password}@${host}:${port}/${database}",
            "pool_size": "${pool_size}",
            "ssl_mode": "${ssl_mode}",
            "timeout": "${timeout}"
        }),
    )
    .with_description("Database connection configuration template".to_string())
    .with_required_variable("host".to_string())
    .with_required_variable("username".to_string())
    .with_required_variable("password".to_string())
    .with_default_value("port".to_string(), json!(5432))
    .with_default_value("database".to_string(), json!("myapp"))
    .with_default_value("pool_size".to_string(), json!(10))
    .with_default_value("ssl_mode".to_string(), json!("prefer"))
    .with_default_value("timeout".to_string(), json!(30));

    let db_tool = BasicTool::builder()
        .name("database_connect")
        .version("1.0.0")
        .description("Connect to database with template parameters")
        .parameters_schema(json!({
            "type": "object",
            "properties": {
                "connection_string": {"type": "string"},
                "pool_size": {"type": "integer"},
                "ssl_mode": {"type": "string"},
                "timeout": {"type": "integer"}
            },
            "required": ["connection_string"]
        }))
        .parameter_template(connection_template)
        .executor(db_executor)
        .build()?;

    registry.register_tool(Arc::new(db_tool))?;

    // Create template context with variables
    let mut template_context = TemplateContext::new();
    template_context.set_variable("host", json!("prod-db.example.com"));
    template_context.set_variable("username", json!("app_user"));
    template_context.set_variable("password", json!("secure_password"));
    template_context.set_variable("database", json!("production"));
    template_context.set_variable("pool_size", json!(20));

    // Execute tool with template expansion
    let execution_context = ExecutionContext::new();
    let params = json!({
        "connection_string": "${host}:${port}/${database}",
        "pool_size": "${pool_size}"
    });

    println!(
        "  Original params: {}",
        serde_json::to_string_pretty(&params)?
    );

    let result = registry
        .execute_tool_with_templates(
            "database_connect",
            params,
            &template_context,
            execution_context,
        )
        .await?;

    println!(
        "  Execution result: {}",
        serde_json::to_string_pretty(&result)?
    );

    // Show available templates for the tool
    let templates = registry.get_tool_templates("database_connect");
    println!(
        "  Available templates for 'database_connect': {} templates",
        templates.len()
    );
    for template in templates {
        println!(
            "    - {}: {}",
            template.name,
            template.description.unwrap_or_default()
        );
        println!(
            "      Required variables: {:?}",
            template.required_variables
        );
        println!("      Default values: {:?}", template.default_values);
    }

    Ok(())
}

fn complex_template_scenarios(engine: &TemplateEngine) -> Result<()> {
    let mut context = TemplateContext::new();

    // Set up complex context
    context.set_variable("environment", json!("production"));
    context.set_variable("service_name", json!("user-service"));
    context.set_variable("version", json!("1.2.3"));
    context.set_variable("replicas", json!(3));
    context.set_variable(
        "resources",
        json!({
            "cpu": "500m",
            "memory": "512Mi"
        }),
    );
    context.set_variable(
        "secrets",
        json!({
            "db_password": "secret123",
            "api_key": "key456"
        }),
    );

    // Complex Kubernetes deployment template
    let k8s_template = json!({
        "apiVersion": "apps/v1",
        "kind": "Deployment",
        "metadata": {
            "name": "${service_name}-${environment}",
            "labels": {
                "app": "${service_name}",
                "version": "${version}",
                "environment": "${environment}"
            }
        },
        "spec": {
            "replicas": "${replicas}",
            "selector": {
                "matchLabels": {
                    "app": "${service_name}",
                    "environment": "${environment}"
                }
            },
            "template": {
                "metadata": {
                    "labels": {
                        "app": "${service_name}",
                        "version": "${version}",
                        "environment": "${environment}"
                    }
                },
                "spec": {
                    "containers": [{
                        "name": "${service_name}",
                        "image": "${service_name}:${version}",
                        "resources": {
                            "requests": {
                                "cpu": "${resources.cpu}",
                                "memory": "${resources.memory}"
                            },
                            "limits": {
                                "cpu": "${resources.cpu}",
                                "memory": "${resources.memory}"
                            }
                        },
                        "env": [
                            {
                                "name": "ENVIRONMENT",
                                "value": "${upper(environment)}"
                            },
                            {
                                "name": "SERVICE_NAME",
                                "value": "${service_name}"
                            },
                            {
                                "name": "DB_PASSWORD",
                                "value": "${secrets.db_password}"
                            }
                        ]
                    }]
                }
            }
        }
    });

    println!("  Expanding complex Kubernetes deployment template...");
    let expanded = engine.expand(&k8s_template, &context)?;
    println!("  Result: {}", serde_json::to_string_pretty(&expanded)?);

    // Conditional template expansion
    context.set_variable("enable_monitoring", json!(true));
    context.set_variable("monitoring_port", json!(9090));

    let conditional_template = json!({
        "monitoring": {
            "enabled": "${enable_monitoring}",
            "port": "${default(monitoring_port, 8080)}",
            "endpoint": "/metrics"
        }
    });

    println!("\n  Conditional template expansion:");
    let conditional_result = engine.expand(&conditional_template, &context)?;
    println!(
        "  Result: {}",
        serde_json::to_string_pretty(&conditional_result)?
    );

    Ok(())
}
