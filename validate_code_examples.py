#!/usr/bin/env python3
"""
Code Examples and Schema Validation Script
Validates JSON schemas and code examples in documentation
"""

import json
import os
import re
import sys
import subprocess
from pathlib import Path
from typing import List, Dict, Tuple, Any

class CodeValidator:
    def __init__(self, root_dir: str = "."):
        self.root_dir = Path(root_dir)
        self.errors = []
        self.warnings = []
        
    def find_markdown_files(self) -> List[Path]:
        """Find all markdown files in the project"""
        md_files = []
        for pattern in ["*.md", "**/*.md"]:
            md_files.extend(self.root_dir.glob(pattern))
        return sorted(md_files)
    
    def extract_code_blocks(self, content: str) -> List[Tuple[str, str, int]]:
        """Extract code blocks with language and line numbers"""
        code_blocks = []
        lines = content.split('\n')
        
        i = 0
        while i < len(lines):
            line = lines[i].strip()
            if line.startswith('```'):
                # Extract language
                lang = line[3:].strip()
                if not lang:
                    lang = "text"
                
                # Extract code content
                code_lines = []
                i += 1
                start_line = i + 1
                
                while i < len(lines) and not lines[i].strip().startswith('```'):
                    code_lines.append(lines[i])
                    i += 1
                
                code_content = '\n'.join(code_lines)
                code_blocks.append((lang, code_content, start_line))
            
            i += 1
        
        return code_blocks
    
    def validate_json_schema(self, json_content: str, file_path: Path, line_num: int) -> bool:
        """Validate JSON syntax and structure"""
        try:
            parsed = json.loads(json_content)
            
            # Additional validation for common schema patterns
            if isinstance(parsed, dict):
                # Check for common schema fields
                if "type" in parsed and parsed["type"] not in ["object", "array", "string", "number", "boolean", "null"]:
                    self.warnings.append(f"Unusual JSON schema type in {file_path}:{line_num}: {parsed['type']}")
                
                # Check for required fields in tool schemas
                if "display_name" in parsed and not isinstance(parsed["display_name"], str):
                    self.errors.append(f"Invalid display_name type in {file_path}:{line_num}")
                
                if "name" in parsed and not isinstance(parsed["name"], str):
                    self.errors.append(f"Invalid name type in {file_path}:{line_num}")
            
            return True
            
        except json.JSONDecodeError as e:
            self.errors.append(f"Invalid JSON in {file_path}:{line_num}: {e}")
            return False
    
    def validate_rust_code(self, rust_content: str, file_path: Path, line_num: int) -> bool:
        """Basic Rust syntax validation"""
        # Check for common syntax issues
        issues = []
        
        # Check for unmatched braces
        brace_count = rust_content.count('{') - rust_content.count('}')
        if brace_count != 0:
            issues.append(f"Unmatched braces (difference: {brace_count})")
        
        # Check for unmatched parentheses
        paren_count = rust_content.count('(') - rust_content.count(')')
        if paren_count != 0:
            issues.append(f"Unmatched parentheses (difference: {paren_count})")
        
        # Check for basic Rust keywords and patterns
        if 'pub trait' in rust_content and not rust_content.strip().endswith('}'):
            if 'async fn' in rust_content and not ';' in rust_content:
                issues.append("Trait with async functions should end with semicolon or implementation")
        
        if issues:
            for issue in issues:
                self.warnings.append(f"Rust syntax warning in {file_path}:{line_num}: {issue}")
            return False
        
        return True
    
    def validate_bash_commands(self, bash_content: str, file_path: Path, line_num: int) -> bool:
        """Validate bash commands for common issues"""
        lines = bash_content.strip().split('\n')
        
        for i, line in enumerate(lines):
            line = line.strip()
            if not line or line.startswith('#'):
                continue
            
            # Check for dangerous commands
            dangerous_patterns = ['rm -rf /', 'sudo rm', 'format c:', 'del /s']
            for pattern in dangerous_patterns:
                if pattern in line.lower():
                    self.warnings.append(f"Potentially dangerous command in {file_path}:{line_num + i}: {line}")
            
            # Check for cargo commands that should exist
            if line.startswith('cargo '):
                cmd_parts = line.split()
                if len(cmd_parts) >= 2:
                    cargo_cmd = cmd_parts[1]
                    valid_cargo_cmds = ['build', 'run', 'test', 'check', 'fmt', 'clippy', 'clean', 'doc']
                    if cargo_cmd not in valid_cargo_cmds:
                        self.warnings.append(f"Unknown cargo command in {file_path}:{line_num + i}: {cargo_cmd}")
        
        return True
    
    def validate_yaml_content(self, yaml_content: str, file_path: Path, line_num: int) -> bool:
        """Basic YAML validation"""
        try:
            import yaml
            yaml.safe_load(yaml_content)
            return True
        except ImportError:
            self.warnings.append(f"PyYAML not available for YAML validation in {file_path}:{line_num}")
            return True
        except yaml.YAMLError as e:
            self.errors.append(f"Invalid YAML in {file_path}:{line_num}: {e}")
            return False
    
    def validate_file(self, file_path: Path) -> Dict[str, int]:
        """Validate all code blocks in a file"""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
        except Exception as e:
            self.errors.append(f"Cannot read file {file_path}: {e}")
            return {"total": 0, "valid": 0, "invalid": 0}
        
        code_blocks = self.extract_code_blocks(content)
        total_blocks = len(code_blocks)
        valid_blocks = 0
        
        for lang, code_content, line_num in code_blocks:
            is_valid = True
            
            if lang.lower() in ['json']:
                if not self.validate_json_schema(code_content, file_path, line_num):
                    is_valid = False
            elif lang.lower() in ['rust', 'rs']:
                if not self.validate_rust_code(code_content, file_path, line_num):
                    is_valid = False
            elif lang.lower() in ['bash', 'sh', 'shell']:
                if not self.validate_bash_commands(code_content, file_path, line_num):
                    is_valid = False
            elif lang.lower() in ['yaml', 'yml']:
                if not self.validate_yaml_content(code_content, file_path, line_num):
                    is_valid = False
            
            if is_valid:
                valid_blocks += 1
        
        return {
            "total": total_blocks,
            "valid": valid_blocks,
            "invalid": total_blocks - valid_blocks
        }
    
    def check_schema_consistency(self) -> bool:
        """Check if JSON schemas match actual implementations"""
        # Look for actual tool implementations
        tool_files = list(self.root_dir.glob("rt-tools/src/**/mod.rs"))
        plugin_files = list(self.root_dir.glob("rt-plugin-*/src/main.rs"))
        
        schema_issues = []
        
        # Check if documented schemas exist in actual code
        for file_path in self.find_markdown_files():
            if 'DESIGN.md' in str(file_path) or 'USER_GUIDE.md' in str(file_path):
                try:
                    with open(file_path, 'r', encoding='utf-8') as f:
                        content = f.read()
                    
                    # Look for tool names in schemas
                    tool_name_pattern = r'"name":\s*"([^"]+)"'
                    matches = re.findall(tool_name_pattern, content)
                    
                    for tool_name in matches:
                        # Check if this tool actually exists
                        found = False
                        for impl_file in tool_files + plugin_files:
                            try:
                                with open(impl_file, 'r', encoding='utf-8') as f:
                                    impl_content = f.read()
                                if tool_name in impl_content:
                                    found = True
                                    break
                            except:
                                continue
                        
                        if not found:
                            schema_issues.append(f"Tool '{tool_name}' documented in {file_path} but not found in implementations")
                
                except Exception as e:
                    continue
        
        for issue in schema_issues:
            self.warnings.append(issue)
        
        return len(schema_issues) == 0
    
    def validate_all(self) -> Dict[str, Any]:
        """Validate all markdown files"""
        md_files = self.find_markdown_files()
        results = {}
        
        print(f"Validating code examples in {len(md_files)} markdown files...")
        
        for file_path in md_files:
            if 'target/' in str(file_path):  # Skip build artifacts
                continue
            print(f"Checking {file_path}...")
            results[str(file_path)] = self.validate_file(file_path)
        
        # Check schema consistency
        print("Checking schema consistency...")
        self.check_schema_consistency()
        
        return results
    
    def print_summary(self, results: Dict[str, Any]):
        """Print validation summary"""
        total_files = len(results)
        total_blocks = sum(r["total"] for r in results.values())
        total_valid = sum(r["valid"] for r in results.values())
        total_invalid = sum(r["invalid"] for r in results.values())
        
        print("\n" + "="*60)
        print("CODE EXAMPLES VALIDATION SUMMARY")
        print("="*60)
        print(f"Files checked: {total_files}")
        print(f"Total code blocks: {total_blocks}")
        print(f"Valid blocks: {total_valid}")
        print(f"Invalid blocks: {total_invalid}")
        
        if self.errors:
            print(f"\nERRORS ({len(self.errors)}):")
            for error in self.errors:
                print(f"  ❌ {error}")
        
        if self.warnings:
            print(f"\nWARNINGS ({len(self.warnings)}):")
            for warning in self.warnings:
                print(f"  ⚠️  {warning}")
        
        if not self.errors and not self.warnings:
            print("\n✅ All code examples and schemas are valid!")
        elif not self.errors:
            print(f"\n✅ No critical errors found! ({len(self.warnings)} warnings)")
        
        return len(self.errors) == 0

def main():
    validator = CodeValidator()
    results = validator.validate_all()
    success = validator.print_summary(results)
    
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()