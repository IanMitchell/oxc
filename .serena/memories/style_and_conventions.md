# Oxc Code Style and Conventions

## Rust Style

### Formatting

- Uses rustfmt with `style_edition = "2024"`
- `use_small_heuristics = "Max"` for wider formatting
- `use_field_init_shorthand = true`
- `reorder_modules = true`

### Memory Management

- Use `oxc_allocator` for memory management
- Performance-critical: avoid unnecessary allocations
- Arena allocation for AST nodes

### Error Handling

- Use `oxc_diagnostics` for errors with source locations
- Rich error messages with spans

## Naming Conventions

- Snake_case for function and variable names
- PascalCase for types and traits
- SCREAMING_SNAKE_CASE for constants

## Testing

- Unit tests co-located with source in `tests/` directories
- Snapshot testing with `insta` crate
- Conformance tests against Test262, Babel, TypeScript, Prettier

## Code Organization

- Avoid editing `generated` subdirectories
- Follow visitor pattern for AST traversal
- Use `oxc_traverse` for AST traversal utilities

## Comments

- Generally match existing file style
- Necessary to explain complex changes
