# Task Completion Checklist for Oxc

## Before Completing a Task

1. **Format code**
   ```bash
   just fmt
   ```

2. **Run checks**
   ```bash
   just check
   ```

3. **Run tests**
   ```bash
   just test
   ```

4. **Run linting**
   ```bash
   just lint
   ```

5. **Update generated files if needed**
   - After `oxc_ast` changes: `just ast`
   - After `oxc_minifier` changes: `just minsize`
   - After `oxc_parser` changes: `just allocs`

6. **Update snapshots if needed**
   ```bash
   cargo insta review
   ```

## Full Ready Check

Run all checks before finalizing:
```bash
just ready
```

This runs: typos, fmt, check, test, lint, doc, ast, and git status

## Important Notes

- Check `git status` to ensure no unexpected changes
- Review snapshot diffs carefully
- Ensure tests pass for related conformance suites
- Performance is critical - avoid unnecessary allocations
