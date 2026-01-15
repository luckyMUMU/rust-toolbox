import re
import sys
import json

def parse_rust_log(file_path):
    results = {}
    with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
        for line in f:
            # Match: [Experimental] Would move 'FOLDER' to category 'CATEGORY'
            match = re.search(r"\[Experimental\] Would move '(.*?)' to category '(.*?)'", line)
            if match:
                folder = match.group(1)
                category = match.group(2)
                results[folder] = category
            else:
                # Match unclassified
                match_un = re.search(r"\[Experimental\] Folder '(.*?)' status: Unclassified", line)
                if match_un:
                    results[match_un.group(1)] = "Unclassified"
                # Match ambiguous
                match_amb = re.search(r"\[Experimental\] Folder '(.*?)' status: Ambiguous", line)
                if match_amb:
                    results[match_amb.group(1)] = "Ambiguous"
    return results

def parse_python_log(file_path):
    results = {}
    with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
        for line in f:
            # Match: [实验模式] 文件夹: 'FOLDER' -> 分类: CATEGORY (分数
            match = re.search(r"文件夹: '(.*?)' -> .*?分类: (.*?) \(", line)
            if match:
                folder = match.group(1)
                category = match.group(2)
                results[folder] = category
            else:
                # Match auto select
                match_auto = re.search(r"文件夹: '(.*?)' -> 自动选择: (.*?) \(", line)
                if match_auto:
                    folder = match_auto.group(1)
                    category = match_auto.group(2)
                    results[folder] = category
                else:
                    # Unclassified might not be explicitly logged in the same format or I missed it
                    pass
    return results

def main():
    rust_results = parse_rust_log('rust_output.txt')
    python_results = parse_python_log('python_output.txt')

    common_folders = set(rust_results.keys()) & set(python_results.keys())
    only_rust = set(rust_results.keys()) - set(python_results.keys())
    only_python = set(python_results.keys()) - set(rust_results.keys())

    matches = 0
    mismatches = 0
    rust_unclassified = 0
    python_unclassified = 0 # If I could parse it
    
    details = []

    for folder in common_folders:
        r_cat = rust_results[folder]
        p_cat = python_results[folder]
        
        if r_cat == "Unclassified":
            rust_unclassified += 1
        
        if r_cat == p_cat:
            matches += 1
        else:
            mismatches += 1
            details.append({
                "folder": folder,
                "rust": r_cat,
                "python": p_cat
            })

    report = {
        "total_common_folders": len(common_folders),
        "matches": matches,
        "mismatches": mismatches,
        "match_rate": matches / len(common_folders) if common_folders else 0,
        "rust_unclassified_count": rust_unclassified,
        "only_in_rust_log": len(only_rust),
        "only_in_python_log": len(only_python),
        "mismatch_samples": details[:20]
    }

    print(json.dumps(report, indent=2, ensure_ascii=False))

    # Generate Markdown Report
    with open('comparison_report.md', 'w', encoding='utf-8') as f:
        f.write("# 分类效果对比报告\n\n")
        f.write("## 1. 概览\n")
        f.write(f"- **总文件夹数 (Common)**: {len(common_folders)}\n")
        f.write(f"- **完全匹配**: {matches} ({report['match_rate']*100:.2f}%)\n")
        f.write(f"- **不匹配**: {mismatches}\n")
        f.write(f"- **Rust 未分类**: {rust_unclassified}\n\n")
        
        f.write("## 2. 差异分析\n")
        f.write("Rust 工具由于为了避免崩溃而进行了关键词去重（全局唯一），导致部分规则失效。Python 脚本支持复杂的关键词组合和重复关键词（多分类匹配）。\n\n")
        
        f.write("### 差异示例 (前20个)\n")
        f.write("| 文件夹 | Rust 结果 | Python 结果 |\n")
        f.write("|---|---|---|\n")
        for d in details[:20]:
            f.write(f"| {d['folder']} | {d['rust']} | {d['python']} |\n")

if __name__ == "__main__":
    main()
