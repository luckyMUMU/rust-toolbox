//! Aho-Corasick Automaton Implementation
//!
//! This module provides an efficient implementation of the Aho-Corasick algorithm
//! for multi-pattern string matching. It supports both case-sensitive and
//! case-insensitive matching with Unicode text support.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use tracing::{debug, trace};

/// A pattern entry in the automaton
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pattern {
    /// The pattern string to match
    pub pattern: String,
    /// Category or classification for this pattern
    pub category: String,
    /// Score or weight for this pattern
    pub score: f64,
    /// Pattern ID for internal tracking
    pub id: usize,
}

impl Pattern {
    /// Create a new pattern
    pub fn new<S1, S2>(pattern: S1, category: S2, score: f64, id: usize) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            pattern: pattern.into(),
            category: category.into(),
            score,
            id,
        }
    }

    /// Get the pattern length in characters (not bytes)
    pub fn char_len(&self) -> usize {
        self.pattern.chars().count()
    }

    /// Convert pattern to lowercase for case-insensitive matching
    pub fn to_lowercase(&self) -> String {
        self.pattern.to_lowercase()
    }
}

/// A node in the Aho-Corasick automaton trie
#[derive(Debug, Clone)]
pub struct AutomatonNode {
    /// Node ID for internal tracking
    pub id: usize,
    /// Children nodes indexed by character
    pub children: HashMap<char, usize>,
    /// Failure link for efficient pattern matching
    pub failure_link: Option<usize>,
    /// Output patterns that end at this node
    pub output_patterns: Vec<Pattern>,
    /// Depth of this node in the trie (root is 0)
    pub depth: usize,
    /// Parent node ID (None for root)
    pub parent: Option<usize>,
    /// Character that led to this node from parent
    pub parent_char: Option<char>,
}

impl AutomatonNode {
    /// Create a new automaton node
    pub fn new(id: usize, depth: usize, parent: Option<usize>, parent_char: Option<char>) -> Self {
        Self {
            id,
            children: HashMap::new(),
            failure_link: None,
            output_patterns: Vec::new(),
            depth,
            parent,
            parent_char,
        }
    }

    /// Create the root node
    pub fn root() -> Self {
        Self::new(0, 0, None, None)
    }

    /// Check if this node has any output patterns
    pub fn has_output(&self) -> bool {
        !self.output_patterns.is_empty()
    }

    /// Add an output pattern to this node
    pub fn add_output_pattern(&mut self, pattern: Pattern) {
        self.output_patterns.push(pattern);
    }

    /// Get all output patterns at this node
    pub fn get_output_patterns(&self) -> &[Pattern] {
        &self.output_patterns
    }

    /// Check if this node has a child for the given character
    pub fn has_child(&self, ch: char) -> bool {
        self.children.contains_key(&ch)
    }

    /// Get child node ID for the given character
    pub fn get_child(&self, ch: char) -> Option<usize> {
        self.children.get(&ch).copied()
    }

    /// Add a child node for the given character
    pub fn add_child(&mut self, ch: char, child_id: usize) {
        self.children.insert(ch, child_id);
    }

    /// Get all child characters
    pub fn child_chars(&self) -> impl Iterator<Item = &char> {
        self.children.keys()
    }

    /// Set the failure link for this node
    pub fn set_failure_link(&mut self, target_id: usize) {
        self.failure_link = Some(target_id);
    }

    /// Get the failure link target
    pub fn get_failure_link(&self) -> Option<usize> {
        self.failure_link
    }
}

impl fmt::Display for AutomatonNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Node(id={}, depth={}, children={}, outputs={}, failure={})",
            self.id,
            self.depth,
            self.children.len(),
            self.output_patterns.len(),
            self.failure_link
                .map_or("None".to_string(), |id| id.to_string())
        )
    }
}

/// A match result from the automaton
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternMatch {
    /// The matched pattern
    pub pattern: String,
    /// Category of the matched pattern
    pub category: String,
    /// Score of the matched pattern
    pub score: f64,
    /// Pattern ID
    pub pattern_id: usize,
    /// Start position in the text (character index)
    pub start_pos: usize,
    /// End position in the text (character index, exclusive)
    pub end_pos: usize,
}

impl PatternMatch {
    /// Create a new pattern match
    pub fn new(pattern: Pattern, start_pos: usize, end_pos: usize) -> Self {
        Self {
            pattern: pattern.pattern.clone(),
            category: pattern.category.clone(),
            score: pattern.score,
            pattern_id: pattern.id,
            start_pos,
            end_pos,
        }
    }

    /// Get the length of the match in characters
    pub fn match_len(&self) -> usize {
        self.end_pos - self.start_pos
    }

    /// Check if this match overlaps with another match
    pub fn overlaps_with(&self, other: &PatternMatch) -> bool {
        !(self.end_pos <= other.start_pos || other.end_pos <= self.start_pos)
    }
}

impl fmt::Display for PatternMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Match('{}' [{}] at {}..{}, score={})",
            self.pattern, self.category, self.start_pos, self.end_pos, self.score
        )
    }
}

/// Configuration for the Aho-Corasick automaton
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomatonConfig {
    /// Whether to perform case-sensitive matching
    pub case_sensitive: bool,
    /// Whether to find overlapping matches
    pub find_overlapping: bool,
    /// Maximum number of patterns to support
    pub max_patterns: usize,
    /// Maximum pattern length to support
    pub max_pattern_length: usize,
}

impl Default for AutomatonConfig {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            find_overlapping: false,
            max_patterns: 10_000,
            max_pattern_length: 1000,
        }
    }
}

/// Statistics about the automaton
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomatonStats {
    /// Number of nodes in the automaton
    pub node_count: usize,
    /// Number of patterns stored
    pub pattern_count: usize,
    /// Maximum depth of the trie
    pub max_depth: usize,
    /// Total number of edges (transitions)
    pub edge_count: usize,
    /// Memory usage estimate in bytes
    pub estimated_memory_bytes: usize,
}

impl AutomatonStats {
    /// Create empty stats
    pub fn new() -> Self {
        Self {
            node_count: 0,
            pattern_count: 0,
            max_depth: 0,
            edge_count: 0,
            estimated_memory_bytes: 0,
        }
    }

    /// Calculate estimated memory usage
    pub fn calculate_memory_estimate(&mut self, nodes: &[AutomatonNode], patterns: &[Pattern]) {
        // Rough estimate of memory usage
        let node_size = std::mem::size_of::<AutomatonNode>();
        let pattern_size = std::mem::size_of::<Pattern>();

        // Add size of node data structures
        let nodes_memory = self.node_count * node_size;

        // Add size of pattern data
        let patterns_memory = patterns
            .iter()
            .map(|p| pattern_size + p.pattern.len() + p.category.len())
            .sum::<usize>();

        // Add size of hash maps (rough estimate)
        let hashmap_memory = nodes
            .iter()
            .map(|n| {
                n.children.len() * (std::mem::size_of::<char>() + std::mem::size_of::<usize>())
            })
            .sum::<usize>();

        self.estimated_memory_bytes = nodes_memory + patterns_memory + hashmap_memory;
    }
}

impl Default for AutomatonStats {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for AutomatonStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AutomatonStats(nodes={}, patterns={}, max_depth={}, edges={}, memory={}KB)",
            self.node_count,
            self.pattern_count,
            self.max_depth,
            self.edge_count,
            self.estimated_memory_bytes / 1024
        )
    }
}

/// Error types for the Aho-Corasick automaton
#[derive(Debug, thiserror::Error)]
pub enum AutomatonError {
    #[error("Pattern too long: {length} characters (max: {max_length})")]
    PatternTooLong { length: usize, max_length: usize },

    #[error("Too many patterns: {count} (max: {max_patterns})")]
    TooManyPatterns { count: usize, max_patterns: usize },

    #[error("Empty pattern not allowed")]
    EmptyPattern,

    #[error("Invalid node ID: {node_id}")]
    InvalidNodeId { node_id: usize },

    #[error("Automaton not built yet")]
    NotBuilt,

    #[error("Invalid character in pattern: {ch:?}")]
    InvalidCharacter { ch: char },

    #[error("Pattern already exists: {pattern}")]
    DuplicatePattern { pattern: String },
}

pub type AutomatonResult<T> = Result<T, AutomatonError>;

/// The main Aho-Corasick automaton structure
#[derive(Debug, Clone)]
pub struct AhoCorasickMatcher {
    /// All nodes in the automaton
    nodes: Vec<AutomatonNode>,
    /// All patterns stored in the automaton
    patterns: Vec<Pattern>,
    /// Configuration for the automaton
    config: AutomatonConfig,
    /// Whether the automaton has been built (failure links computed)
    is_built: bool,
    /// Statistics about the automaton
    stats: AutomatonStats,
    /// Next available node ID
    next_node_id: usize,
    /// Next available pattern ID
    next_pattern_id: usize,
}

impl AhoCorasickMatcher {
    /// Create a new Aho-Corasick matcher with default configuration
    pub fn new() -> Self {
        Self::with_config(AutomatonConfig::default())
    }

    /// Create a new Aho-Corasick matcher with custom configuration
    pub fn with_config(config: AutomatonConfig) -> Self {
        let mut nodes = Vec::new();
        nodes.push(AutomatonNode::root()); // Root node at index 0

        Self {
            nodes,
            patterns: Vec::new(),
            config,
            is_built: false,
            stats: AutomatonStats::new(),
            next_node_id: 1, // Root is 0, next is 1
            next_pattern_id: 0,
        }
    }

    /// Add a pattern to the automaton
    pub fn add_pattern<S1, S2>(
        &mut self,
        pattern: S1,
        category: S2,
        score: f64,
    ) -> AutomatonResult<usize>
    where
        S1: AsRef<str>,
        S2: Into<String>,
    {
        let pattern_str = pattern.as_ref();

        // Validate pattern
        if pattern_str.is_empty() {
            return Err(AutomatonError::EmptyPattern);
        }

        let char_count = pattern_str.chars().count();
        if char_count > self.config.max_pattern_length {
            return Err(AutomatonError::PatternTooLong {
                length: char_count,
                max_length: self.config.max_pattern_length,
            });
        }

        if self.patterns.len() >= self.config.max_patterns {
            return Err(AutomatonError::TooManyPatterns {
                count: self.patterns.len() + 1,
                max_patterns: self.config.max_patterns,
            });
        }

        // Check for duplicate patterns
        let pattern_to_check = if self.config.case_sensitive {
            pattern_str.to_string()
        } else {
            pattern_str.to_lowercase()
        };

        if self.patterns.iter().any(|p| {
            let existing_pattern = if self.config.case_sensitive {
                p.pattern.clone()
            } else {
                p.pattern.to_lowercase()
            };
            existing_pattern == pattern_to_check
        }) {
            return Err(AutomatonError::DuplicatePattern {
                pattern: pattern_str.to_string(),
            });
        }

        // Create the pattern
        let pattern_id = self.next_pattern_id;
        let pattern_obj = Pattern::new(pattern_str, category, score, pattern_id);

        // Insert pattern into trie
        self.insert_pattern_into_trie(&pattern_obj)?;

        // Store the pattern
        self.patterns.push(pattern_obj);
        self.next_pattern_id += 1;

        // Mark as not built since we added a new pattern
        self.is_built = false;

        debug!("Added pattern '{}' with ID {}", pattern_str, pattern_id);
        Ok(pattern_id)
    }

    /// Insert a pattern into the trie structure
    fn insert_pattern_into_trie(&mut self, pattern: &Pattern) -> AutomatonResult<()> {
        let pattern_chars: Vec<char> = if self.config.case_sensitive {
            pattern.pattern.chars().collect()
        } else {
            pattern.to_lowercase().chars().collect()
        };

        let mut current_node_id = 0; // Start at root

        // Traverse/create path for each character in the pattern
        for (depth, &ch) in pattern_chars.iter().enumerate() {
            let current_node = &self.nodes[current_node_id];

            if let Some(child_id) = current_node.get_child(ch) {
                // Child exists, move to it
                current_node_id = child_id;
            } else {
                // Create new child node
                let new_node_id = self.next_node_id;
                let new_node =
                    AutomatonNode::new(new_node_id, depth + 1, Some(current_node_id), Some(ch));

                self.nodes.push(new_node);

                // Add child reference to parent
                self.nodes[current_node_id].add_child(ch, new_node_id);

                current_node_id = new_node_id;
                self.next_node_id += 1;

                trace!(
                    "Created new node {} for character '{}' at depth {}",
                    new_node_id,
                    ch,
                    depth + 1
                );
            }
        }

        // Add pattern as output at the final node
        self.nodes[current_node_id].add_output_pattern(pattern.clone());

        trace!(
            "Pattern '{}' ends at node {}",
            pattern.pattern,
            current_node_id
        );
        Ok(())
    }

    /// Build the automaton by constructing failure links
    /// This must be called after adding all patterns and before matching
    pub fn build(&mut self) -> AutomatonResult<()> {
        if self.is_built {
            debug!("Automaton already built, skipping");
            return Ok(());
        }

        if self.patterns.is_empty() {
            debug!("No patterns to build automaton for");
            self.is_built = true;
            return Ok(());
        }

        debug!(
            "Building Aho-Corasick automaton with {} patterns",
            self.patterns.len()
        );

        // Build failure links using BFS
        self.build_failure_links()?;

        // Update statistics
        self.update_stats();

        self.is_built = true;
        debug!("Automaton built successfully: {}", self.stats);

        Ok(())
    }

    /// Build failure links using breadth-first search
    fn build_failure_links(&mut self) -> AutomatonResult<()> {
        use std::collections::VecDeque;

        let mut queue = VecDeque::new();

        // Initialize failure links for depth-1 nodes (direct children of root)
        let root_children: Vec<char> = self.nodes[0].child_chars().cloned().collect();

        for ch in root_children {
            if let Some(child_id) = self.nodes[0].get_child(ch) {
                // All direct children of root have failure link to root
                self.nodes[child_id].set_failure_link(0);
                queue.push_back(child_id);
                trace!(
                    "Set failure link for node {} (char '{}') to root",
                    child_id,
                    ch
                );
            }
        }

        // Process remaining nodes level by level
        while let Some(current_id) = queue.pop_front() {
            let current_children: Vec<(char, usize)> = self.nodes[current_id]
                .children
                .iter()
                .map(|(&ch, &child_id)| (ch, child_id))
                .collect();

            for (ch, child_id) in current_children {
                queue.push_back(child_id);

                // Find failure link for this child
                let failure_link = self.compute_failure_link(current_id, ch)?;
                self.nodes[child_id].set_failure_link(failure_link);

                trace!(
                    "Set failure link for node {} (char '{}') to node {}",
                    child_id,
                    ch,
                    failure_link
                );
            }
        }

        Ok(())
    }

    /// Compute the failure link for a node given its parent and the character
    fn compute_failure_link(&self, parent_id: usize, ch: char) -> AutomatonResult<usize> {
        let mut current_id = self.nodes[parent_id].get_failure_link().unwrap_or(0);

        loop {
            // Check if current node has a child for character ch
            if let Some(child_id) = self.nodes[current_id].get_child(ch) {
                return Ok(child_id);
            }

            // If we're at root and no child found, failure link points to root
            if current_id == 0 {
                return Ok(0);
            }

            // Follow failure link of current node
            current_id = self.nodes[current_id].get_failure_link().unwrap_or(0);
        }
    }

    /// Get the next state given current state and input character
    /// This follows failure links until a valid transition is found
    pub fn get_next_state(&self, current_state: usize, ch: char) -> AutomatonResult<usize> {
        if !self.is_built {
            return Err(AutomatonError::NotBuilt);
        }

        let input_char = if self.config.case_sensitive {
            ch
        } else {
            ch.to_lowercase().next().unwrap_or(ch)
        };
        let mut state = current_state;

        loop {
            // Check if current state has a transition for the input character
            if let Some(next_state) = self.nodes[state].get_child(input_char) {
                return Ok(next_state);
            }

            // If we're at root state and no transition found, stay at root
            if state == 0 {
                return Ok(0);
            }

            // Follow failure link
            state = self.nodes[state].get_failure_link().unwrap_or(0);
        }
    }

    /// Check if a state has any output patterns
    pub fn has_output(&self, state: usize) -> AutomatonResult<bool> {
        let node = self.get_node(state)?;
        Ok(node.has_output())
    }

    /// Get output patterns at a given state
    pub fn get_output_patterns(&self, state: usize) -> AutomatonResult<&[Pattern]> {
        let node = self.get_node(state)?;
        Ok(node.get_output_patterns())
    }

    /// Validate that the automaton is properly built
    pub fn validate(&self) -> AutomatonResult<()> {
        if !self.is_built {
            return Err(AutomatonError::NotBuilt);
        }

        // Check that all nodes (except root) have failure links
        for (i, node) in self.nodes.iter().enumerate() {
            if i == 0 {
                // Root should not have a failure link
                if node.get_failure_link().is_some() {
                    return Err(AutomatonError::InvalidNodeId { node_id: i });
                }
            } else {
                // All other nodes should have failure links
                if node.get_failure_link().is_none() {
                    return Err(AutomatonError::InvalidNodeId { node_id: i });
                }
            }
        }

        // Check that failure links point to valid nodes
        for node in &self.nodes {
            if let Some(failure_id) = node.get_failure_link() {
                if failure_id >= self.nodes.len() {
                    return Err(AutomatonError::InvalidNodeId {
                        node_id: failure_id,
                    });
                }
            }
        }

        debug!("Automaton validation passed");
        Ok(())
    }

    /// Get the root node
    pub fn root(&self) -> &AutomatonNode {
        &self.nodes[0]
    }

    /// Get a node by ID
    pub fn get_node(&self, node_id: usize) -> AutomatonResult<&AutomatonNode> {
        self.nodes
            .get(node_id)
            .ok_or(AutomatonError::InvalidNodeId { node_id })
    }

    /// Get a mutable node by ID
    #[allow(dead_code)]
    fn get_node_mut(&mut self, node_id: usize) -> AutomatonResult<&mut AutomatonNode> {
        self.nodes
            .get_mut(node_id)
            .ok_or(AutomatonError::InvalidNodeId { node_id })
    }

    /// Get all patterns
    pub fn patterns(&self) -> &[Pattern] {
        &self.patterns
    }

    /// Get configuration
    pub fn config(&self) -> &AutomatonConfig {
        &self.config
    }

    /// Check if the automaton is built (failure links computed)
    pub fn is_built(&self) -> bool {
        self.is_built
    }

    /// Get statistics about the automaton
    pub fn stats(&self) -> &AutomatonStats {
        &self.stats
    }

    /// Update statistics
    fn update_stats(&mut self) {
        self.stats.node_count = self.nodes.len();
        self.stats.pattern_count = self.patterns.len();
        self.stats.max_depth = self.nodes.iter().map(|n| n.depth).max().unwrap_or(0);
        self.stats.edge_count = self.nodes.iter().map(|n| n.children.len()).sum();
        self.stats
            .calculate_memory_estimate(&self.nodes, &self.patterns);
    }

    /// Clear all patterns and reset the automaton
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.nodes.push(AutomatonNode::root()); // Re-add root
        self.patterns.clear();
        self.is_built = false;
        self.next_node_id = 1;
        self.next_pattern_id = 0;
        self.stats = AutomatonStats::new();

        debug!("Cleared automaton");
    }

    /// Get the number of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of patterns
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    /// Check if the automaton is empty (no patterns)
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// Find all matches in the given text
    pub fn find_matches(&self, text: &str) -> AutomatonResult<Vec<PatternMatch>> {
        if !self.is_built {
            return Err(AutomatonError::NotBuilt);
        }

        let mut matches = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let mut state = 0;

        for (pos, &ch) in chars.iter().enumerate() {
            // Get next state
            state = self.get_next_state(state, ch)?;

            // Check for matches at current position by walking failure links
            let mut current_state = state;
            while current_state != 0 {
                // Check if current state has output patterns
                if !self.nodes[current_state].output_patterns.is_empty() {
                    for pattern in &self.nodes[current_state].output_patterns {
                        let start_pos = pos + 1 - pattern.char_len();
                        let end_pos = pos + 1;

                        let pattern_match = PatternMatch::new(pattern.clone(), start_pos, end_pos);
                        matches.push(pattern_match);
                    }
                }

                // Follow failure link to find more matches
                current_state = self.nodes[current_state].get_failure_link().unwrap_or(0);
            }
        }

        // If not finding overlapping matches, filter out overlapping ones
        if !self.config.find_overlapping {
            matches = self.filter_overlapping_matches(matches);
        }

        debug!(
            "Found {} matches in text of length {}",
            matches.len(),
            chars.len()
        );
        Ok(matches)
    }

    /// Filter overlapping matches to keep only non-overlapping ones (leftmost-longest)
    fn filter_overlapping_matches(&self, mut matches: Vec<PatternMatch>) -> Vec<PatternMatch> {
        if matches.is_empty() {
            return matches;
        }

        // Sort by start position, then by end position (descending for longest first)
        matches.sort_by(|a, b| {
            a.start_pos
                .cmp(&b.start_pos)
                .then_with(|| b.end_pos.cmp(&a.end_pos))
        });

        let mut filtered = Vec::new();
        let mut last_end = 0;

        for pattern_match in matches {
            if pattern_match.start_pos >= last_end {
                last_end = pattern_match.end_pos;
                filtered.push(pattern_match);
            }
        }

        filtered
    }

    /// Find overlapping matches in the given text
    pub fn find_overlapping_matches(&self, text: &str) -> AutomatonResult<Vec<PatternMatch>> {
        if !self.is_built {
            return Err(AutomatonError::NotBuilt);
        }

        let mut matches = Vec::new();
        let chars: Vec<char> = text.chars().collect();

        // For overlapping matches, we need to check all possible starting positions
        for start_pos in 0..chars.len() {
            let mut state = 0;

            for (offset, &ch) in chars[start_pos..].iter().enumerate() {
                // Get next state
                state = self.get_next_state(state, ch)?;

                // Check for matches at current position
                if self.has_output(state)? {
                    let output_patterns = self.get_output_patterns(state)?;

                    for pattern in output_patterns {
                        let match_start = start_pos + offset + 1 - pattern.char_len();
                        let match_end = start_pos + offset + 1;

                        // Only add if this match starts at the current start_pos
                        if match_start == start_pos {
                            let pattern_match =
                                PatternMatch::new(pattern.clone(), match_start, match_end);
                            matches.push(pattern_match);
                        }
                    }
                }
            }
        }

        // Remove duplicates and sort by position
        matches.sort_by(|a, b| {
            a.start_pos
                .cmp(&b.start_pos)
                .then_with(|| a.end_pos.cmp(&b.end_pos))
                .then_with(|| a.pattern_id.cmp(&b.pattern_id))
        });
        matches.dedup_by(|a, b| {
            a.start_pos == b.start_pos && a.end_pos == b.end_pos && a.pattern_id == b.pattern_id
        });

        debug!(
            "Found {} overlapping matches in text of length {}",
            matches.len(),
            chars.len()
        );
        Ok(matches)
    }

    /// Find the first match in the given text
    pub fn find_first_match(&self, text: &str) -> AutomatonResult<Option<PatternMatch>> {
        if !self.is_built {
            return Err(AutomatonError::NotBuilt);
        }

        let chars: Vec<char> = text.chars().collect();
        let mut state = 0;

        for (pos, &ch) in chars.iter().enumerate() {
            // Get next state
            state = self.get_next_state(state, ch)?;

            // Check for matches at current position
            if self.has_output(state)? {
                let output_patterns = self.get_output_patterns(state)?;

                if let Some(pattern) = output_patterns.first() {
                    let start_pos = pos + 1 - pattern.char_len();
                    let end_pos = pos + 1;

                    let pattern_match = PatternMatch::new(pattern.clone(), start_pos, end_pos);
                    return Ok(Some(pattern_match));
                }
            }
        }

        Ok(None)
    }

    /// Check if the text contains any of the patterns
    pub fn contains_match(&self, text: &str) -> AutomatonResult<bool> {
        if !self.is_built {
            return Err(AutomatonError::NotBuilt);
        }

        let chars: Vec<char> = text.chars().collect();
        let mut state = 0;

        for &ch in chars.iter() {
            // Get next state
            state = self.get_next_state(state, ch)?;

            // Check for matches at current position
            if self.has_output(state)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Find matches and return them grouped by category
    pub fn find_matches_by_category(
        &self,
        text: &str,
    ) -> AutomatonResult<HashMap<String, Vec<PatternMatch>>> {
        let matches = if self.config.find_overlapping {
            self.find_overlapping_matches(text)?
        } else {
            self.find_matches(text)?
        };

        let mut categorized = HashMap::new();

        for pattern_match in matches {
            categorized
                .entry(pattern_match.category.clone())
                .or_insert_with(Vec::new)
                .push(pattern_match);
        }

        Ok(categorized)
    }

    /// Get match statistics for the given text
    pub fn get_match_statistics(&self, text: &str) -> AutomatonResult<MatchStatistics> {
        let matches = if self.config.find_overlapping {
            self.find_overlapping_matches(text)?
        } else {
            self.find_matches(text)?
        };

        let mut category_counts = HashMap::new();
        let mut total_score = 0.0;
        let mut max_score: f64 = 0.0;
        let mut min_score = f64::INFINITY;

        for pattern_match in &matches {
            *category_counts
                .entry(pattern_match.category.clone())
                .or_insert(0) += 1;
            total_score += pattern_match.score;
            max_score = max_score.max(pattern_match.score);
            min_score = min_score.min(pattern_match.score);
        }

        if matches.is_empty() {
            min_score = 0.0;
        }

        let stats = MatchStatistics {
            total_matches: matches.len(),
            unique_patterns: matches
                .iter()
                .map(|m| m.pattern_id)
                .collect::<std::collections::HashSet<_>>()
                .len(),
            categories_found: category_counts.keys().cloned().collect(),
            category_counts,
            total_score,
            average_score: if matches.is_empty() {
                0.0
            } else {
                total_score / matches.len() as f64
            },
            max_score,
            min_score,
            text_length: text.chars().count(),
            coverage_ratio: if text.is_empty() {
                0.0
            } else {
                matches.iter().map(|m| m.match_len()).sum::<usize>() as f64
                    / text.chars().count() as f64
            },
        };

        Ok(stats)
    }
}

impl Default for AhoCorasickMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for AhoCorasickMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AhoCorasickMatcher(patterns={}, nodes={}, built={}, {})",
            self.pattern_count(),
            self.node_count(),
            self.is_built(),
            self.stats
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_creation() {
        let pattern = Pattern::new("test", "category", 1.0, 0);
        assert_eq!(pattern.pattern, "test");
        assert_eq!(pattern.category, "category");
        assert_eq!(pattern.score, 1.0);
        assert_eq!(pattern.id, 0);
        assert_eq!(pattern.char_len(), 4);
    }

    #[test]
    fn test_pattern_lowercase() {
        let pattern = Pattern::new("TeSt", "category", 1.0, 0);
        assert_eq!(pattern.to_lowercase(), "test");
    }

    #[test]
    fn test_automaton_node_creation() {
        let root = AutomatonNode::root();
        assert_eq!(root.id, 0);
        assert_eq!(root.depth, 0);
        assert!(root.parent.is_none());
        assert!(root.parent_char.is_none());
        assert!(!root.has_output());
        assert_eq!(root.children.len(), 0);
    }

    #[test]
    fn test_automaton_node_children() {
        let mut node = AutomatonNode::new(1, 1, Some(0), Some('a'));

        assert!(!node.has_child('b'));
        assert!(node.get_child('b').is_none());

        node.add_child('b', 2);
        assert!(node.has_child('b'));
        assert_eq!(node.get_child('b'), Some(2));
    }

    #[test]
    fn test_automaton_node_output() {
        let mut node = AutomatonNode::new(1, 1, Some(0), Some('a'));
        let pattern = Pattern::new("test", "category", 1.0, 0);

        assert!(!node.has_output());
        assert_eq!(node.get_output_patterns().len(), 0);

        node.add_output_pattern(pattern.clone());
        assert!(node.has_output());
        assert_eq!(node.get_output_patterns().len(), 1);
        assert_eq!(node.get_output_patterns()[0], pattern);
    }

    #[test]
    fn test_pattern_match_creation() {
        let pattern = Pattern::new("test", "category", 1.5, 0);
        let match_result = PatternMatch::new(pattern, 5, 9);

        assert_eq!(match_result.pattern, "test");
        assert_eq!(match_result.category, "category");
        assert_eq!(match_result.score, 1.5);
        assert_eq!(match_result.pattern_id, 0);
        assert_eq!(match_result.start_pos, 5);
        assert_eq!(match_result.end_pos, 9);
        assert_eq!(match_result.match_len(), 4);
    }

    #[test]
    fn test_pattern_match_overlap() {
        let pattern1 = Pattern::new("test", "category", 1.0, 0);
        let pattern2 = Pattern::new("other", "category", 1.0, 1);

        let match1 = PatternMatch::new(pattern1, 5, 9);
        let match2 = PatternMatch::new(pattern2.clone(), 7, 12); // Overlaps
        let match3 = PatternMatch::new(pattern2, 10, 15); // No overlap

        assert!(match1.overlaps_with(&match2));
        assert!(match2.overlaps_with(&match1));
        assert!(!match1.overlaps_with(&match3));
        assert!(!match3.overlaps_with(&match1));
    }

    #[test]
    fn test_automaton_config_default() {
        let config = AutomatonConfig::default();
        assert!(!config.case_sensitive);
        assert!(!config.find_overlapping);
        assert_eq!(config.max_patterns, 10_000);
        assert_eq!(config.max_pattern_length, 1000);
    }

    #[test]
    fn test_automaton_stats() {
        let mut stats = AutomatonStats::new();
        assert_eq!(stats.node_count, 0);
        assert_eq!(stats.pattern_count, 0);
        assert_eq!(stats.max_depth, 0);
        assert_eq!(stats.edge_count, 0);
        assert_eq!(stats.estimated_memory_bytes, 0);

        // Test memory calculation with empty data
        let nodes = vec![];
        let patterns = vec![];
        stats.calculate_memory_estimate(&nodes, &patterns);
        assert_eq!(stats.estimated_memory_bytes, 0);
    }

    #[test]
    fn test_unicode_support() {
        let pattern = Pattern::new("测试", "chinese", 1.0, 0);
        assert_eq!(pattern.char_len(), 2); // 2 Chinese characters
        assert_eq!(pattern.pattern.len(), 6); // 6 bytes in UTF-8
    }

    #[test]
    fn test_error_types() {
        let error = AutomatonError::PatternTooLong {
            length: 1001,
            max_length: 1000,
        };
        assert!(error.to_string().contains("Pattern too long"));

        let error = AutomatonError::EmptyPattern;
        assert!(error.to_string().contains("Empty pattern"));

        let error = AutomatonError::NotBuilt;
        assert!(error.to_string().contains("not built"));
    }
}

#[cfg(test)]
mod automaton_tests {
    use super::*;

    #[test]
    fn test_automaton_creation() {
        let matcher = AhoCorasickMatcher::new();
        assert_eq!(matcher.node_count(), 1); // Root node
        assert_eq!(matcher.pattern_count(), 0);
        assert!(!matcher.is_built());
        assert!(matcher.is_empty());
    }

    #[test]
    fn test_automaton_with_config() {
        let config = AutomatonConfig {
            case_sensitive: true,
            find_overlapping: true,
            max_patterns: 100,
            max_pattern_length: 50,
        };

        let matcher = AhoCorasickMatcher::with_config(config.clone());
        assert_eq!(matcher.config().case_sensitive, true);
        assert_eq!(matcher.config().find_overlapping, true);
        assert_eq!(matcher.config().max_patterns, 100);
        assert_eq!(matcher.config().max_pattern_length, 50);
    }

    #[test]
    fn test_add_single_pattern() {
        let mut matcher = AhoCorasickMatcher::new();

        let result = matcher.add_pattern("test", "category", 1.0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0); // First pattern gets ID 0

        assert_eq!(matcher.pattern_count(), 1);
        assert!(matcher.node_count() > 1); // Should have created nodes
        assert!(!matcher.is_empty());

        let patterns = matcher.patterns();
        assert_eq!(patterns[0].pattern, "test");
        assert_eq!(patterns[0].category, "category");
        assert_eq!(patterns[0].score, 1.0);
        assert_eq!(patterns[0].id, 0);
    }

    #[test]
    fn test_add_multiple_patterns() {
        let mut matcher = AhoCorasickMatcher::new();

        let id1 = matcher.add_pattern("hello", "greeting", 1.0).unwrap();
        let id2 = matcher.add_pattern("world", "noun", 0.8).unwrap();
        let id3 = matcher.add_pattern("test", "action", 1.2).unwrap();

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 2);

        assert_eq!(matcher.pattern_count(), 3);
        assert!(matcher.node_count() > 3); // Should have created multiple nodes
    }

    #[test]
    fn test_add_overlapping_patterns() {
        let mut matcher = AhoCorasickMatcher::new();

        // Add patterns that share prefixes
        matcher.add_pattern("he", "pronoun", 1.0).unwrap();
        matcher.add_pattern("her", "pronoun", 1.0).unwrap();
        matcher.add_pattern("hello", "greeting", 1.0).unwrap();

        assert_eq!(matcher.pattern_count(), 3);

        // The trie should efficiently share nodes for common prefixes
        // "he" and "her" should share the "he" prefix
        // "he" and "hello" should share the "he" prefix
    }

    #[test]
    fn test_case_sensitivity() {
        let mut case_sensitive = AhoCorasickMatcher::with_config(AutomatonConfig {
            case_sensitive: true,
            ..Default::default()
        });

        let mut case_insensitive = AhoCorasickMatcher::with_config(AutomatonConfig {
            case_sensitive: false,
            ..Default::default()
        });

        case_sensitive.add_pattern("Test", "category", 1.0).unwrap();
        case_insensitive
            .add_pattern("Test", "category", 1.0)
            .unwrap();

        // Both should accept the pattern, but internal representation may differ
        assert_eq!(case_sensitive.pattern_count(), 1);
        assert_eq!(case_insensitive.pattern_count(), 1);
    }

    #[test]
    fn test_empty_pattern_error() {
        let mut matcher = AhoCorasickMatcher::new();

        let result = matcher.add_pattern("", "category", 1.0);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AutomatonError::EmptyPattern));
    }

    #[test]
    fn test_pattern_too_long_error() {
        let mut matcher = AhoCorasickMatcher::with_config(AutomatonConfig {
            max_pattern_length: 5,
            ..Default::default()
        });

        let result = matcher.add_pattern("toolong", "category", 1.0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AutomatonError::PatternTooLong { .. }
        ));
    }

    #[test]
    fn test_too_many_patterns_error() {
        let mut matcher = AhoCorasickMatcher::with_config(AutomatonConfig {
            max_patterns: 2,
            ..Default::default()
        });

        matcher.add_pattern("one", "category", 1.0).unwrap();
        matcher.add_pattern("two", "category", 1.0).unwrap();

        let result = matcher.add_pattern("three", "category", 1.0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AutomatonError::TooManyPatterns { .. }
        ));
    }

    #[test]
    fn test_duplicate_pattern_error() {
        let mut matcher = AhoCorasickMatcher::new();

        matcher.add_pattern("test", "category1", 1.0).unwrap();

        let result = matcher.add_pattern("test", "category2", 2.0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AutomatonError::DuplicatePattern { .. }
        ));
    }

    #[test]
    fn test_unicode_patterns() {
        let mut matcher = AhoCorasickMatcher::new();

        // Test Chinese characters
        matcher.add_pattern("测试", "chinese", 1.0).unwrap();
        matcher.add_pattern("hello世界", "mixed", 1.0).unwrap();

        assert_eq!(matcher.pattern_count(), 2);

        let patterns = matcher.patterns();
        assert_eq!(patterns[0].pattern, "测试");
        assert_eq!(patterns[1].pattern, "hello世界");
    }

    #[test]
    fn test_clear_automaton() {
        let mut matcher = AhoCorasickMatcher::new();

        matcher.add_pattern("test1", "category", 1.0).unwrap();
        matcher.add_pattern("test2", "category", 1.0).unwrap();

        assert_eq!(matcher.pattern_count(), 2);
        assert!(matcher.node_count() > 1);

        matcher.clear();

        assert_eq!(matcher.pattern_count(), 0);
        assert_eq!(matcher.node_count(), 1); // Only root remains
        assert!(matcher.is_empty());
        assert!(!matcher.is_built());
    }

    #[test]
    fn test_get_node() {
        let matcher = AhoCorasickMatcher::new();

        // Root node should exist
        let root = matcher.get_node(0);
        assert!(root.is_ok());
        assert_eq!(root.unwrap().id, 0);

        // Non-existent node should return error
        let result = matcher.get_node(999);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AutomatonError::InvalidNodeId { .. }
        ));
    }

    #[test]
    fn test_stats_update() {
        let mut matcher = AhoCorasickMatcher::new();

        matcher.add_pattern("test", "category", 1.0).unwrap();
        matcher.add_pattern("hello", "greeting", 1.0).unwrap();

        matcher.update_stats();

        let stats = matcher.stats();
        assert!(stats.node_count > 1);
        assert_eq!(stats.pattern_count, 2);
        assert!(stats.max_depth > 0);
        assert!(stats.edge_count > 0);
        assert!(stats.estimated_memory_bytes > 0);
    }
}

#[cfg(test)]
mod failure_link_tests {
    use super::*;

    #[test]
    fn test_build_empty_automaton() {
        let mut matcher = AhoCorasickMatcher::new();

        let result = matcher.build();
        assert!(result.is_ok());
        assert!(matcher.is_built());
    }

    #[test]
    fn test_build_single_pattern() {
        let mut matcher = AhoCorasickMatcher::new();
        matcher.add_pattern("test", "category", 1.0).unwrap();

        let result = matcher.build();
        assert!(result.is_ok());
        assert!(matcher.is_built());

        // Validate the automaton
        assert!(matcher.validate().is_ok());
    }

    #[test]
    fn test_build_multiple_patterns() {
        let mut matcher = AhoCorasickMatcher::new();
        matcher.add_pattern("he", "pronoun", 1.0).unwrap();
        matcher.add_pattern("she", "pronoun", 1.0).unwrap();
        matcher.add_pattern("his", "possessive", 1.0).unwrap();
        matcher.add_pattern("hers", "possessive", 1.0).unwrap();

        let result = matcher.build();
        assert!(result.is_ok());
        assert!(matcher.is_built());

        // Validate the automaton
        assert!(matcher.validate().is_ok());
    }

    #[test]
    fn test_build_overlapping_patterns() {
        let mut matcher = AhoCorasickMatcher::new();

        // Classic Aho-Corasick example
        matcher.add_pattern("he", "pronoun", 1.0).unwrap();
        matcher.add_pattern("she", "pronoun", 1.0).unwrap();
        matcher.add_pattern("her", "pronoun", 1.0).unwrap();
        matcher.add_pattern("hers", "possessive", 1.0).unwrap();

        let result = matcher.build();
        assert!(result.is_ok());
        assert!(matcher.is_built());

        // Validate the automaton
        assert!(matcher.validate().is_ok());

        // Test some state transitions
        let state1 = matcher.get_next_state(0, 'h').unwrap();
        assert!(state1 > 0);

        let state2 = matcher.get_next_state(state1, 'e').unwrap();
        assert!(state2 > 0);

        // Should have output at this state (pattern "he")
        assert!(matcher.has_output(state2).unwrap());
        let outputs = matcher.get_output_patterns(state2).unwrap();
        assert!(outputs.iter().any(|p| p.pattern == "he"));
    }

    #[test]
    fn test_failure_link_construction() {
        let mut matcher = AhoCorasickMatcher::new();

        // Add patterns that will create interesting failure links
        matcher.add_pattern("aba", "pattern1", 1.0).unwrap();
        matcher.add_pattern("ab", "pattern2", 1.0).unwrap();
        matcher.add_pattern("a", "pattern3", 1.0).unwrap();

        matcher.build().unwrap();

        // Test state transitions and failure links
        let state_a = matcher.get_next_state(0, 'a').unwrap();
        assert!(state_a > 0);

        let state_ab = matcher.get_next_state(state_a, 'b').unwrap();
        assert!(state_ab > 0);

        let state_aba = matcher.get_next_state(state_ab, 'a').unwrap();
        assert!(state_aba > 0);

        // Validate the automaton structure
        assert!(matcher.validate().is_ok());
    }

    #[test]
    fn test_case_insensitive_failure_links() {
        let mut matcher = AhoCorasickMatcher::with_config(AutomatonConfig {
            case_sensitive: false,
            ..Default::default()
        });

        matcher.add_pattern("Test", "category", 1.0).unwrap();

        // Second pattern should fail because it's a duplicate when case-insensitive
        // "TEST" should be treated as the same as "Test" when case_sensitive is false
        assert!(matcher.add_pattern("TEST", "category3", 1.0).is_err());

        matcher.build().unwrap();

        // Test that case doesn't matter for transitions
        let state1 = matcher.get_next_state(0, 't').unwrap();
        let state2 = matcher.get_next_state(0, 'T').unwrap();
        assert_eq!(state1, state2);

        assert!(matcher.validate().is_ok());
    }

    #[test]
    fn test_unicode_failure_links() {
        let mut matcher = AhoCorasickMatcher::new();

        matcher.add_pattern("测试", "chinese", 1.0).unwrap();
        matcher.add_pattern("测", "chinese_char", 1.0).unwrap();
        matcher.add_pattern("试验", "test", 1.0).unwrap();

        matcher.build().unwrap();

        // Test Unicode character transitions
        let state1 = matcher.get_next_state(0, '测').unwrap();
        assert!(state1 > 0);

        let state2 = matcher.get_next_state(state1, '试').unwrap();
        assert!(state2 > 0);

        // Should have outputs for both "测" and "测试"
        assert!(matcher.has_output(state1).unwrap()); // "测"
        assert!(matcher.has_output(state2).unwrap()); // "测试"

        assert!(matcher.validate().is_ok());
    }

    #[test]
    fn test_get_next_state_not_built() {
        let matcher = AhoCorasickMatcher::new();

        let result = matcher.get_next_state(0, 'a');
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AutomatonError::NotBuilt));
    }

    #[test]
    fn test_validate_not_built() {
        let matcher = AhoCorasickMatcher::new();

        let result = matcher.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AutomatonError::NotBuilt));
    }

    #[test]
    fn test_rebuild_automaton() {
        let mut matcher = AhoCorasickMatcher::new();

        matcher.add_pattern("test", "category", 1.0).unwrap();
        matcher.build().unwrap();
        assert!(matcher.is_built());

        // Adding a new pattern should mark as not built
        matcher.add_pattern("hello", "greeting", 1.0).unwrap();
        assert!(!matcher.is_built());

        // Should be able to rebuild
        matcher.build().unwrap();
        assert!(matcher.is_built());
        assert!(matcher.validate().is_ok());
    }

    #[test]
    fn test_complex_failure_links() {
        let mut matcher = AhoCorasickMatcher::new();

        // Add patterns that create complex failure link scenarios
        matcher.add_pattern("abcab", "pattern1", 1.0).unwrap();
        matcher.add_pattern("abc", "pattern2", 1.0).unwrap();
        matcher.add_pattern("cab", "pattern3", 1.0).unwrap();
        matcher.add_pattern("ab", "pattern4", 1.0).unwrap();

        matcher.build().unwrap();

        // Test various state transitions
        let mut state = 0;

        // Process "abcab"
        state = matcher.get_next_state(state, 'a').unwrap();
        state = matcher.get_next_state(state, 'b').unwrap();
        assert!(matcher.has_output(state).unwrap()); // Should match "ab"

        state = matcher.get_next_state(state, 'c').unwrap();
        assert!(matcher.has_output(state).unwrap()); // Should match "abc"

        state = matcher.get_next_state(state, 'a').unwrap();
        state = matcher.get_next_state(state, 'b').unwrap();
        assert!(matcher.has_output(state).unwrap()); // Should match "abcab" and "ab"

        assert!(matcher.validate().is_ok());
    }
}

/// Statistics about pattern matching results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchStatistics {
    /// Total number of matches found
    pub total_matches: usize,
    /// Number of unique patterns that matched
    pub unique_patterns: usize,
    /// Categories that had matches
    pub categories_found: Vec<String>,
    /// Count of matches per category
    pub category_counts: HashMap<String, usize>,
    /// Sum of all match scores
    pub total_score: f64,
    /// Average score of matches
    pub average_score: f64,
    /// Highest score among matches
    pub max_score: f64,
    /// Lowest score among matches
    pub min_score: f64,
    /// Length of the input text in characters
    pub text_length: usize,
    /// Ratio of matched characters to total characters
    pub coverage_ratio: f64,
}

impl fmt::Display for MatchStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MatchStats(matches={}, patterns={}, categories={}, avg_score={:.2}, coverage={:.1}%)",
            self.total_matches,
            self.unique_patterns,
            self.categories_found.len(),
            self.average_score,
            self.coverage_ratio * 100.0
        )
    }
}

#[cfg(test)]
mod matching_tests {
    use super::*;

    #[test]
    fn test_simple_matching() {
        let mut matcher = AhoCorasickMatcher::with_config(AutomatonConfig {
            find_overlapping: true,
            ..Default::default()
        });

        matcher.add_pattern("he", "pronoun", 1.0).unwrap();
        matcher.add_pattern("she", "pronoun", 1.5).unwrap();
        matcher.add_pattern("his", "possessive", 2.0).unwrap();

        matcher.build().unwrap();

        let matches = matcher.find_matches("she sells his shells").unwrap();

        assert_eq!(matches.len(), 5);

        // Check matches in order they appear
        assert_eq!(matches[0].pattern, "she");
        assert_eq!(matches[0].start_pos, 0);
        assert_eq!(matches[0].end_pos, 3);

        assert_eq!(matches[1].pattern, "he");
        assert_eq!(matches[1].start_pos, 1);
        assert_eq!(matches[1].end_pos, 3);

        assert_eq!(matches[2].pattern, "his");
        assert_eq!(matches[2].start_pos, 10);
        assert_eq!(matches[2].end_pos, 13);

        assert_eq!(matches[3].pattern, "she");
        assert_eq!(matches[3].start_pos, 14);
        assert_eq!(matches[3].end_pos, 17);

        assert_eq!(matches[4].pattern, "he");
        assert_eq!(matches[4].start_pos, 15);
        assert_eq!(matches[4].end_pos, 17);
    }

    #[test]
    fn test_case_insensitive_matching() {
        let mut matcher = AhoCorasickMatcher::with_config(AutomatonConfig {
            case_sensitive: false,
            ..Default::default()
        });

        matcher.add_pattern("Hello", "greeting", 1.0).unwrap();
        matcher.add_pattern("WORLD", "noun", 1.0).unwrap();

        matcher.build().unwrap();

        let matches = matcher.find_matches("hello world HELLO world").unwrap();

        assert_eq!(matches.len(), 4);

        // All matches should be found regardless of case
        assert!(matches
            .iter()
            .any(|m| m.pattern == "Hello" && m.start_pos == 0));
        assert!(matches
            .iter()
            .any(|m| m.pattern == "WORLD" && m.start_pos == 6));
        assert!(matches
            .iter()
            .any(|m| m.pattern == "Hello" && m.start_pos == 12));
        assert!(matches
            .iter()
            .any(|m| m.pattern == "WORLD" && m.start_pos == 18));
    }

    #[test]
    fn test_overlapping_matches() {
        let mut matcher = AhoCorasickMatcher::with_config(AutomatonConfig {
            find_overlapping: true,
            ..Default::default()
        });

        matcher.add_pattern("abc", "pattern1", 1.0).unwrap();
        matcher.add_pattern("bcd", "pattern2", 1.0).unwrap();
        matcher.add_pattern("cde", "pattern3", 1.0).unwrap();

        matcher.build().unwrap();

        let matches = matcher.find_overlapping_matches("abcde").unwrap();

        assert_eq!(matches.len(), 3);

        // Check all overlapping matches are found
        assert!(matches
            .iter()
            .any(|m| m.pattern == "abc" && m.start_pos == 0));
        assert!(matches
            .iter()
            .any(|m| m.pattern == "bcd" && m.start_pos == 1));
        assert!(matches
            .iter()
            .any(|m| m.pattern == "cde" && m.start_pos == 2));
    }

    #[test]
    fn test_unicode_matching() {
        let mut matcher = AhoCorasickMatcher::new();

        matcher.add_pattern("测试", "chinese", 1.0).unwrap();
        matcher.add_pattern("hello", "english", 1.0).unwrap();
        matcher.add_pattern("世界", "world", 1.0).unwrap();

        matcher.build().unwrap();

        let matches = matcher.find_matches("hello测试世界").unwrap();

        assert_eq!(matches.len(), 3);

        // Check Unicode matches
        assert!(matches
            .iter()
            .any(|m| m.pattern == "hello" && m.start_pos == 0));
        assert!(matches
            .iter()
            .any(|m| m.pattern == "测试" && m.start_pos == 5));
        assert!(matches
            .iter()
            .any(|m| m.pattern == "世界" && m.start_pos == 7));
    }

    #[test]
    fn test_find_first_match() {
        let mut matcher = AhoCorasickMatcher::new();

        matcher.add_pattern("abc", "pattern1", 1.0).unwrap();
        matcher.add_pattern("def", "pattern2", 2.0).unwrap();

        matcher.build().unwrap();

        let first_match = matcher.find_first_match("xyzabcdef").unwrap();

        assert!(first_match.is_some());
        let match_result = first_match.unwrap();
        assert_eq!(match_result.pattern, "abc");
        assert_eq!(match_result.start_pos, 3);
        assert_eq!(match_result.end_pos, 6);
    }

    #[test]
    fn test_contains_match() {
        let mut matcher = AhoCorasickMatcher::new();

        matcher.add_pattern("needle", "target", 1.0).unwrap();

        matcher.build().unwrap();

        assert!(matcher.contains_match("haystack needle haystack").unwrap());
        assert!(!matcher.contains_match("haystack haystack").unwrap());
    }

    #[test]
    fn test_matches_by_category() {
        let mut matcher = AhoCorasickMatcher::new();

        matcher.add_pattern("cat", "animal", 1.0).unwrap();
        matcher.add_pattern("dog", "animal", 1.0).unwrap();
        matcher.add_pattern("red", "color", 1.0).unwrap();
        matcher.add_pattern("blue", "color", 1.0).unwrap();

        matcher.build().unwrap();

        let categorized = matcher
            .find_matches_by_category("red cat and blue dog")
            .unwrap();

        assert_eq!(categorized.len(), 2);
        assert_eq!(categorized["animal"].len(), 2);
        assert_eq!(categorized["color"].len(), 2);
    }

    #[test]
    fn test_match_statistics() {
        let mut matcher = AhoCorasickMatcher::new();

        matcher.add_pattern("test", "category1", 1.0).unwrap();
        matcher.add_pattern("hello", "category2", 2.0).unwrap();
        matcher.add_pattern("world", "category2", 3.0).unwrap();

        matcher.build().unwrap();

        let text = "test hello world";
        let stats = matcher.get_match_statistics(text).unwrap();

        assert_eq!(stats.total_matches, 3);
        assert_eq!(stats.unique_patterns, 3);
        assert_eq!(stats.categories_found.len(), 2);
        assert_eq!(stats.category_counts["category1"], 1);
        assert_eq!(stats.category_counts["category2"], 2);
        assert_eq!(stats.total_score, 6.0);
        assert_eq!(stats.average_score, 2.0);
        assert_eq!(stats.max_score, 3.0);
        assert_eq!(stats.min_score, 1.0);
        assert_eq!(stats.text_length, 16); // Corrected from 17 to 16
    }

    #[test]
    fn test_empty_text() {
        let mut matcher = AhoCorasickMatcher::new();

        matcher.add_pattern("test", "category", 1.0).unwrap();
        matcher.build().unwrap();

        let matches = matcher.find_matches("").unwrap();
        assert_eq!(matches.len(), 0);

        let first_match = matcher.find_first_match("").unwrap();
        assert!(first_match.is_none());

        assert!(!matcher.contains_match("").unwrap());

        let stats = matcher.get_match_statistics("").unwrap();
        assert_eq!(stats.total_matches, 0);
        assert_eq!(stats.text_length, 0);
    }

    #[test]
    fn test_no_matches() {
        let mut matcher = AhoCorasickMatcher::new();

        matcher.add_pattern("needle", "target", 1.0).unwrap();
        matcher.build().unwrap();

        let matches = matcher.find_matches("haystack haystack").unwrap();
        assert_eq!(matches.len(), 0);

        let first_match = matcher.find_first_match("haystack haystack").unwrap();
        assert!(first_match.is_none());

        assert!(!matcher.contains_match("haystack haystack").unwrap());
    }

    #[test]
    fn test_pattern_match_overlap_detection() {
        let pattern1 = Pattern::new("test", "category", 1.0, 0);
        let pattern2 = Pattern::new("other", "category", 1.0, 1);

        let match1 = PatternMatch::new(pattern1, 0, 4);
        let match2 = PatternMatch::new(pattern2.clone(), 2, 7); // Overlaps with match1
        let match3 = PatternMatch::new(pattern2, 5, 10); // No overlap with match1

        assert!(match1.overlaps_with(&match2));
        assert!(match2.overlaps_with(&match1));
        assert!(!match1.overlaps_with(&match3));
        assert!(!match3.overlaps_with(&match1));
    }

    #[test]
    fn test_complex_overlapping_scenario() {
        let mut matcher = AhoCorasickMatcher::with_config(AutomatonConfig {
            find_overlapping: true,
            ..Default::default()
        });

        // Create patterns that will have complex overlapping behavior
        matcher.add_pattern("abab", "pattern1", 1.0).unwrap();
        matcher.add_pattern("baba", "pattern2", 1.0).unwrap();
        matcher.add_pattern("ab", "pattern3", 1.0).unwrap();
        matcher.add_pattern("ba", "pattern4", 1.0).unwrap();

        matcher.build().unwrap();

        let matches = matcher.find_matches("abababa").unwrap();

        // Should find multiple overlapping matches
        assert!(matches.len() >= 4);

        // Verify some expected matches
        assert!(matches.iter().any(|m| m.pattern == "abab"));
        assert!(matches.iter().any(|m| m.pattern == "baba"));
        assert!(matches.iter().any(|m| m.pattern == "ab"));
        assert!(matches.iter().any(|m| m.pattern == "ba"));
    }
}
