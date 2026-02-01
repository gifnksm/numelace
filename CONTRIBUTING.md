# Contributing

This document describes the standards and guidelines for this project.

## Documentation

- [Architecture](docs/ARCHITECTURE.md) - Project architecture and design decisions
- [Documentation Guide](docs/DOCUMENTATION_GUIDE.md) - How to write documentation
- [Testing Guidelines](docs/TESTING.md) - Testing philosophy and best practices
- [Backlog](docs/BACKLOG.md) - Ideas, experiments, and future work
- Naming: use `Numelace` for the app/brand name, and `Sudoku` when referring to the puzzle rules

### Icon assets

- Source SVG lives in `assets/source/icon.svg`.
- Generate the SVG with:
  - `scripts/generate_icon_svg`
- Generate all PNG/ICO assets with:
  - `scripts/generate_icon_assets`
- Default output path matches `assets/source/icon.svg`.

## Development Workflow

### Local CI checks

Prerequisites: `cargo`, `trunk`, and `typos` must be available on your PATH.

Run the local CI script to mirror GitHub Actions checks:

```bash
scripts/ci
```

### Code Quality

Format code with `cargo fmt`:

```bash
cargo fmt
```

Occasionally run the following to format doc comments and organize imports:

```bash
cargo fmt -- --config format_code_in_doc_comments=true \
             --config group_imports=StdExternalCrate \
             --config imports_granularity=Crate
```

Note: These are unstable rustfmt options, but work via command line even on stable Rust.

Run clippy and address warnings:

```bash
cargo clippy --all-targets
```

WASM target checks (lint and docs):

```bash
cargo clippy --all-targets --target wasm32-unknown-unknown
cargo doc --no-deps --target wasm32-unknown-unknown
```

### Documentation

Check markdown files with markdownlint:

```bash
npx markdownlint .
```

Generate and view API documentation:

```bash
# Generate documentation (project crates only, faster)
cargo doc --no-deps

# Generate and open documentation
cargo doc --no-deps --open
```

### Testing

```bash
# Run all tests
cargo test

# Run benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench backtrack
cargo bench --bench solver
cargo bench --bench generator
```
