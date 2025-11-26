# Suggested Commands for Oxc Development

## Essential Development Commands

```bash
# Format code (run after modifications)
just fmt

# Run cargo check for compilation errors
just check

# Run all unit/integration tests
just test

# Lint the project
just lint

# Build documentation
just doc

# Update AST generated files (after oxc_ast changes)
just ast

# Run all checks (use before committing)
just ready
```

## Testing Commands

```bash
# Run all tests
cargo test --all-features

# Run tests for a specific crate
cargo test -p <crate_name>

# Run conformance tests
just conformance

# Run all coverage tests
cargo coverage

# Test with specific filter
cargo test -p <crate_name> -- <test_name>
```

## Running Examples

```bash
# Run crate examples
cargo run -p <crate_name> --example <example_name> -- [args]

# Common examples
cargo run -p oxc_parser --example parser -- test.js
cargo run -p oxc_linter --example linter -- src/
cargo run -p oxc --example compiler --features="full" -- test.js
```

## Build Commands

```bash
# Build oxlint
cargo build -p oxlint --release

# Build oxfmt
cargo build -p oxfmt --release
```

## Snapshot Testing

```bash
# Update insta snapshots
cargo insta review

# Update minifier size snapshots
just minsize

# Update allocation snapshots
just allocs
```

## Watch Mode

```bash
# Watch files and re-run command
just watch "command"

# Watch specific example
just watch-oxlint test.js
```

## Code Navigation with ast-grep

```bash
# Syntax-aware search in Rust
ast-grep --lang rust -p '<pattern>'
```
