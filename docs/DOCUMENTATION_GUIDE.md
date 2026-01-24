# Documentation Guide

This guide explains how to write and structure documentation for this project.

## TL;DR

- Start with Overview, then Design/Architecture, then Examples.
- Keep doc comments in complete sentences with runnable examples.
- Link to related items using `[ItemName]`.

## Crate-level Documentation (lib.rs)

### Standard Structure

Each crate's `lib.rs` should follow this structure:

> **1. Overview**
>
> Brief description of what the crate does (1-2 paragraphs)
>
> **2. Architecture / Design / Algorithm**
>
> Main design patterns, architecture, or algorithms (can include multiple sections as needed)
>
> - Include a **Design Rationale** subsection explaining why these design choices were made
>
> **3. Examples**
>
> Runnable code examples:
>
> - Basic Usage
> - Advanced Usage (optional)
>
> **4. Advanced Topics** (optional)
>
> - Performance characteristics
> - Extending functionality (e.g., "Adding New Techniques")
> - Error handling

### Choosing Section Names

These section types are not mutually exclusive. A crate can have multiple sections:

- **Architecture** for structural design (e.g., "Two-Grid Architecture", "Two-Layer Design")
- **Design** for design patterns or decisions (e.g., "Design Decisions", "Semantics Pattern")
- **Algorithm** for procedural approaches (e.g., "Removal Method")

For example, sudoku-generator uses both "Algorithm" (describes the removal method) and "Design Rationale" (explains why that method was chosen).

### Example

See [sudoku-core/src/lib.rs](../crates/sudoku-core/src/lib.rs) for a complete example following this structure.

## Documentation Style

- **Complete sentences**: Write in full sentences, not fragments
- **Runnable examples**: Prefer runnable code examples (examples in doc comments are automatically tested as doctests)
- **Cross-references**: Use `[ItemName]` to link to related items (types, functions, modules, etc.)

## Formatting

Run the following occasionally to format doc comments and organize imports:

```bash
cargo fmt -- --config format_code_in_doc_comments=true \
             --config group_imports=StdExternalCrate \
             --config imports_granularity=Crate
```

Note: These are unstable rustfmt options, but work via command line even on stable Rust.
