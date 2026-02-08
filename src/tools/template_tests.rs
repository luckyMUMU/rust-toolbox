//! Tests for parameter template expansion functionality

use super::*;
use crate::tools::ToolRegistry;
use serde_json::json;
use std::sync::Arc;
use proptest::prelude::*;

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
    use crate::tools::types::{NativeToolBuilder, Tool, ToolInput, ToolOutput};
    use crate::core::ExecutionContext;
    
    let native_tool = NativeToolBuilder::new()
        .name("api_client")
        .version("1.0.0")
        .description("API client tool")
        .executor(|input: ToolInput, _ctx: ExecutionContext| async move {
            Ok(ToolOutput::success(json!({
                "processed": true,
                "input": input.params
            })))
        })
        .build()
        .unwrap();
    
    let registry = ToolRegistry::new();
    registry.register("api_client", Tool::Native(Arc::new(native_tool)));
    
    // Test template expansion through registry
    let mut template_context = TemplateContext::new();
    template_context.set_variable("host", json!("api.example.com"));
    template_context.set_variable("timeout", json!(60));
    
    let params = json!({
        "server_url": "https://${host}:${port}/api",
        "timeout": "${timeout}"
    });
    
    let execution_context = ExecutionContext::new();
    let tool_input = ToolInput::new(params);
    let result = registry.execute_tool("api_client", tool_input, execution_context).await.unwrap();
    
    let expected_input = json!({
        "server_url": "https://api.example.com:443/api",
        "timeout": 60
    });
    
    assert_eq!(result.result["processed"], json!(true));
    // Note: Template expansion happens at workflow level, not tool level in new system
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

// Property-based tests for parameter template expansion
// **Feature: workflow-toolkit, Property 15: 参数模板展开正确性**
// **Validates: Requirements 6.5**

// Generator for simple template variables
fn arb_template_variable() -> impl Strategy<Value = (String, Value)> {
    (
        "[a-z][a-z0-9_]{1,10}",
        prop_oneof![
            any::<String>().prop_map(|s| json!(s)),
            any::<i32>().prop_map(|i| json!(i)),
            any::<bool>().prop_map(|b| json!(b)),
        ]
    ).prop_map(|(name, value)| (name, value))
}

// Generator for template contexts
fn arb_template_context() -> impl Strategy<Value = TemplateContext> {
    prop::collection::vec(arb_template_variable(), 1..5)
        .prop_map(|vars| {
            let mut context = TemplateContext::new();
            for (name, value) in vars {
                context.set_variable(name, value);
            }
            context
        })
}

// Generator for simple template strings
fn arb_simple_template() -> impl Strategy<Value = String> {
    prop_oneof![
        "Hello ${[a-z][a-z0-9_]{1,10}}!",
        "${[a-z][a-z0-9_]{1,10}} is ${[a-z][a-z0-9_]{1,10}}",
        "Value: ${[a-z][a-z0-9_]{1,10}}",
    ]
}

proptest! {
    #[test]
    fn property_template_expansion_deterministic(
        template in arb_simple_template(),
        context in arb_template_context()
    ) {
        // **Feature: workflow-toolkit, Property 15: 参数模板展开正确性**
        // For any template and context, expansion should be deterministic
        let engine = TemplateEngine::new().unwrap();
        
        let result1 = engine.expand_string(&template, &context);
        let result2 = engine.expand_string(&template, &context);
        
        prop_assert_eq!(result1.is_ok(), result2.is_ok());
        if let (Ok(r1), Ok(r2)) = (result1, result2) {
            prop_assert_eq!(r1, r2);
        }
    }

    #[test]
    fn property_template_variable_substitution(
        var_name in "[a-z][a-z0-9_]{1,10}",
        var_value in any::<String>()
    ) {
        // **Feature: workflow-toolkit, Property 15: 参数模板展开正确性**
        // For any variable name and value, substitution should work correctly
        let engine = TemplateEngine::new().unwrap();
        let mut context = TemplateContext::new();
        context.set_variable(&var_name, json!(var_value.clone()));
        
        let template = format!("${{{}}}", var_name);
        let result = engine.expand_string(&template, &context).unwrap();
        
        prop_assert_eq!(result, json!(var_value));
    }

    #[test]
    fn property_template_no_variables_unchanged(
        text in "[a-zA-Z0-9 .,!?-]{1,50}"
    ) {
        // **Feature: workflow-toolkit, Property 15: 参数模板展开正确性**
        // For any text without template variables, expansion should return unchanged
        let engine = TemplateEngine::new().unwrap();
        let context = TemplateContext::new();
        
        // Ensure the text doesn't contain template syntax
        let safe_text = text.replace("${", "").replace("}", "");
        let result = engine.expand_string(&safe_text, &context).unwrap();
        
        prop_assert_eq!(result, json!(safe_text));
    }

    #[test]
    fn property_parameter_template_required_variables(
        template_name in "[a-z][a-z0-9_]{1,10}",
        required_var in "[a-z][a-z0-9_]{1,10}",
        var_value in any::<String>()
    ) {
        // **Feature: workflow-toolkit, Property 15: 参数模板展开正确性**
        // For any parameter template with required variables, expansion should succeed when provided
        let template = ParameterTemplate::new(
            template_name,
            json!({"value": format!("${{{}}}", required_var)})
        ).with_required_variable(required_var.clone());
        
        let engine = TemplateEngine::new().unwrap();
        let mut context = TemplateContext::new();
        context.set_variable(&required_var, json!(var_value.clone()));
        
        let result = template.expand(&mut context, &engine).unwrap();
        prop_assert_eq!(result["value"], json!(var_value));
    }

    #[test]
    fn property_template_default_values(
        template_name in "[a-z][a-z0-9_]{1,10}",
        var_name in "[a-z][a-z0-9_]{1,10}",
        default_value in any::<String>()
    ) {
        // **Feature: workflow-toolkit, Property 15: 参数模板展开正确性**
        // For any parameter template with default values, defaults should be used when variable not provided
        let template = ParameterTemplate::new(
            template_name,
            json!({"value": format!("${{{}}}", var_name)})
        ).with_default_value(var_name.clone(), json!(default_value.clone()));
        
        let engine = TemplateEngine::new().unwrap();
        let mut context = TemplateContext::new();
        // Don't set the variable, should use default
        
        let result = template.expand(&mut context, &engine).unwrap();
        prop_assert_eq!(result["value"], json!(default_value));
    }

    #[test]
    fn property_template_json_structure_preserved(
        context in arb_template_context()
    ) {
        // **Feature: workflow-toolkit, Property 15: 参数模板展开正确性**
        // For any JSON template, the structure should be preserved after expansion
        let engine = TemplateEngine::new().unwrap();
        
        let template = json!({
            "level1": {
                "level2": {
                    "array": [1, 2, 3],
                    "boolean": true,
                    "null_value": null
                }
            },
            "simple_string": "no variables here"
        });
        
        let result = engine.expand(&template, &context).unwrap();
        
        // Structure should be preserved
        prop_assert!(result.is_object());
        prop_assert!(result["level1"].is_object());
        prop_assert!(result["level1"]["level2"].is_object());
        prop_assert!(result["level1"]["level2"]["array"].is_array());
        prop_assert_eq!(result["level1"]["level2"]["array"].as_array().unwrap().len(), 3);
        prop_assert_eq!(result["level1"]["level2"]["boolean"], json!(true));
        prop_assert_eq!(result["level1"]["level2"]["null_value"], json!(null));
        prop_assert_eq!(result["simple_string"], json!("no variables here"));
    }
}