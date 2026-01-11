//! Integration tests for tool version dependency resolution
//! **Feature: workflow-toolkit, Property 14: 工具版本依赖解析**
//! **Validates: Requirements 6.4**

use proptest::prelude::*;
use std::str::FromStr;
use workflow_toolkit::tools::version::*;

// Generator for valid versions
fn arb_version() -> impl Strategy<Value = Version> {
    (0u32..100, 0u32..100, 0u32..100)
        .prop_map(|(major, minor, patch)| Version::new(major, minor, patch))
}

// Generator for tool names
fn arb_tool_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_-]{2,15}".prop_map(|s| s.to_string())
}

// Generator for version requirements
fn arb_version_requirement() -> impl Strategy<Value = VersionRequirement> {
    prop_oneof![
        arb_version().prop_map(VersionRequirement::Exact),
        arb_version().prop_map(VersionRequirement::GreaterThanOrEqual),
        arb_version().prop_map(VersionRequirement::Compatible),
        Just(VersionRequirement::Any),
    ]
}

// Generator for tool dependencies
fn arb_tool_dependency() -> impl Strategy<Value = ToolDependency> {
    (arb_tool_name(), arb_version_requirement(), any::<bool>()).prop_map(|(name, req, optional)| {
        let mut dep = ToolDependency::new(name, req);
        if optional {
            dep = dep.optional();
        }
        dep
    })
}

// Generator for tool versions with dependencies
fn arb_tool_version() -> impl Strategy<Value = ToolVersion> {
    (
        arb_tool_name(),
        arb_version(),
        prop::collection::vec(arb_tool_dependency(), 0..3), // Reduced complexity
    )
        .prop_map(|(name, version, deps)| {
            let mut tool_version = ToolVersion::new(name, version);
            for dep in deps {
                tool_version = tool_version.with_dependency(dep);
            }
            tool_version
        })
}

proptest! {
    #[test]
    fn property_version_parsing_roundtrip(
        major in 0u32..10,
        minor in 0u32..10,
        patch in 0u32..10
    ) {
        // **Feature: workflow-toolkit, Property 14: 工具版本依赖解析**
        // For any valid version, parsing then formatting should be consistent
        let version = Version::new(major, minor, patch);
        let version_str = version.to_string();
        let parsed_version = Version::from_str(&version_str).unwrap();

        prop_assert_eq!(version, parsed_version);
    }

    #[test]
    fn property_version_requirement_consistency(
        version in arb_version(),
        requirement in arb_version_requirement()
    ) {
        // **Feature: workflow-toolkit, Property 14: 工具版本依赖解析**
        // For any version and requirement, satisfies should be deterministic
        let satisfies1 = version.satisfies(&requirement);
        let satisfies2 = version.satisfies(&requirement);

        prop_assert_eq!(satisfies1, satisfies2);
    }

    #[test]
    fn property_dependency_resolution_deterministic(
        tool_versions in prop::collection::vec(arb_tool_version(), 1..5),
        requirements in prop::collection::vec(arb_tool_dependency(), 1..3)
    ) {
        // **Feature: workflow-toolkit, Property 14: 工具版本依赖解析**
        // For any set of tool versions and requirements, dependency resolution should be deterministic
        let mut resolver1 = DependencyResolver::new();
        let mut resolver2 = DependencyResolver::new();

        // Add the same tool versions to both resolvers
        for tool_version in &tool_versions {
            resolver1.add_tool_version(tool_version.clone());
            resolver2.add_tool_version(tool_version.clone());
        }

        // Resolve the same requirements
        let result1 = resolver1.resolve_dependencies(requirements.clone());
        let result2 = resolver2.resolve_dependencies(requirements);

        // Results should be identical
        match (result1, result2) {
            (Ok(res1), Ok(res2)) => {
                prop_assert_eq!(res1.resolved_versions, res2.resolved_versions);
                prop_assert_eq!(res1.conflicts.len(), res2.conflicts.len());
            }
            (Err(_), Err(_)) => {
                // Both failed, which is also consistent
            }
            _ => {
                prop_assert!(false, "Inconsistent resolution results");
            }
        }
    }

    #[test]
    fn property_successful_resolution_has_no_conflicts(
        tool_versions in prop::collection::vec(arb_tool_version(), 1..3),
        requirements in prop::collection::vec(arb_tool_dependency(), 1..2)
    ) {
        // **Feature: workflow-toolkit, Property 14: 工具版本依赖解析**
        // For any successful dependency resolution, there should be no conflicts
        let mut resolver = DependencyResolver::new();

        for tool_version in tool_versions {
            resolver.add_tool_version(tool_version);
        }

        if let Ok(result) = resolver.resolve_dependencies(requirements) {
            if result.is_successful() {
                prop_assert!(result.conflicts.is_empty());
            }
        }
    }
}

#[test]
fn test_version_parsing_basic() {
    let version = Version::from_str("1.2.3").unwrap();
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 3);
}

#[test]
fn test_dependency_resolution_basic() {
    let mut resolver = DependencyResolver::new();

    let tool_a = ToolVersion::new("tool_a".to_string(), Version::new(1, 0, 0));
    resolver.add_tool_version(tool_a);

    let requirements = vec![ToolDependency::new(
        "tool_a".to_string(),
        VersionRequirement::Any,
    )];

    let result = resolver.resolve_dependencies(requirements).unwrap();
    assert!(result.is_successful());
    assert!(result.resolved_versions.contains_key("tool_a"));
}
