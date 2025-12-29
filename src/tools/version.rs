//! Tool version management and dependency resolution

use crate::error::{Result, WorkflowError};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::{is_cyclic_directed, toposort};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::str::FromStr;
use tracing::{debug, warn};

/// Semantic version representation
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre_release: Option<String>,
    pub build: Option<String>,
}

impl Version {
    /// Create a new version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
            build: None,
        }
    }
    
    /// Create a version with pre-release identifier
    pub fn with_pre_release(mut self, pre_release: String) -> Self {
        self.pre_release = Some(pre_release);
        self
    }
    
    /// Create a version with build metadata
    pub fn with_build(mut self, build: String) -> Self {
        self.build = Some(build);
        self
    }
    
    /// Check if this version is compatible with another version
    pub fn is_compatible_with(&self, other: &Version) -> bool {
        // Major version must match for compatibility
        if self.major != other.major {
            return false;
        }
        
        // Minor version can be higher or equal
        if self.minor < other.minor {
            return false;
        }
        
        // If minor versions are equal, patch can be higher or equal
        if self.minor == other.minor && self.patch < other.patch {
            return false;
        }
        
        true
    }
    
    /// Check if this version satisfies a version requirement
    pub fn satisfies(&self, requirement: &VersionRequirement) -> bool {
        match requirement {
            VersionRequirement::Exact(version) => self == version,
            VersionRequirement::GreaterThan(version) => self > version,
            VersionRequirement::GreaterThanOrEqual(version) => self >= version,
            VersionRequirement::LessThan(version) => self < version,
            VersionRequirement::LessThanOrEqual(version) => self <= version,
            VersionRequirement::Compatible(version) => self.is_compatible_with(version),
            VersionRequirement::Range { min, max } => self >= min && self <= max,
            VersionRequirement::Any => true,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        
        if let Some(pre) = &self.pre_release {
            write!(f, "-{}", pre)?;
        }
        
        if let Some(build) = &self.build {
            write!(f, "+{}", build)?;
        }
        
        Ok(())
    }
}

impl FromStr for Version {
    type Err = WorkflowError;
    
    fn from_str(s: &str) -> Result<Self> {
        let mut parts = s.split('+');
        let version_part = parts.next().unwrap();
        let build = parts.next().map(|s| s.to_string());
        
        let mut parts = version_part.split('-');
        let version_numbers = parts.next().unwrap();
        let pre_release = parts.next().map(|s| s.to_string());
        
        let numbers: Vec<&str> = version_numbers.split('.').collect();
        if numbers.len() != 3 {
            return Err(WorkflowError::ValidationError(
                format!("Invalid version format: {}", s)
            ));
        }
        
        let major = numbers[0].parse::<u32>()
            .map_err(|_| WorkflowError::ValidationError(format!("Invalid major version: {}", numbers[0])))?;
        let minor = numbers[1].parse::<u32>()
            .map_err(|_| WorkflowError::ValidationError(format!("Invalid minor version: {}", numbers[1])))?;
        let patch = numbers[2].parse::<u32>()
            .map_err(|_| WorkflowError::ValidationError(format!("Invalid patch version: {}", numbers[2])))?;
        
        Ok(Version {
            major,
            minor,
            patch,
            pre_release,
            build,
        })
    }
}

/// Version requirement specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VersionRequirement {
    /// Exact version match
    Exact(Version),
    /// Greater than version
    GreaterThan(Version),
    /// Greater than or equal to version
    GreaterThanOrEqual(Version),
    /// Less than version
    LessThan(Version),
    /// Less than or equal to version
    LessThanOrEqual(Version),
    /// Compatible version (same major, minor >= required)
    Compatible(Version),
    /// Version range (inclusive)
    Range { min: Version, max: Version },
    /// Any version
    Any,
}

impl VersionRequirement {
    /// Parse a version requirement from string
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();
        
        if s == "*" || s.is_empty() {
            return Ok(VersionRequirement::Any);
        }
        
        if s.starts_with(">=") {
            let version = Version::from_str(&s[2..].trim())?;
            return Ok(VersionRequirement::GreaterThanOrEqual(version));
        }
        
        if s.starts_with("<=") {
            let version = Version::from_str(&s[2..].trim())?;
            return Ok(VersionRequirement::LessThanOrEqual(version));
        }
        
        if s.starts_with('>') {
            let version = Version::from_str(&s[1..].trim())?;
            return Ok(VersionRequirement::GreaterThan(version));
        }
        
        if s.starts_with('<') {
            let version = Version::from_str(&s[1..].trim())?;
            return Ok(VersionRequirement::LessThan(version));
        }
        
        if s.starts_with('~') {
            let version = Version::from_str(&s[1..].trim())?;
            return Ok(VersionRequirement::Compatible(version));
        }
        
        if s.starts_with('=') {
            let version = Version::from_str(&s[1..].trim())?;
            return Ok(VersionRequirement::Exact(version));
        }
        
        // Check for range format "1.0.0 - 2.0.0"
        if let Some(dash_pos) = s.find(" - ") {
            let min_str = s[..dash_pos].trim();
            let max_str = s[dash_pos + 3..].trim();
            let min = Version::from_str(min_str)?;
            let max = Version::from_str(max_str)?;
            return Ok(VersionRequirement::Range { min, max });
        }
        
        // Default to exact match
        let version = Version::from_str(s)?;
        Ok(VersionRequirement::Exact(version))
    }
}

impl fmt::Display for VersionRequirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VersionRequirement::Exact(v) => write!(f, "={}", v),
            VersionRequirement::GreaterThan(v) => write!(f, ">{}", v),
            VersionRequirement::GreaterThanOrEqual(v) => write!(f, ">={}", v),
            VersionRequirement::LessThan(v) => write!(f, "<{}", v),
            VersionRequirement::LessThanOrEqual(v) => write!(f, "<={}", v),
            VersionRequirement::Compatible(v) => write!(f, "~{}", v),
            VersionRequirement::Range { min, max } => write!(f, "{} - {}", min, max),
            VersionRequirement::Any => write!(f, "*"),
        }
    }
}

/// Tool dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDependency {
    pub name: String,
    pub version_requirement: VersionRequirement,
    pub optional: bool,
}

impl ToolDependency {
    /// Create a new tool dependency
    pub fn new(name: String, version_requirement: VersionRequirement) -> Self {
        Self {
            name,
            version_requirement,
            optional: false,
        }
    }
    
    /// Create an optional dependency
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
    
    /// Check if a tool version satisfies this dependency
    pub fn is_satisfied_by(&self, version: &Version) -> bool {
        version.satisfies(&self.version_requirement)
    }
}

/// Tool version information with dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolVersion {
    pub name: String,
    pub version: Version,
    pub dependencies: Vec<ToolDependency>,
    pub conflicts: Vec<String>, // Tool names that conflict with this version
}

impl ToolVersion {
    /// Create a new tool version
    pub fn new(name: String, version: Version) -> Self {
        Self {
            name,
            version,
            dependencies: Vec::new(),
            conflicts: Vec::new(),
        }
    }
    
    /// Add a dependency
    pub fn with_dependency(mut self, dependency: ToolDependency) -> Self {
        self.dependencies.push(dependency);
        self
    }
    
    /// Add a conflict
    pub fn with_conflict(mut self, conflict: String) -> Self {
        self.conflicts.push(conflict);
        self
    }
}

/// Version conflict information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionConflict {
    pub tool_name: String,
    pub required_versions: Vec<VersionRequirement>,
    pub available_versions: Vec<Version>,
    pub conflict_type: ConflictType,
}

/// Type of version conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    /// No version satisfies all requirements
    NoSatisfyingVersion,
    /// Multiple incompatible versions required
    IncompatibleVersions,
    /// Circular dependency detected
    CircularDependency,
    /// Tool conflicts with another tool
    ToolConflict { conflicting_tool: String },
}

/// Dependency resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionResult {
    pub resolved_versions: HashMap<String, Version>,
    pub conflicts: Vec<VersionConflict>,
    pub warnings: Vec<String>,
}

impl ResolutionResult {
    /// Check if resolution was successful (no conflicts)
    pub fn is_successful(&self) -> bool {
        self.conflicts.is_empty()
    }
    
    /// Get the resolved version for a tool
    pub fn get_version(&self, tool_name: &str) -> Option<&Version> {
        self.resolved_versions.get(tool_name)
    }
}

/// Dependency resolver for tool versions
pub struct DependencyResolver {
    available_versions: HashMap<String, Vec<ToolVersion>>,
}

impl DependencyResolver {
    /// Create a new dependency resolver
    pub fn new() -> Self {
        Self {
            available_versions: HashMap::new(),
        }
    }
    
    /// Add available tool versions
    pub fn add_tool_version(&mut self, tool_version: ToolVersion) {
        let name = tool_version.name.clone();
        self.available_versions
            .entry(name)
            .or_insert_with(Vec::new)
            .push(tool_version);
    }
    
    /// Add multiple tool versions
    pub fn add_tool_versions(&mut self, tool_versions: Vec<ToolVersion>) {
        for tool_version in tool_versions {
            self.add_tool_version(tool_version);
        }
    }
    
    /// Resolve dependencies for a set of required tools
    pub fn resolve_dependencies(
        &self,
        requirements: Vec<ToolDependency>,
    ) -> Result<ResolutionResult> {
        debug!("Resolving dependencies for {} tools", requirements.len());
        
        let mut resolved_versions = HashMap::new();
        let mut conflicts = Vec::new();
        let mut warnings = Vec::new();
        
        // Build dependency graph
        let mut graph = DiGraph::new();
        let mut node_map = HashMap::new();
        let mut tool_nodes = HashMap::new();
        
        // Add all required tools as nodes
        for requirement in &requirements {
            if !node_map.contains_key(&requirement.name) {
                let node_idx = graph.add_node(requirement.name.clone());
                node_map.insert(requirement.name.clone(), node_idx);
                tool_nodes.insert(node_idx, requirement.name.clone());
            }
        }
        
        // Collect all dependencies recursively
        let mut to_process = requirements.clone();
        let mut processed = HashSet::new();
        
        while let Some(requirement) = to_process.pop() {
            if processed.contains(&requirement.name) {
                continue;
            }
            processed.insert(requirement.name.clone());
            
            // Find a suitable version for this requirement
            if let Some(versions) = self.available_versions.get(&requirement.name) {
                let mut suitable_version = None;
                
                // Find the highest version that satisfies the requirement
                for version in versions.iter().rev() { // Assume versions are sorted
                    if requirement.is_satisfied_by(&version.version) {
                        suitable_version = Some(version);
                        break;
                    }
                }
                
                if let Some(version) = suitable_version {
                    resolved_versions.insert(requirement.name.clone(), version.version.clone());
                    
                    // Add dependencies to processing queue
                    for dep in &version.dependencies {
                        if !processed.contains(&dep.name) {
                            to_process.push(dep.clone());
                            
                            // Add dependency edge to graph
                            if !node_map.contains_key(&dep.name) {
                                let node_idx = graph.add_node(dep.name.clone());
                                node_map.insert(dep.name.clone(), node_idx);
                                tool_nodes.insert(node_idx, dep.name.clone());
                            }
                            
                            let from_node = node_map[&requirement.name];
                            let to_node = node_map[&dep.name];
                            graph.add_edge(from_node, to_node, ());
                        }
                    }
                    
                    // Check for conflicts
                    for conflict in &version.conflicts {
                        if resolved_versions.contains_key(conflict) {
                            conflicts.push(VersionConflict {
                                tool_name: requirement.name.clone(),
                                required_versions: vec![requirement.version_requirement.clone()],
                                available_versions: vec![version.version.clone()],
                                conflict_type: ConflictType::ToolConflict {
                                    conflicting_tool: conflict.clone(),
                                },
                            });
                        }
                    }
                } else {
                    // No suitable version found
                    let available_versions = versions.iter()
                        .map(|v| v.version.clone())
                        .collect();
                    
                    conflicts.push(VersionConflict {
                        tool_name: requirement.name.clone(),
                        required_versions: vec![requirement.version_requirement.clone()],
                        available_versions,
                        conflict_type: ConflictType::NoSatisfyingVersion,
                    });
                }
            } else if !requirement.optional {
                // Tool not available and not optional
                conflicts.push(VersionConflict {
                    tool_name: requirement.name.clone(),
                    required_versions: vec![requirement.version_requirement.clone()],
                    available_versions: Vec::new(),
                    conflict_type: ConflictType::NoSatisfyingVersion,
                });
            } else {
                warnings.push(format!("Optional tool '{}' not available", requirement.name));
            }
        }
        
        // Check for circular dependencies
        if is_cyclic_directed(&graph) {
            // Find the cycle (simplified approach)
            conflicts.push(VersionConflict {
                tool_name: "multiple".to_string(),
                required_versions: Vec::new(),
                available_versions: Vec::new(),
                conflict_type: ConflictType::CircularDependency,
            });
        }
        
        // Validate version compatibility
        self.validate_version_compatibility(&resolved_versions, &mut conflicts, &mut warnings)?;
        
        Ok(ResolutionResult {
            resolved_versions,
            conflicts,
            warnings,
        })
    }
    
    /// Validate that all resolved versions are compatible with each other
    fn validate_version_compatibility(
        &self,
        resolved_versions: &HashMap<String, Version>,
        conflicts: &mut Vec<VersionConflict>,
        warnings: &mut Vec<String>,
    ) -> Result<()> {
        for (tool_name, version) in resolved_versions {
            if let Some(tool_versions) = self.available_versions.get(tool_name) {
                if let Some(tool_version) = tool_versions.iter()
                    .find(|tv| tv.version == *version) {
                    
                    // Check if all dependencies are satisfied
                    for dep in &tool_version.dependencies {
                        if let Some(dep_version) = resolved_versions.get(&dep.name) {
                            if !dep.is_satisfied_by(dep_version) {
                                conflicts.push(VersionConflict {
                                    tool_name: tool_name.clone(),
                                    required_versions: vec![dep.version_requirement.clone()],
                                    available_versions: vec![dep_version.clone()],
                                    conflict_type: ConflictType::IncompatibleVersions,
                                });
                            }
                        } else if !dep.optional {
                            warnings.push(format!(
                                "Dependency '{}' for tool '{}' not resolved",
                                dep.name, tool_name
                            ));
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get available versions for a tool
    pub fn get_available_versions(&self, tool_name: &str) -> Option<&Vec<ToolVersion>> {
        self.available_versions.get(tool_name)
    }
    
    /// Get the latest version of a tool
    pub fn get_latest_version(&self, tool_name: &str) -> Option<&Version> {
        self.available_versions.get(tool_name)
            .and_then(|versions| versions.last())
            .map(|tv| &tv.version)
    }
    
    /// Check if a tool version is available
    pub fn has_version(&self, tool_name: &str, version: &Version) -> bool {
        self.available_versions.get(tool_name)
            .map_or(false, |versions| {
                versions.iter().any(|tv| tv.version == *version)
            })
    }
}

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_parsing() {
        let version = Version::from_str("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.pre_release, None);
        assert_eq!(version.build, None);
        
        let version = Version::from_str("1.2.3-alpha.1+build.123").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.pre_release, Some("alpha.1".to_string()));
        assert_eq!(version.build, Some("build.123".to_string()));
    }
    
    #[test]
    fn test_version_compatibility() {
        let v1_0_0 = Version::new(1, 0, 0);
        let v1_1_0 = Version::new(1, 1, 0);
        let v1_1_1 = Version::new(1, 1, 1);
        let v2_0_0 = Version::new(2, 0, 0);
        
        assert!(v1_1_0.is_compatible_with(&v1_0_0));
        assert!(v1_1_1.is_compatible_with(&v1_1_0));
        assert!(!v1_0_0.is_compatible_with(&v1_1_0));
        assert!(!v2_0_0.is_compatible_with(&v1_0_0));
    }
    
    #[test]
    fn test_version_requirements() {
        let v1_0_0 = Version::new(1, 0, 0);
        let v1_1_0 = Version::new(1, 1, 0);
        let v2_0_0 = Version::new(2, 0, 0);
        
        let req = VersionRequirement::GreaterThanOrEqual(v1_0_0.clone());
        assert!(v1_0_0.satisfies(&req));
        assert!(v1_1_0.satisfies(&req));
        assert!(v2_0_0.satisfies(&req));
        
        let req = VersionRequirement::Compatible(v1_0_0.clone());
        assert!(v1_0_0.satisfies(&req));
        assert!(v1_1_0.satisfies(&req));
        assert!(!v2_0_0.satisfies(&req));
    }
    
    #[test]
    fn test_dependency_resolution() {
        let mut resolver = DependencyResolver::new();
        
        // Add tool versions
        let tool_a_v1 = ToolVersion::new("tool_a".to_string(), Version::new(1, 0, 0))
            .with_dependency(ToolDependency::new(
                "tool_b".to_string(),
                VersionRequirement::GreaterThanOrEqual(Version::new(1, 0, 0)),
            ));
        
        let tool_b_v1 = ToolVersion::new("tool_b".to_string(), Version::new(1, 0, 0));
        let tool_b_v2 = ToolVersion::new("tool_b".to_string(), Version::new(2, 0, 0));
        
        resolver.add_tool_version(tool_a_v1);
        resolver.add_tool_version(tool_b_v1);
        resolver.add_tool_version(tool_b_v2);
        
        // Resolve dependencies
        let requirements = vec![
            ToolDependency::new(
                "tool_a".to_string(),
                VersionRequirement::Any,
            ),
        ];
        
        let result = resolver.resolve_dependencies(requirements).unwrap();
        assert!(result.is_successful());
        assert_eq!(result.resolved_versions.len(), 2);
        assert!(result.resolved_versions.contains_key("tool_a"));
        assert!(result.resolved_versions.contains_key("tool_b"));
    }
}