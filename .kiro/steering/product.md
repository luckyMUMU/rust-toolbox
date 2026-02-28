# Product Overview

Workflow Toolkit is a Rust-based workflow execution system that serves as a tool capability orchestration and reuse platform. It enables both AI agents and human users to discover, compose, and execute tools through workflow orchestration.

## Core Value Propositions

- **Tool Orchestration & Reuse**: Define workflows once, reuse multiple times. Fixed processing flows are solidified into reusable units.
- **AI Agent Capability Extension**: Dynamic orchestration allows AI agents to break through single-tool limitations and build complex functionality by composing tools.
- **Dual-Mode Access**: Supports both AI agent access (via MCP protocol) and human user access (via CLI/TUI interfaces).
- **Multi-Language Plugin System**: Extend capabilities using Native (Rust), Python, Node.js, Docker, or WASM plugins.
- **High Performance**: Built on Rust + Tokio async runtime with microsecond-level scheduling latency, supporting 100+ concurrent workflows on a single machine.

## Key Use Cases

- **AI Agent Orchestration**: Multi-step LLM calls, tool usage, and decision flows
- **ETL Data Pipelines**: Complex dependency relationships and data transformations
- **Automated Operations**: Parallel execution of multi-server tasks
- **File Management**: Intelligent classification, batch processing, human-in-the-loop confirmation

## Architecture

The system follows Domain-Driven Design (DDD) with four layers:
- **Interfaces Layer**: CLI, TUI, MCP Server
- **Application Layer**: Use cases, services, workflow orchestration
- **Domain Layer**: Business models and port interfaces
- **Infrastructure Layer**: Persistence, plugins, cache, external integrations

## Target Users

- **AI Agents**: Intelligent systems that need to call tools to complete tasks
- **AI Developers**: Building LLM agent applications
- **Workflow Developers**: Writing workflow scripts to define business processes
- **Plugin Developers**: Developing multi-language plugins to extend system capabilities
- **Data Engineers**: Building ETL pipelines
- **Operations Engineers**: Automating operational tasks
- **File Administrators**: Batch file processing
