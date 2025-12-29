//! Integration tests for parameter template expansion
//! **Feature: workflow-toolkit, Property 15: 参数模板展开正确性**
//! **Validates: Requirements 6.5**

use workflow_toolkit::tools::template::*;
use serde_json::{json, Value};
use proptest::prelude::*;

// Generator for simple template variables
fn arb_template_variable() -> impl Strategy<Value = (String, Value)> {
    (
        "[a-z][a-z0-9_]{1,10}",
        prop_oneof![
            "[a-zA-Z][a-zA-Z0-9 ]{0,20}".prop_map(|s| json!(s)), // Non-numeric strings
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

proptest! {
    #[test]
    fn property_template_expansion_deterministic(
        context in arb_template_context()
    ) {
        // **Feature: workflow-toolkit, Property 15: 参数模板展开正确性**
        // For any template and context, expansion should be deterministic
        let engine = TemplateEngine::new().unwrap();
        let template = "Hello ${name}!";
        
        let result1 = engine.expand_string(template, &context);
        let result2 = engine.expand_string(template, &context);
        
        prop_assert_eq!(result1.is_ok(), result2.is_ok());
        if let (Ok(r1), Ok(r2)) = (result1, result2) {
            prop_assert_eq!(r1, r2);
        }
    }

    #[test]
    fn property_template_variable_substitution(
        var_name in "[a-z][a-z0-9_]{1,10}",
        var_value in "[a-zA-Z][a-zA-Z0-9 ]{1,20}" // Non-numeric strings only
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
        text in "[a-zA-Z .,!?-]{1,50}"
    ) {
        // **Feature: workflow-toolkit, Property 15: 参数模板展开正确性**
        // For any text without template variables, expansion should return unchanged
        let engine = TemplateEngine::new().unwrap();
        let context = TemplateContext::new();
        
        // Ensure the text doesn't contain template syntax and isn't a number
        let safe_text = text.replace("${", "").replace("}", "");
        if safe_text.parse::<i64>().is_ok() || safe_text.parse::<f64>().is_ok() || safe_text.parse::<bool>().is_ok() {
            // Skip numeric and boolean strings as they get converted to their respective types
            return Ok(());
        }
        
        let result = engine.expand_string(&safe_text, &context).unwrap();
        
        prop_assert_eq!(result, json!(safe_text));
    }

    #[test]
    fn property_parameter_template_required_variables(
        template_name in "[a-z][a-z0-9_]{1,10}",
        required_var in "[a-z][a-z0-9_]{1,10}",
        var_value in "[a-zA-Z][a-zA-Z0-9 ]{1,20}" // Non-numeric strings only
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
        prop_assert_eq!(&result["value"], &json!(var_value));
    }

    #[test]
    fn property_template_default_values(
        template_name in "[a-z][a-z0-9_]{1,10}",
        var_name in "[a-z][a-z0-9_]{1,10}",
        default_value in "[a-zA-Z][a-zA-Z0-9 ]{1,20}" // Non-numeric strings only
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
        prop_assert_eq!(&result["value"], &json!(default_value));
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
        prop_assert_eq!(&result["level1"]["level2"]["boolean"], &json!(true));
        prop_assert_eq!(&result["level1"]["level2"]["null_value"], &json!(null));
        prop_assert_eq!(&result["simple_string"], &json!("no variables here"));
    }
}

#[test]
fn test_template_expansion_basic() {
    let engine = TemplateEngine::new().unwrap();
    let mut context = TemplateContext::new();
    context.set_variable("name", json!("world"));
    
    let result = engine.expand_string("Hello ${name}!", &context).unwrap();
    assert_eq!(result, json!("Hello world!"));
}

#[test]
fn test_parameter_template_basic() {
    let template = ParameterTemplate::new(
        "test".to_string(),
        json!({"greeting": "Hello ${name}!"})
    );
    
    let engine = TemplateEngine::new().unwrap();
    let mut context = TemplateContext::new();
    context.set_variable("name", json!("Alice"));
    
    let result = template.expand(&mut context, &engine).unwrap();
    assert_eq!(result["greeting"], json!("Hello Alice!"));
}

#[test]
fn test_template_numeric_conversion() {
    // Test that numeric strings are correctly converted to numbers
    let engine = TemplateEngine::new().unwrap();
    let mut context = TemplateContext::new();
    context.set_variable("number", json!("42"));
    
    // When we substitute a variable containing a numeric string, 
    // the result gets converted to a number by the template engine
    let result = engine.expand_string("${number}", &context).unwrap();
    assert_eq!(result, json!(42)); // Gets converted to number
    
    // Test direct numeric template - the engine converts numeric strings to numbers
    let result = engine.expand_string("42", &context).unwrap();
    assert_eq!(result, json!(42)); // Should be converted to number
    
    // Test that non-numeric strings remain as strings
    context.set_variable("text", json!("hello"));
    let result = engine.expand_string("${text}", &context).unwrap();
    assert_eq!(result, json!("hello")); // Should remain as string
}