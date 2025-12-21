#!/usr/bin/env python3
"""
Simple JSON validation for documentation
"""

import json
import re
import sys
from pathlib import Path

def extract_json_blocks(content):
    """Extract JSON code blocks from markdown"""
    pattern = r'```json\n(.*?)\n```'
    return re.findall(pattern, content, re.DOTALL)

def validate_json_blocks(file_path):
    """Validate all JSON blocks in a file"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
    except Exception as e:
        print(f"❌ Cannot read {file_path}: {e}")
        return False
    
    json_blocks = extract_json_blocks(content)
    all_valid = True
    
    for i, json_str in enumerate(json_blocks):
        try:
            json.loads(json_str)
            print(f"✅ {file_path} - JSON block {i+1}: Valid")
        except json.JSONDecodeError as e:
            print(f"❌ {file_path} - JSON block {i+1}: {e}")
            all_valid = False
    
    return all_valid

def main():
    # Key files to validate
    key_files = [
        "USER_GUIDE.md",
        "PLUGIN_GUIDE.md", 
        "rt-tools/src/text/ac_automaton/DESIGN.md",
        "rt-plugin-czkawka/README.md"
    ]
    
    all_valid = True
    
    for file_path in key_files:
        path = Path(file_path)
        if path.exists():
            if not validate_json_blocks(path):
                all_valid = False
        else:
            print(f"⚠️  File not found: {file_path}")
    
    if all_valid:
        print("\n🎉 All JSON blocks are valid!")
    else:
        print("\n💥 Some JSON blocks have errors!")
    
    return all_valid

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)