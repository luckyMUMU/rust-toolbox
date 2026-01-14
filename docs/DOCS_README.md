# Documentation Directory

> **Meta-guide for the `docs/` folder structure**  
> *Last Updated: 2026-01-14*

## 📂 Directory Structure

This folder contains all documentation for the Rust Workflow Toolkit, organized by purpose and audience.

```
docs/
├── INDEX.md                 # Main entry point for all documentation
├── DOCS_README.md           # This file (folder guide)
│
├── api/                     # Technical specifications
│   ├── CLI_REFERENCE.md     # Command-line interface reference
│   └── RUST_SDK_REFERENCE.md# Core library API reference
│
├── guides/                  # User manuals and guides
│   ├── USER_GUIDE.md        # Comprehensive user manual
│   ├── CHEATSHEET.md        # Quick reference
│   ├── TUTORIAL.md          # Step-by-step tutorial
│   ├── TROUBLESHOOTING.md   # Problem solving
│   └── MIGRATION_GUIDE.md   # Migration from previous versions
│
├── dev/                     # Developer resources
│   ├── DEVELOPMENT_GUIDE.md # Contributor guide
│   ├── PLUGIN_DEVELOPMENT.md# Extension development
│   ├── PROJECT_OVERVIEW.md  # Architecture and design
│   └── DOCUMENTATION_STANDARDS.md # Writing guidelines
│
└── plugins/                 # Plugin-specific documentation
    └── file_management/     # File management tool suite
        ├── FILE_MANAGEMENT_TOOLS_INDEX.md
        └── ...
```

## 📝 Writing Documentation

### Adding New Docs
1. **Identify Category:** Choose the appropriate subdirectory (`api`, `guides`, `dev`, `plugins`).
2. **Follow Standards:** See `dev/DOCUMENTATION_STANDARDS.md` (if available).
3. **Update Index:** Add a link to your new file in `INDEX.md`.
4. **Link Back:** Ensure your file has a link back to `INDEX.md` or its parent category.

### Style Guide
- **Clear & Concise:** Use direct language.
- **Code First:** Provide examples for every feature.
- **Cross-Link:** Link to related documents using relative paths (e.g., `[Guide](../guides/USER_GUIDE.md)`).
- **Metadata:** Include a "Last Updated" date at the top.

## 🔗 Key Links
- **[Main Index](INDEX.md)**: Start here for navigation.
- **[Project Root](../README.md)**: Return to the codebase root.
