# File Management Tools Usage Examples

This document provides comprehensive usage examples for the File Management Tools workflow templates, demonstrating various scenarios from simple automation to complex enterprise workflows.

## Table of Contents

1. [Basic Usage Examples](#basic-usage-examples)
2. [Advanced Workflow Scenarios](#advanced-workflow-scenarios)
3. [Enterprise Use Cases](#enterprise-use-cases)
4. [Specialized Configurations](#specialized-configurations)
5. [Integration Examples](#integration-examples)
6. [Troubleshooting Scenarios](#troubleshooting-scenarios)

## Basic Usage Examples

### Example 1: Simple Folder Classification

**Scenario**: Organize a messy Downloads folder with basic classification rules.

```bash
# Create basic classification rules
cat > basic-rules.json << 'EOF'
{
  "categories": {
    "documents": {
      "keywords": [
        {"pattern": "doc", "weight": 1.0},
        {"pattern": "pdf", "weight": 1.0},
        {"pattern": "report", "weight": 1.2}
      ],
      "target_directory": "Documents"
    },
    "images": {
      "keywords": [
        {"pattern": "photo", "weight": 1.0},
        {"pattern": "image", "weight": 1.0},
        {"pattern": "pic", "weight": 0.8}
      ],
      "target_directory": "Images"
    }
  }
}
EOF

# Run classification in experimental mode
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/home/user/Downloads" \
  --param output_directory="/home/user/Organized" \
  --param classification_rules="basic-rules.json" \
  --param experimental_mode=true \
  --param batch_size=5
```

**Expected Output**:
```
📁 Scanning folders in /home/user/Downloads...
Found 23 folders to classify

🔄 Processing batch 1/5 (5 folders)...
✅ Classified: "Project Documents" → Documents (confidence: 0.85)
❓ Ambiguous: "Photo Collection 2024" → Multiple matches found

👤 Human Decision Required:
Folder: "Photo Collection 2024"
Options:
  1. Images (confidence: 0.72)
  2. Documents (confidence: 0.68)
  
Your choice [1-2]: 1

📊 Batch Results:
- Classified: 4/5 folders
- Requires review: 1/5 folders
- Processing time: 2.3s

❓ Experimental Mode: Review operations before execution?
[Y]es / [N]o / [M]odify rules: Y

⚡ Executing 4 file operations...
✅ All operations completed successfully
```

### Example 2: Folder Merging with User Decisions

**Scenario**: Merge duplicate folders across multiple locations.

```bash
# Run interactive merge workflow
cargo run -- workflow execute examples/templates/interactive-merge-workflow.yaml \
  --param source_directories='["/home/user/Desktop", "/home/user/Downloads", "/home/user/Documents/Temp"]' \
  --param target_directory="/home/user/Organized" \
  --param merge_strategy="UserDecision" \
  --param experimental_mode=true \
  --param enable_size_analysis=true
```

**Expected Output**:
```
🔍 Scanning directories for mergeable folders...
Found 3 groups of duplicate folders:

Group 1: "Projects" (3 locations)
- /home/user/Desktop/Projects (1.2 GB, 45 files)
- /home/user/Downloads/Projects (800 MB, 23 files)  
- /home/user/Documents/Temp/Projects (2.1 GB, 67 files)

👤 Merge Strategy Decision:
Group: "Projects"
Options:
  1. Merge smaller folders into largest (→ /home/user/Documents/Temp/Projects)
  2. Merge all to target directory (→ /home/user/Organized/Projects)
  3. Skip this group
  
Your choice [1-3]: 2

⚠️ Duplicate File Conflict Detected:
File: "project_plan.docx"
- Source 1: 2.1 MB, modified 2024-01-08 14:30
- Source 2: 1.8 MB, modified 2024-01-05 09:15

Resolution options:
  1. Keep newer file (Source 1)
  2. Keep larger file (Source 1)  
  3. Rename and keep both
  
Your choice [1-3]: 3

📋 Final Merge Plan:
- Projects: Merge 3 folders → /home/user/Organized/Projects
- Estimated space savings: 1.2 GB
- Total files to merge: 135
- Conflicts resolved: 3 files renamed

❓ Execute merge operations? [Y/N]: Y
```

### Example 3: Batch File Processing

**Scenario**: Process a large number of files with conflict resolution.

```bash
# Run batch processing workflow
cargo run -- workflow execute examples/templates/interactive-batch-processing-workflow.yaml \
  --param source_directory="/data/incoming" \
  --param target_directory="/data/processed" \
  --param operation_type="move" \
  --param batch_size=20 \
  --param max_concurrent_batches=3 \
  --param conflict_resolution="UserDecision" \
  --param experimental_mode=false
```

**Expected Output**:
```
📊 Analyzing source directory: /data/incoming
Found 1,247 items to process

📋 Batch Processing Plan:
- Total batches: 63
- Concurrent batches: 3
- Estimated duration: 15-20 minutes
- Potential conflicts: 12 items

⚡ Starting batch processing...

Batch 1/63: Processing items 1-20... ✅ Completed (18.2s)
Batch 2/63: Processing items 21-40... ✅ Completed (16.7s)
Batch 3/63: Processing items 41-60... ⚠️ Conflict detected

👤 Conflict Resolution Required:
Item: "report_2024.pdf"
Conflict: Target file already exists
- Source: 2.3 MB, modified 2024-01-08
- Target: 2.1 MB, modified 2024-01-07

Options:
  1. Skip this file
  2. Rename source file (report_2024_1.pdf)
  3. Overwrite target file
  
Your choice [1-3]: 2
Apply to similar conflicts? [Y/N]: Y

📊 Progress: 60/1,247 items processed (4.8%)
Estimated time remaining: 16 minutes
```

## Advanced Workflow Scenarios

### Example 4: Multi-Stage Classification with Learning

**Scenario**: Implement a learning classification system that improves over time.

```bash
# Create adaptive classification rules
cat > adaptive-rules.json << 'EOF'
{
  "version": "2.0",
  "learning_enabled": true,
  "categories": {
    "work_documents": {
      "keywords": [
        {"pattern": "work", "weight": 1.0},
        {"pattern": "office", "weight": 0.9},
        {"pattern": "business", "weight": 1.1}
      ],
      "target_directory": "Work/Documents",
      "learning_weight": 1.2
    },
    "personal_files": {
      "keywords": [
        {"pattern": "personal", "weight": 1.0},
        {"pattern": "family", "weight": 0.8},
        {"pattern": "home", "weight": 0.7}
      ],
      "target_directory": "Personal",
      "learning_weight": 1.0
    }
  },
  "learning_config": {
    "track_decisions": true,
    "update_weights": true,
    "confidence_improvement_target": 0.1
  }
}
EOF

# Run learning-enabled classification
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/home/user/Mixed_Files" \
  --param output_directory="/home/user/Organized" \
  --param classification_rules="adaptive-rules.json" \
  --param learning_enabled=true \
  --param track_decisions=true \
  --param experimental_mode=true
```

**Expected Output**:
```
🧠 Loading decision history... Found 127 previous decisions
📈 Analyzing patterns... Confidence improved by 0.08 since last run

🔄 Processing with learned preferences...
✅ Auto-classified: "Work Reports 2024" → Work/Documents (confidence: 0.92)
   Reason: Learned from 15 similar decisions

❓ Learning Decision Required:
Folder: "Family Photos Summer"
System recommendation: Personal (confidence: 0.65)
Based on: Previous decisions for "Family" folders

Options:
  1. Personal (recommended based on learning)
  2. Images (alternative classification)
  3. Custom location
  
Your choice [1-3]: 1

🧠 Learning Update: Strengthened "family" → "Personal" association
📊 New confidence for similar folders: 0.78 (+0.13)

📋 Session Summary:
- Auto-classified: 23/30 folders (77%)
- Human decisions: 7/30 folders (23%)
- Learning improvements: 5 pattern updates
- Average confidence increase: +0.11
```

### Example 5: Enterprise Compliance Workflow

**Scenario**: Process files with compliance requirements and audit trails.

```bash
# Create compliance configuration
cat > compliance-config.yaml << 'EOF'
compliance:
  framework: "SOX"
  data_classification: true
  audit_trail: true
  retention_policies: true
  
security:
  encrypt_sensitive: true
  access_control: true
  dual_approval_threshold: "high_risk"
  
governance:
  require_approval: true
  escalation_hierarchy:
    - "team_lead"
    - "compliance_officer"
    - "legal_department"
EOF

# Run compliance-aware processing
cargo run -- workflow execute examples/templates/interactive-batch-processing-workflow.yaml \
  --param source_directory="/enterprise/incoming" \
  --param target_directory="/enterprise/classified" \
  --param compliance_config="compliance-config.yaml" \
  --param governance_mode=true \
  --param audit_enabled=true \
  --param experimental_mode=true
```

**Expected Output**:
```
🔒 Compliance Mode: SOX Framework Active
🔍 Scanning for sensitive data... 

⚠️ Sensitive Data Detected:
- 15 files contain PII (Personally Identifiable Information)
- 8 files contain financial data
- 3 files marked as confidential

📋 Compliance Assessment:
- High Risk Operations: 12 items
- Medium Risk Operations: 45 items  
- Low Risk Operations: 234 items

👤 Governance Approval Required:
Risk Level: HIGH
Affected Items: 12 files with financial data
Compliance Impact: SOX Section 404 requirements
Retention Period: 7 years

Approval Options:
  1. Approve with standard controls
  2. Approve with enhanced controls
  3. Escalate to compliance officer
  4. Reject operations

Your choice [1-4]: 2

🔐 Enhanced Controls Applied:
- Encryption: AES-256 enabled
- Access logging: Detailed audit trail
- Dual approval: Required for modifications
- Retention: 7-year automatic retention

📊 Compliance Report Generated:
- Audit ID: COMP-2024-0108-001
- Operations: 291 items processed
- Risk Mitigations: 15 controls applied
- Compliance Status: APPROVED
```

### Example 6: Multi-Language File Organization

**Scenario**: Organize files with Chinese and English names using advanced text processing.

```bash
# Create multilingual classification rules
cat > multilingual-rules.json << 'EOF'
{
  "version": "1.0",
  "settings": {
    "enable_chinese_processing": true,
    "enable_pinyin_conversion": true,
    "unicode_normalization": "NFC"
  },
  "categories": {
    "documents": {
      "keywords": [
        {"pattern": "文档", "weight": 1.0, "language": "zh"},
        {"pattern": "document", "weight": 1.0, "language": "en"},
        {"pattern": "wendang", "weight": 0.9, "type": "pinyin"}
      ],
      "target_directory": "文档_Documents"
    },
    "photos": {
      "keywords": [
        {"pattern": "照片", "weight": 1.0, "language": "zh"},
        {"pattern": "photo", "weight": 1.0, "language": "en"},
        {"pattern": "zhaopian", "weight": 0.9, "type": "pinyin"}
      ],
      "target_directory": "照片_Photos"
    }
  }
}
EOF

# Run multilingual classification
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/home/user/混合文件夹" \
  --param output_directory="/home/user/整理后" \
  --param classification_rules="multilingual-rules.json" \
  --param enable_chinese_processing=true \
  --param experimental_mode=true
```

**Expected Output**:
```
🌐 Multilingual Processing Enabled
🔤 Text Processing: Chinese + English + Pinyin

📁 Processing folder: "工作文档2024"
🔄 Text analysis:
- Original: "工作文档2024"
- Simplified: "工作文档2024"
- Pinyin: "gongzuo wendang 2024"
- Keywords found: ["工作", "文档", "gongzuo", "wendang"]

✅ Classification: 文档_Documents (confidence: 0.94)
   Matched: "文档" (weight: 1.0) + "wendang" (weight: 0.9)

📁 Processing folder: "Family Photos 家庭照片"
🔄 Text analysis:
- Mixed language detected: English + Chinese
- English part: "Family Photos"
- Chinese part: "家庭照片"
- Combined keywords: ["family", "photos", "家庭", "照片"]

✅ Classification: 照片_Photos (confidence: 0.96)
   Matched: "photos" (weight: 1.0) + "照片" (weight: 1.0)

📊 Multilingual Statistics:
- Chinese folders: 15
- English folders: 8
- Mixed language: 4
- Pinyin matches: 12
- Average confidence: 0.87
```

## Enterprise Use Cases

### Example 7: Large-Scale Data Migration

**Scenario**: Migrate and organize 100,000+ files from legacy systems.

```bash
# Create migration configuration
cat > migration-config.yaml << 'EOF'
migration:
  source_systems:
    - "/legacy/fileserver1"
    - "/legacy/fileserver2"
    - "/legacy/sharepoint_export"
  
  target_structure:
    base_path: "/enterprise/organized"
    preserve_timestamps: true
    maintain_permissions: true
  
  performance:
    batch_size: 500
    max_concurrent_batches: 10
    streaming_mode: true
    memory_limit: "16GB"
  
  validation:
    verify_checksums: true
    validate_file_integrity: true
    create_migration_report: true
EOF

# Run large-scale migration
cargo run -- workflow execute examples/templates/interactive-batch-processing-workflow.yaml \
  --config migration-config.yaml \
  --param operation_type="migrate" \
  --param enable_progress_reporting=true \
  --param create_backup=true \
  --param experimental_mode=false
```

**Expected Output**:
```
🚀 Large-Scale Migration Started
📊 Source Analysis:
- Total files: 127,543
- Total size: 2.3 TB
- Estimated duration: 4-6 hours

📋 Migration Plan:
- Batches: 256 (500 files each)
- Concurrent processing: 10 batches
- Backup creation: Enabled
- Integrity verification: Enabled

⚡ Migration Progress:
[████████████████████████████████████████] 100% (127,543/127,543)

Batch 001/256: ✅ 500 files (1.2 GB) - 45.2s
Batch 002/256: ✅ 500 files (1.1 GB) - 43.8s
Batch 003/256: ✅ 500 files (1.3 GB) - 47.1s
...
Batch 256/256: ✅ 43 files (0.2 GB) - 12.3s

📊 Migration Summary:
- Files processed: 127,543
- Data migrated: 2.3 TB
- Duration: 4h 23m 17s
- Average speed: 147 MB/s
- Errors: 0
- Verification: 100% passed

📋 Migration Report: migration_report_20240108.pdf
```

### Example 8: Automated Compliance Monitoring

**Scenario**: Continuous monitoring and organization of files for regulatory compliance.

```bash
# Create compliance monitoring workflow
cat > compliance-monitor.yaml << 'EOF'
name: "compliance_monitoring"
description: "Continuous compliance monitoring and file organization"

schedule:
  frequency: "daily"
  time: "02:00"
  
parameters:
  watch_directories:
    - "/enterprise/shared"
    - "/departments/*/documents"
  compliance_rules: "enterprise-compliance.json"
  retention_policies: "retention-policies.json"
  
steps:
  - name: "scan_new_files"
    tool: "file-scanner"
    params:
      directories: "{{ watch_directories }}"
      since_last_run: true
      
  - name: "classify_compliance_level"
    tool: "compliance-classifier"
    params:
      files: "{{ scan_new_files.new_files }}"
      rules: "{{ compliance_rules }}"
      
  - name: "apply_retention_policies"
    tool: "retention-manager"
    params:
      classified_files: "{{ classify_compliance_level.results }}"
      policies: "{{ retention_policies }}"
      
  - name: "generate_compliance_report"
    tool: "compliance-reporter"
    params:
      processed_files: "{{ apply_retention_policies.results }}"
      report_format: "regulatory"
EOF

# Schedule compliance monitoring
cargo run -- workflow schedule compliance-monitor.yaml
```

**Expected Output**:
```
📅 Compliance Monitoring Scheduled
⏰ Next run: 2024-01-09 02:00:00

🔍 Daily Compliance Scan (2024-01-08 02:00:15)
📁 Scanning directories...
- /enterprise/shared: 1,247 files scanned
- /departments/finance/documents: 234 files scanned
- /departments/hr/documents: 156 files scanned
- /departments/legal/documents: 89 files scanned

🏷️ Compliance Classification:
- Public: 1,156 files
- Internal: 445 files  
- Confidential: 98 files
- Restricted: 27 files

📋 Retention Policy Application:
- Immediate retention: 27 files (Restricted)
- 7-year retention: 98 files (Confidential)
- 3-year retention: 445 files (Internal)
- Standard retention: 1,156 files (Public)

⚠️ Compliance Alerts:
- 3 files exceed retention period (auto-archived)
- 1 file missing classification (escalated)
- 0 policy violations detected

📊 Compliance Report Generated:
- Report ID: COMP-DAILY-20240108
- Files processed: 1,726
- Policy violations: 0
- Actions taken: 4
- Compliance score: 99.94%
```

## Specialized Configurations

### Example 9: Media File Organization

**Scenario**: Organize large media collections with metadata-based classification.

```bash
# Create media-specific rules
cat > media-rules.json << 'EOF'
{
  "categories": {
    "photos_by_year": {
      "keywords": [
        {"pattern": "photo", "weight": 1.0},
        {"pattern": "img", "weight": 0.9},
        {"pattern": "pic", "weight": 0.8}
      ],
      "target_directory": "Photos/{year}",
      "metadata_extraction": true,
      "date_based_organization": true
    },
    "videos_by_type": {
      "keywords": [
        {"pattern": "video", "weight": 1.0},
        {"pattern": "movie", "weight": 1.1},
        {"pattern": "clip", "weight": 0.9}
      ],
      "target_directory": "Videos/{type}",
      "analyze_video_content": true
    }
  },
  "metadata_config": {
    "extract_exif": true,
    "analyze_content": true,
    "detect_faces": false,
    "extract_location": true
  }
}
EOF

# Run media organization
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/media/unsorted" \
  --param output_directory="/media/organized" \
  --param classification_rules="media-rules.json" \
  --param enable_metadata_extraction=true \
  --param batch_size=10
```

**Expected Output**:
```
📸 Media Organization Mode
🔍 Analyzing media files...

📁 Processing: "Vacation Photos Summer 2023"
📊 Metadata Analysis:
- File types: JPG (45), PNG (3), RAW (12)
- Date range: 2023-07-15 to 2023-08-20
- Location data: Available (GPS coordinates)
- Average file size: 8.2 MB

✅ Classification: Photos/2023/Summer_Vacation
   Organization: By year and season
   Metadata: EXIF data preserved

📁 Processing: "Family Movie Night"
📊 Content Analysis:
- Video format: MP4, H.264
- Duration: 2h 15m
- Resolution: 1920x1080
- Audio: Stereo, AAC

✅ Classification: Videos/Family/Movies
   Organization: By type and category
   Metadata: Video properties preserved

📊 Media Organization Summary:
- Photos organized: 1,247 files → 15 year/season folders
- Videos organized: 89 files → 8 category folders
- Metadata preserved: 100%
- Duplicate detection: 23 duplicates found and handled
- Storage optimization: 15% space saved through deduplication
```

### Example 10: Development Project Organization

**Scenario**: Organize software development projects and repositories.

```bash
# Create development-specific rules
cat > dev-rules.json << 'EOF'
{
  "categories": {
    "active_projects": {
      "keywords": [
        {"pattern": "project", "weight": 1.0},
        {"pattern": "src", "weight": 1.2},
        {"pattern": "code", "weight": 1.1}
      ],
      "target_directory": "Development/Active",
      "detect_git_repos": true,
      "analyze_project_structure": true
    },
    "archived_projects": {
      "keywords": [
        {"pattern": "old", "weight": 1.0},
        {"pattern": "archive", "weight": 1.2},
        {"pattern": "backup", "weight": 1.1}
      ],
      "target_directory": "Development/Archive",
      "compress_archives": true
    },
    "documentation": {
      "keywords": [
        {"pattern": "doc", "weight": 1.0},
        {"pattern": "readme", "weight": 1.2},
        {"pattern": "wiki", "weight": 1.1}
      ],
      "target_directory": "Development/Documentation"
    }
  },
  "development_config": {
    "detect_languages": true,
    "analyze_dependencies": true,
    "check_git_status": true,
    "preserve_git_history": true
  }
}
EOF

# Run development project organization
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/home/developer/workspace" \
  --param output_directory="/home/developer/organized" \
  --param classification_rules="dev-rules.json" \
  --param analyze_project_structure=true \
  --param preserve_git_repos=true
```

**Expected Output**:
```
💻 Development Project Organization
🔍 Analyzing project structure...

📁 Processing: "web-app-2024"
🔧 Project Analysis:
- Git repository: Active (15 commits ahead)
- Languages: TypeScript (65%), CSS (20%), HTML (15%)
- Dependencies: package.json, yarn.lock
- Last commit: 2024-01-07 16:30
- Branch: feature/user-auth

✅ Classification: Development/Active/WebApps/web-app-2024
   Git status: Preserved with full history
   Dependencies: Analyzed and documented

📁 Processing: "old-php-project-backup"
🔧 Project Analysis:
- Git repository: None
- Languages: PHP (80%), JavaScript (15%), CSS (5%)
- Last modified: 2022-03-15
- Size: 45 MB

❓ Archive Decision Required:
Project: "old-php-project-backup"
Analysis: Inactive for 22 months, no recent commits
Recommendation: Archive with compression

Options:
  1. Archive and compress (recommended)
  2. Keep in active projects
  3. Delete after backup
  
Your choice [1-3]: 1

✅ Classification: Development/Archive/PHP/old-php-project-backup.tar.gz
   Compression: 45 MB → 12 MB (73% reduction)
   Backup: Created before archiving

📊 Development Organization Summary:
- Active projects: 12 (preserved Git history)
- Archived projects: 8 (compressed)
- Documentation: 25 folders
- Languages detected: TypeScript, JavaScript, Python, PHP, Go
- Total space saved: 234 MB through compression
- Git repositories: 12 preserved, 3 archived
```

## Integration Examples

### Example 11: CI/CD Pipeline Integration

**Scenario**: Integrate file organization into continuous integration workflows.

```bash
# Create CI/CD integration script
cat > .github/workflows/organize-artifacts.yml << 'EOF'
name: Organize Build Artifacts

on:
  workflow_run:
    workflows: ["Build and Test"]
    types: [completed]

jobs:
  organize-artifacts:
    runs-on: ubuntu-latest
    steps:
      - name: Download artifacts
        uses: actions/download-artifact@v3
        
      - name: Setup Workflow Toolkit
        run: |
          curl -L https://github.com/workflow-toolkit/releases/latest/download/workflow-toolkit-linux.tar.gz | tar xz
          chmod +x workflow-toolkit
          
      - name: Organize build artifacts
        run: |
          ./workflow-toolkit workflow execute organize-artifacts.yaml \
            --param source_directory="./artifacts" \
            --param target_directory="./organized-artifacts" \
            --param build_number="${{ github.run_number }}" \
            --param branch_name="${{ github.ref_name }}"
            
      - name: Upload organized artifacts
        uses: actions/upload-artifact@v3
        with:
          name: organized-artifacts
          path: ./organized-artifacts
EOF

# Create artifact organization workflow
cat > organize-artifacts.yaml << 'EOF'
name: "organize_build_artifacts"
description: "Organize CI/CD build artifacts by type and branch"

parameters:
  source_directory: "./artifacts"
  target_directory: "./organized-artifacts"
  build_number: "{{ env.BUILD_NUMBER }}"
  branch_name: "{{ env.BRANCH_NAME }}"

steps:
  - name: "classify_artifacts"
    tool: "folder-classifier"
    params:
      folder_path: "{{ source_directory }}"
      classification_rules:
        categories:
          binaries:
            keywords: [{"pattern": "bin", "weight": 1.0}]
            target_directory: "{{ target_directory }}/{{ branch_name }}/build-{{ build_number }}/binaries"
          documentation:
            keywords: [{"pattern": "doc", "weight": 1.0}]
            target_directory: "{{ target_directory }}/{{ branch_name }}/build-{{ build_number }}/docs"
          test_results:
            keywords: [{"pattern": "test", "weight": 1.0}]
            target_directory: "{{ target_directory }}/{{ branch_name }}/build-{{ build_number }}/tests"
      experimental_mode: false
      
  - name: "move_artifacts"
    tool: "file-mover"
    params:
      operations: "{{ classify_artifacts.operations }}"
      create_directories: true
      conflict_resolution: "Rename"
EOF
```

### Example 12: Database Integration

**Scenario**: Integrate with database systems for metadata tracking.

```bash
# Create database integration configuration
cat > db-integration.yaml << 'EOF'
database:
  type: "postgresql"
  connection: "postgresql://user:pass@localhost/filemanagement"
  
tracking:
  track_operations: true
  store_metadata: true
  maintain_history: true
  
tables:
  file_operations:
    - operation_id
    - source_path
    - target_path
    - operation_type
    - timestamp
    - user_id
    - status
  
  classification_history:
    - classification_id
    - folder_path
    - category
    - confidence_score
    - decision_type
    - timestamp
EOF

# Run with database integration
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml \
  --param source_directory="/data/incoming" \
  --param output_directory="/data/organized" \
  --param classification_rules="rules.json" \
  --param database_config="db-integration.yaml" \
  --param track_in_database=true
```

## Troubleshooting Scenarios

### Example 13: Recovery from Failed Operations

**Scenario**: Recover from a failed batch operation and resume processing.

```bash
# Check workflow state after failure
cargo run -- workflow status --workflow-id batch-process-20240108

# Resume from last checkpoint
cargo run -- workflow resume --workflow-id batch-process-20240108 \
  --from-checkpoint checkpoint-batch-45

# Alternative: Rollback and restart
cargo run -- workflow rollback --workflow-id batch-process-20240108 \
  --to-checkpoint checkpoint-batch-40

cargo run -- workflow restart --workflow-id batch-process-20240108 \
  --param batch_size=10 \
  --param max_concurrent_batches=2
```

**Expected Output**:
```
🔍 Workflow Status: batch-process-20240108
Status: FAILED
Last checkpoint: checkpoint-batch-45
Completed batches: 45/67
Failed batch: 46 (network timeout)
Remaining items: 440

🔄 Resuming from checkpoint-batch-45...
📊 Resume Analysis:
- Completed operations: 900 items
- Remaining operations: 440 items
- Failed operations: 20 items (will retry)
- Estimated completion: 15 minutes

⚡ Resuming batch processing...
Batch 46/67: ✅ Completed (retry successful)
Batch 47/67: ✅ Completed
...
```

### Example 14: Performance Optimization

**Scenario**: Optimize performance for large-scale operations.

```bash
# Run performance analysis
cargo run -- workflow analyze-performance \
  --workflow-file examples/templates/interactive-batch-processing-workflow.yaml \
  --param source_directory="/large-dataset" \
  --param target_directory="/processed" \
  --param enable_profiling=true

# Apply optimized configuration
cargo run -- workflow execute examples/templates/interactive-batch-processing-workflow.yaml \
  --param source_directory="/large-dataset" \
  --param target_directory="/processed" \
  --param batch_size=100 \
  --param max_concurrent_batches=8 \
  --param streaming_mode=true \
  --param memory_limit="8GB" \
  --param enable_compression=true
```

**Expected Output**:
```
📊 Performance Analysis Results:
Current configuration:
- Batch size: 20 (suboptimal)
- Concurrency: 3 (underutilized)
- Memory usage: 45% (room for improvement)
- I/O throughput: 67 MB/s (can be improved)

🚀 Optimization Recommendations:
- Increase batch size to 100 (+150% throughput)
- Increase concurrency to 8 (+167% parallelism)
- Enable streaming mode (-60% memory usage)
- Use compression (+25% I/O efficiency)

Estimated improvements:
- Processing time: 45 minutes → 18 minutes (-60%)
- Memory usage: 4.2 GB → 1.7 GB (-60%)
- I/O throughput: 67 MB/s → 156 MB/s (+133%)
```

These comprehensive usage examples demonstrate the flexibility and power of the File Management Tools workflow templates across various scenarios, from simple personal file organization to complex enterprise workflows with compliance requirements.