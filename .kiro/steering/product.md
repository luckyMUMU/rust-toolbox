# Product Overview

Workflow Toolkit is a high-performance workflow orchestration engine built in Rust. It enables complex AI agent workflows, data pipelines, and automation tasks with microsecond-level scheduling latency.

## Core Value Propositions

- **High Performance**: Rust + Tokio async runtime, supports 100+ concurrent workflows on a single machine
- **Multi-Language Plugin System**: Native Rust, Python, Node.js, Docker, and WASM plugins
- **AI-Ready**: Native MCP (Model Context Protocol) support for LLM integration
- **Reliable**: Built-in checkpointing, automatic retry, transactional state management, and audit logging
- **Observable**: TUI real-time monitoring and LanceDB time-travel queries

## Key Use Cases

- AI Agent orchestration with multi-step LLM calls and tool usage
- ETL data pipelines with complex dependencies
- Automated operations across multiple servers
- Intelligent file management with classification and batch processing

## Architecture

DDD layered architecture with clear separation of concerns:
- **Interfaces Layer**: CLI, TUI, MCP Server
- **Application Layer**: Use cases, services, workflow orchestration
- **Domain Layer**: Business models and port interfaces
- **Infrastructure Layer**: Persistence, plugins, cache, external services
