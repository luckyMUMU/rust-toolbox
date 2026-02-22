//! Proc-macro crate for workflow-toolkit
//!
//! This crate provides derive macros for the workflow-toolkit:
//! - `#[derive(ToolInput)]` - Derive macro for tool input types
//! - `#[derive(ToolOutput)]` - Derive macro for tool output types (optional)
//!
//! # Example
//!
//! ```rust
//! use workflow_toolkit_macros::ToolInput;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(ToolInput, Serialize, Deserialize, Debug)]
//! pub struct EchoInput {
//!     #[tool_input(description = "Message to echo", required = true)]
//!     pub message: String,
//!     
//!     #[tool_input(description = "Number of times to echo", default = 1)]
//!     pub count: u32,
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Lit, Token};

/// Derive macro for ToolInput trait
///
/// This macro generates:
/// 1. Implementation of `Into<ToolInput>` for the struct
/// 2. Validation logic based on field attributes
/// 3. Schema generation for the input type
///
/// # Attributes
///
/// - `#[tool_input(description = "...")]` - Field description
/// - `#[tool_input(required = true)]` - Mark field as required
/// - `#[tool_input(default = ...)]` - Default value for field
/// - `#[tool_input(validate = "...")]` - Validation rule
#[proc_macro_derive(ToolInput, attributes(tool_input))]
pub fn derive_tool_input(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Extract field information
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("ToolInput only supports named fields"),
        },
        _ => panic!("ToolInput only supports structs"),
    };

    // Generate field metadata
    let field_metadata: Vec<_> = fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            let field_type = &field.ty;

            // Parse tool_input attributes
            let mut description = String::new();
            let mut required = false;
            let mut default_value = None;
            let mut validation = None;

            for attr in &field.attrs {
                if attr.path().is_ident("tool_input") {
                    // Parse the attribute content
                    attr.parse_args_with(|input: syn::parse::ParseStream| {
                        let mut first = true;
                        while !input.is_empty() {
                            if !first {
                                input.parse::<Token![,]>()?;
                            }
                            first = false;

                            let key: syn::Ident = input.parse()?;
                            input.parse::<Token![=]>()?;

                            match key.to_string().as_str() {
                                "description" => {
                                    let lit: Lit = input.parse()?;
                                    if let Lit::Str(s) = lit {
                                        description = s.value();
                                    }
                                }
                                "required" => {
                                    let lit: Lit = input.parse()?;
                                    if let Lit::Bool(b) = lit {
                                        required = b.value();
                                    }
                                }
                                "default" => {
                                    let lit: Lit = input.parse()?;
                                    default_value = Some(quote!(#lit));
                                }
                                "validate" => {
                                    let lit: Lit = input.parse()?;
                                    if let Lit::Str(s) = lit {
                                        validation = Some(s.value());
                                    }
                                }
                                _ => {
                                    // Skip unknown attributes
                                    let _: syn::Expr = input.parse()?;
                                }
                            }
                        }
                        Ok(())
                    })
                    .ok();
                }
            }

            (
                field_name,
                field_type,
                description,
                required,
                default_value,
                validation,
            )
        })
        .collect();

    // Generate validation code
    let validations: Vec<_> = field_metadata
        .iter()
        .map(|(field_name, _ty, _desc, required, _default, validation)| {
            let field_str = field_name.to_string();
            let mut checks = Vec::new();

            if *required {
                checks.push(quote! {
                    if self.#field_name.is_empty() {
                        return Err(workflow_toolkit::WorkflowError::ValidationError(
                            format!("Field '{}' is required", #field_str)
                        ));
                    }
                });
            }

            if let Some(_rule) = validation {
                // TODO: Implement validation rules
                checks.push(quote! {
                    // Validation logic here
                });
            }

            quote! { #(#checks)* }
        })
        .collect();

    // Generate schema code
    let schema_fields: Vec<_> = field_metadata
        .iter()
        .map(|(field_name, _ty, desc, required, _default, _validation)| {
            let field_str = field_name.to_string();
            quote! {
                schema.properties.insert(
                    #field_str.to_string(),
                    serde_json::json!({
                        "description": #desc,
                        "required": #required,
                    })
                );
            }
        })
        .collect();

    // Generate the implementation
    let expanded = quote! {
        impl #impl_generics workflow_toolkit::tools::ToolInputConvert for #struct_name #ty_generics #where_clause {
            fn into_tool_input(self) -> workflow_toolkit::tools::ToolInput {
                workflow_toolkit::tools::ToolInput::new(
                    serde_json::to_value(&self).unwrap_or_default()
                )
            }

            fn from_tool_input(input: &workflow_toolkit::tools::ToolInput) -> Result<Self, workflow_toolkit::WorkflowError> {
                serde_json::from_value(input.params.clone())
                    .map_err(|e| workflow_toolkit::WorkflowError::ValidationError(
                        format!("Failed to parse input: {}", e)
                    ))
            }

            fn validate(&self) -> Result<(), workflow_toolkit::WorkflowError> {
                #(#validations)*
                Ok(())
            }

            fn schema() -> workflow_toolkit::tools::InputSchema {
                let mut schema = workflow_toolkit::tools::InputSchema::default();
                schema.type_name = stringify!(#struct_name).to_string();
                #(#schema_fields)*
                schema
            }
        }

        impl #impl_generics std::convert::TryFrom<workflow_toolkit::tools::ToolInput> for #struct_name #ty_generics #where_clause {
            type Error = workflow_toolkit::WorkflowError;

            fn try_from(input: workflow_toolkit::tools::ToolInput) -> Result<Self, Self::Error> {
                Self::from_tool_input(&input)
            }
        }

        impl #impl_generics std::convert::From<#struct_name> for workflow_toolkit::tools::ToolInput #ty_generics #where_clause {
            fn from(val: #struct_name) -> Self {
                val.into_tool_input()
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for ToolOutput trait (optional)
#[proc_macro_derive(ToolOutput, attributes(tool_output))]
pub fn derive_tool_output(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics workflow_toolkit::tools::ToolOutputConvert for #struct_name #ty_generics #where_clause {
            fn into_tool_output(self) -> workflow_toolkit::tools::ToolOutput {
                workflow_toolkit::tools::ToolOutput::success(
                    serde_json::to_value(&self).unwrap_or_default()
                )
            }

            fn from_tool_output(output: &workflow_toolkit::tools::ToolOutput) -> Result<Self, workflow_toolkit::WorkflowError> {
                serde_json::from_value(output.result.clone())
                    .map_err(|e| workflow_toolkit::WorkflowError::ValidationError(
                        format!("Failed to parse output: {}", e)
                    ))
            }
        }

        impl #impl_generics std::convert::TryFrom<workflow_toolkit::tools::ToolOutput> for #struct_name #ty_generics #where_clause {
            type Error = workflow_toolkit::WorkflowError;

            fn try_from(output: workflow_toolkit::tools::ToolOutput) -> Result<Self, Self::Error> {
                Self::from_tool_output(&output)
            }
        }
    };

    TokenStream::from(expanded)
}
