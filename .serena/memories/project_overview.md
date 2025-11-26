# Oxc Project Overview

## Purpose
Oxc is a high-performance JavaScript/TypeScript toolchain written in Rust. It contains:
- **Parser** - JavaScript/TypeScript parser with AST support
- **Linter (oxlint)** - Fast linting engine
- **Formatter (oxfmt)** - Code formatting (Prettier-like)
- **Transformer** - Code transformation (Babel-like)
- **Minifier** - Code minification
- **Codegen** - Code generation from AST

## Tech Stack
- **Language**: Rust (MSRV: 1.91.1)
- **Package Manager**: Cargo (Rust), pnpm (Node.js)
- **Build System**: Just (task runner), Cargo
- **Testing**: Rust tests, insta (snapshots), conformance tests

## Repository Structure
- `crates/` - Core Rust functionality
- `apps/` - Application binaries (oxlint, oxfmt)
- `napi/` - Node.js bindings
- `npm/` - npm packages
- `tasks/` - Development tools/automation
- `editors/` - Editor integrations

## Key Crates
- `oxc_parser` - JS/TS parser
- `oxc_ast` - AST definitions/utilities
- `oxc_semantic` - Semantic analysis
- `oxc_linter` - Linting engine/rules
- `oxc_formatter` - Code formatting
- `oxc_transformer` - Code transformation
- `oxc_minifier` - Code minification
- `oxc_codegen` - Code generation
- `oxc_diagnostics` - Error reporting
- `oxc_traverse` - AST traversal utilities
- `oxc_allocator` - Memory management
