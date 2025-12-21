#!/usr/bin/env python3
"""
Documentation Link Validation Script
Validates all internal and external links in markdown files
"""

import os
import re
import sys
import urllib.request
import urllib.error
from pathlib import Path
from typing import List, Tuple, Dict

class LinkValidator:
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
    
    def extract_links(self, content: str) -> List[Tuple[str, str]]:
        """Extract all markdown links from content"""
        # Pattern for [text](url) format
        link_pattern = r'\[([^\]]*)\]\(([^)]+)\)'
        return re.findall(link_pattern, content)
    
    def validate_internal_link(self, file_path: Path, link_url: str) -> bool:
        """Validate internal file links"""
        # Handle relative paths
        if link_url.startswith('./') or not link_url.startswith('http'):
            # Remove anchor fragments
            clean_url = link_url.split('#')[0]
            if not clean_url:  # Just an anchor
                return True
                
            # Resolve relative to the current file's directory
            target_path = (file_path.parent / clean_url).resolve()
            
            # Check if target exists
            if not target_path.exists():
                self.errors.append(f"Broken internal link in {file_path}: {link_url} -> {target_path}")
                return False
        return True
    
    def validate_external_link(self, file_path: Path, link_url: str) -> bool:
        """Validate external HTTP links"""
        if link_url.startswith('http'):
            try:
                req = urllib.request.Request(link_url, headers={'User-Agent': 'Mozilla/5.0'})
                with urllib.request.urlopen(req, timeout=10) as response:
                    if response.status >= 400:
                        self.errors.append(f"External link error in {file_path}: {link_url} (Status: {response.status})")
                        return False
            except urllib.error.URLError as e:
                self.warnings.append(f"External link warning in {file_path}: {link_url} ({e})")
                return False
            except Exception as e:
                self.warnings.append(f"External link check failed in {file_path}: {link_url} ({e})")
                return False
        return True
    
    def validate_file(self, file_path: Path) -> Dict[str, int]:
        """Validate all links in a single file"""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
        except Exception as e:
            self.errors.append(f"Cannot read file {file_path}: {e}")
            return {"total": 0, "valid": 0, "invalid": 0}
        
        links = self.extract_links(content)
        total_links = len(links)
        valid_links = 0
        
        for link_text, link_url in links:
            is_valid = True
            
            # Validate internal links
            if not self.validate_internal_link(file_path, link_url):
                is_valid = False
            
            # Validate external links (with timeout protection)
            if is_valid and not self.validate_external_link(file_path, link_url):
                is_valid = False
            
            if is_valid:
                valid_links += 1
        
        return {
            "total": total_links,
            "valid": valid_links, 
            "invalid": total_links - valid_links
        }
    
    def validate_all(self) -> Dict[str, any]:
        """Validate all markdown files"""
        md_files = self.find_markdown_files()
        results = {}
        
        print(f"Validating links in {len(md_files)} markdown files...")
        
        for file_path in md_files:
            print(f"Checking {file_path}...")
            results[str(file_path)] = self.validate_file(file_path)
        
        return results
    
    def print_summary(self, results: Dict[str, any]):
        """Print validation summary"""
        total_files = len(results)
        total_links = sum(r["total"] for r in results.values())
        total_valid = sum(r["valid"] for r in results.values())
        total_invalid = sum(r["invalid"] for r in results.values())
        
        print("\n" + "="*60)
        print("LINK VALIDATION SUMMARY")
        print("="*60)
        print(f"Files checked: {total_files}")
        print(f"Total links: {total_links}")
        print(f"Valid links: {total_valid}")
        print(f"Invalid links: {total_invalid}")
        
        if self.errors:
            print(f"\nERRORS ({len(self.errors)}):")
            for error in self.errors:
                print(f"  ❌ {error}")
        
        if self.warnings:
            print(f"\nWARNINGS ({len(self.warnings)}):")
            for warning in self.warnings:
                print(f"  ⚠️  {warning}")
        
        if not self.errors and not self.warnings:
            print("\n✅ All links are valid!")
        
        return len(self.errors) == 0

def main():
    validator = LinkValidator()
    results = validator.validate_all()
    success = validator.print_summary(results)
    
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()