//! Tests for parameter template expansion functionality

use super::*;
use crate::tools::{AsyncFunctionExecutor, BasicTool, BasicToolRegistry, ToolRegistry};
use serde_json::json;
use std::sync::Arc;

#[test]
fn test_template_context_basic() {
    let mut context = TemplateContext::new();
    
    // Test setting and getting variables
    context.set_variable("name", json!("test"));
    context.set_variable("count", json!(42));
    
    assert_eq!(context.get_variable("name"), Some(&json!("test")));
    assert_eq!(context.get_variable("count"), Some(&json!(42)));
    assert_eq!(context.get_variable("missing"), None);
    
    // Test setting multiple variables
    let mut vars = HashMap::new();
    vars.insert("key1".to_string(), json!("value1"));
    vars.insert("key2".to_string(), json!(123));
    context.set_variables(vars);
    
    assert_eq!(context.get_variable("key1"), Some(&json!("value1")));
    assert_eq!(context.get_variable("key2"), Some(&json!(123)));
}

#[test]
fn test_template_engine_variable_expansion() {
    let engine = TemplateEngine::new().unwrap();
    let mut context = TemplateContext::new();
    context.set_variable("name", json!("world"));
    context.set_variable("count", json!(5));
    
    // Basic string expansion
    let result = engine.expand_string("Hello ${name}!", &context).unwrap();
    assert_eq!(result, json!("Hello world!"));
    
    // Number expansion
    let result = engine.expand_string("Count: ${count}", &context).unwrap();
    assert_eq!(result, json!("Count: 5"));
    
    // Multiple variables
    let result = engine.expand_string("${name} has ${count} items", &context).unwrap();
    assert_eq!(result, json!("world has 5 items"));
}

#[test]
fn test_template_engine_json_expansion() {
    let engine = TemplateEngine::new().unwrap();
    let mut context = TemplateContext::new();
    context.set_variable("host", json!("localhost"));
    context.set_variable("port", json!(8080));
    
    let template = json!({
        "server": {
            "host": "${host}",
            "port": "${port}",
            "url": "http://${host}:${port}"
        },
        "config": {
            "debug": true,
            "timeout": 30
        }
    });
    
    let result = engine.expand(&template, &context).unwrap();
    let expected = json!({
        "server": {
            "host": "localhost",
            "port": 8080,
            "url": "http://localhost:8080"
        },
        "config": {
            "debug": true,
            "timeout": 30
        }
    });
    
    assert_eq!(result, expected);
}

#[test]
fn test_template_functions() {
    let engine = TemplateEngine::new().unwrap();
    let mut context = TemplateContext::new();
    context.set_variable("text", json!("hello"));
    context.set_variable("items", json!(["a", "b", "c"]));
    
    // Test upper function
    let result = engine.expand_string("${upper(text)}", &context).unwrap();
    assert_eq!(result, json!("HELLO"));
    
    // Test lower function
    let result = engine.expand_string("${lower('WORLD')}", &context).unwrap();
    assert_eq!(result, json!("world"));
    
    // Test join function
    let result = engine.expand_string("${join(items, '-')}", &context).unwrap();
    assert_eq!(result, json!("a-b-c"));
    
    // Test split function
    let result = engine.expand_string("${split('x,y,z', ',')}", &context).unwrap();
    assert_eq!(result, json!(["x", "y", "z"]));
    
    // Test length function
    let result = engine.expand_string("${length(text)}", &context).unwrap();
    assert_eq!(result, json!(5));
    
    let result = engine.expand_string("${length(items)}", &context).unwrap();
    assert_eq!(result, json!(3));
    
    // Test default function
    let result = engine.expand_string("${default(missing, 'fallback')}", &context).unwrap();
    assert_eq!(result, json!("fallback"));
    
    let result = engine.expand_string("${default(text, 'fallback')}", &context).unwrap();
    assert_eq!(result, json!("hello"));
}

#[test]
fn test_nested_property_access() {
    let engine = TemplateEngine::new().unwrap();
    let mut context = TemplateContext::new();
    context.set_variable("config", json!({
        "database": {
            "host": "db.example.com",
            "port": 5432,
            "credentials": {
                "username": "admin",
                "password": "secret"
            }
        },
        "servers": ["web1", "web2", "web3"]
    }));
    
    // Test nested object access
    let result = engine.expand_string("${config.database.host}", &context).unwrap();
    assert_eq!(result, json!("db.example.com"));
    
    let result = engine.expand_string("${config.database.port}", &context).unwrap();
    assert_eq!(result, json!(5432));
    
    let result = engine.expand_string("${config.database.credentials.username}", &context).unwrap();
    assert_eq!(result, json!("admin"));
    
    // Test array access
    let result = engine.expand_string("${config.servers.0}", &context).unwrap();
    assert_eq!(result, json!("web1"));
    
    let result = engine.expand_string("${config.servers.2}", &context).unwrap();
    assert_eq!(result, json!("web3"));
}

#[test]
fn test_parameter_template() {
    let template = ParameterTemplate::new(
        "test_template".to_string(),
        json!({
            "host": "${host}",
            "port": "${port}",
            "database": "${database}",
            "connection_string": "${host}:${port}/${database}"
        })
    )
    .with_description("Test database template".to_string())
    .with_required_variable("host".to_string())
    .with_default_value("port".to_string(), json!(5432))
    .with_default_value("database".to_string(), json!("testdb"));
    
    let engine = TemplateEngine::new().unwrap();
    let mut context = TemplateContext::new();
    context.set_variable("host", json!("localhost"));
    
    let result = template.expand(&mut context, &engine).unwrap();
    let expected = json!({
        "host": "localhost",
        "port": 5432,
        "database": "testdb",
        "connection_string": "localhost:5432/testdb"
    });
    
    assert_eq!(result, expected);
}

#[test]
fn test_parameter_template_missing_required() {
    let template = ParameterTemplate::new(
        "test_template".to_string(),
        json!({"host": "${host}"})
    )
    .with_required_variable("host".to_string());
    
    let engine = TemplateEngine::new().unwrap();
    let mut context = TemplateContext::new();
    // Don't set the required variable
    
    let result = template.expand(&mut context, &engine);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Required template variable 'host' not provided"));
}

#[tokio::test]
async fn test_tool_with_parameter_templates() {
    let executor = Arc::new(AsyncFunctionExecutor::new(|params, _context| async move {
        Ok(json!({
            "processed": true,
            "input": params
        }))
    }));
    
    let template = ParameterTemplate::new(
        "config_template".to_string(),
        json!({
            "server_url": "https://${host}:${port}/api",
            "timeout": "${timeout}",
            "retries": "${retries}"
        })
    )
    .with_required_variable("host".to_string())
    .with_default_value("port".to_string(), json!(443))
    .with_default_value("timeout".to_string(), json!(30))
    .with_default_value("retries".to_string(), json!(3));
    
    let tool = BasicTool::builder()
        .name("api_client")
        .version("1.0.0")
        .description("API client tool")
        .parameter_template(template)
        .executor(executor)
        .build()
        .unwrap();
    
    let mut registry = BasicToolRegistry::new();
    registry.register_tool(Arc::new(tool)).unwrap();
    
    // Test template expansion through registry
    let mut template_context = TemplateContext::new();
    template_context.set_variable("host", json!("api.example.com"));
    template_context.set_variable("timeout", json!(60));
    
    let params = json!({
        "server_url": "https://${host}:${port}/api",
        "timeout": "${timeout}"
    });
    
    let execution_context = crate::core::ExecutionContext::new();
    let result = registry.execute_tool_with_templates(
        "api_client",
        params,
        &template_context,
        execution_context
    ).await.unwrap();
    
    let expected_input = json!({
        "server_url": "https://api.example.com:443/api",
        "timeout": 60
    });
    
    assert_eq!(result["processed"], json!(true));
    assert_eq!(result["input"], expected_input);
    
    // Test getting templates from registry
    let templates = registry.get_tool_templates("api_client");
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].name, "config_template");
    assert_eq!(templates[0].required_variables, vec!["host"]);
}

#[test]
fn test_env_function() {
    std::env::set_var("TEST_VAR", "test_value");
    
    let engine = TemplateEngine::new().unwrap();
    let context = TemplateContext::new();
    
    let result = engine.expand_string("${env('TEST_VAR')}", &context).unwrap();
    assert_eq!(result, json!("test_value"));
    
    // Test non-existent environment variable
    let result = engine.expand_string("${env('NON_EXISTENT_VAR')}", &context).unwrap();
    assert_eq!(result, json!(null));
}

#[test]
fn test_format_function() {
    let engine = TemplateEngine::new().unwrap();
    let context = TemplateContext::new();
    
    let result = engine.expand_string("${format('Hello {0}, you have {1} items', 'Alice', 5)}", &context).unwrap();
    assert_eq!(result, json!("Hello Alice, you have 5 items"));
    
    let result = engine.expand_string("${format('Value: {0}', 42)}", &context).unwrap();
    assert_eq!(result, json!("Value: 42"));
}

#[test]
fn test_error_handling() {
    let engine = TemplateEngine::new().unwrap();
    let context = TemplateContext::new();
    
    // Test unknown variable (should return error)
    let result = engine.expand_string("${unknown_variable}", &context);
    assert!(result.is_err());
    
    // Test unknown function
    let result = engine.expand_string("${unknown_function()}", &context);
    assert!(result.is_err());
    
    // Test invalid function arguments
    let result = engine.expand_string("${upper()}", &context); // missing argument
    assert!(result.is_err());
    
    let result = engine.expand_string("${join('not_an_array', ',')}", &context);
    assert!(result.is_err());
}

#[test]
fn test_complex_template_scenario() {
    let engine = TemplateEngine::new().unwrap();
    let mut context = TemplateContext::new();
    
    context.set_variable("app_name", json!("my-service"));
    context.set_variable("environment", json!("production"));
    context.set_variable("version", json!("1.2.3"));
    context.set_variable("config", json!({
        "database": {
            "host": "prod-db.example.com",
            "port": 5432
        },
        "redis": {
            "host": "prod-redis.example.com",
            "port": 6379
        }
    }));
    
    let complex_template = json!({
        "deployment": {
            "name": "${app_name}-${environment}",
            "image": "${app_name}:${version}",
            "environment_vars": {
                "APP_ENV": "${upper(environment)}",
                "DATABASE_URL": "postgresql://user:pass@${config.database.host}:${config.database.port}/mydb",
                "REDIS_URL": "redis://${config.redis.host}:${config.redis.port}",
                "SERVICE_NAME": "${format('{0}-{1}', app_name, environment)}"
            }
        }
    });
    
    let result = engine.expand(&complex_template, &context).unwrap();
    
    let expected = json!({
        "deployment": {
            "name": "my-service-production",
            "image": "my-service:1.2.3",
            "environment_vars": {
                "APP_ENV": "PRODUCTION",
                "DATABASE_URL": "postgresql://user:pass@prod-db.example.com:5432/mydb",
                "REDIS_URL": "redis://prod-redis.example.com:6379",
                "SERVICE_NAME": "my-service-production"
            }
        }
    });
    
    assert_eq!(result, expected);
}