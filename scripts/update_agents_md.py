import os
import re
from pathlib import Path
from datetime import datetime

# Configuration
AGENT_FILENAMES = ["AGENT.md", "AGENTS.md"]
MARKER_START = "<!-- AUTO-GENERATED-AGENT-MAP:START -->"
MARKER_END = "<!-- AUTO-GENERATED-AGENT-MAP:END -->"
SECTION_TITLE = "## 🗺️ Agent Map & Directory Structure"

def find_agent_file(directory):
    """Finds the AGENT.md or AGENTS.md file in a directory."""
    for filename in AGENT_FILENAMES:
        path = directory / filename
        if path.exists():
            return path
    return None

def get_file_summary(file_path):
    """Extracts a summary from an AGENT.md file."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
            
        # Try to find OVERVIEW section
        overview_match = re.search(r'## OVERVIEW\s+(.*?)(?=\n##|\Z)', content, re.DOTALL)
        if overview_match:
            return overview_match.group(1).strip().split('\n')[0]  # First line of overview
            
        # Try to find first paragraph after title
        lines = content.split('\n')
        for line in lines:
            if line.strip() and not line.startswith('#') and not line.startswith('>'):
                return line.strip()
                
        return "No summary available."
    except Exception:
        return "Error reading summary."

def get_dir_description(directory):
    """Generates a description for a directory without an AGENT.md."""
    files = [f for f in os.listdir(directory) if os.path.isfile(directory / f)]
    subdirs = [d for d in os.listdir(directory) if os.path.isdir(directory / d)]
    
    important_files = [f for f in files if f.endswith('.rs') or f.endswith('.py') or f.endswith('.md')]
    
    if not important_files and not subdirs:
        return "Empty or asset-only directory."
        
    desc = []
    if important_files:
        desc.append(f"Contains {len(important_files)} files (e.g., {', '.join(important_files[:3])})")
    if subdirs:
        desc.append(f"Has {len(subdirs)} subdirectories")
        
    return ". ".join(desc) + "."

def update_agent_file(file_path, content_to_inject):
    """Updates the agent file with the generated content."""
    with open(file_path, 'r', encoding='utf-8') as f:
        original_content = f.read()

    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    header = f"{SECTION_TITLE}\n\n> **Auto-generated** on {timestamp}\n\n"
    full_injection = f"{MARKER_START}\n{header}{content_to_inject}\n{MARKER_END}"

    if MARKER_START in original_content:
        # Replace existing block
        pattern = re.compile(f"{re.escape(MARKER_START)}.*?{re.escape(MARKER_END)}", re.DOTALL)
        new_content = pattern.sub(full_injection, original_content)
    else:
        # Append to end
        new_content = f"{original_content.rstrip()}\n\n{full_injection}\n"

    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(new_content)
    print(f"Updated {file_path}")

def process_directory(current_dir):
    """Recursively processes directories."""
    current_path = Path(current_dir)
    agent_file = find_agent_file(current_path)
    
    # Collect child info regardless of whether current dir has agent file
    # (because we might be in root which definitely needs it)
    
    child_dirs = [d for d in os.listdir(current_path) if os.path.isdir(current_path / d)]
    child_dirs.sort()
    
    map_content = ""
    
    has_sub_agents = False
    
    for child in child_dirs:
        if child.startswith('.') or child == 'target' or child == 'node_modules':
            continue
            
        child_path = current_path / child
        child_agent = find_agent_file(child_path)
        
        if child_agent:
            has_sub_agents = True
            summary = get_file_summary(child_agent)
            link_path = child_agent.relative_to(current_path).as_posix()
            map_content += f"- **[{child}/]({link_path})**: {summary}\n"
            # Recursively process the child
            process_directory(child_path)
        else:
            # Check if this subdir has interesting stuff
            desc = get_dir_description(child_path)
            map_content += f"- **{child}/**: {desc}\n"
            # We also might want to recurse if we want to update deep agent files even if intermediate ones are missing?
            # But the requirement implies hierarchy. If intermediate is missing, we just list it. 
            # We still need to recurse to find deeper AGENT.md files to update THEM.
            process_directory(child_path)

    if agent_file and map_content:
        update_agent_file(agent_file, map_content)

if __name__ == "__main__":
    root_dir = os.getcwd()
    print(f"Scanning from {root_dir}...")
    process_directory(root_dir)
    print("Done.")
