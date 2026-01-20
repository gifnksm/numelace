# Solver Design

## Overview

Solver implements technique-based solving with backtracking fallback. Simple techniques are applied first, progressing to more complex ones. When any progress is made (cell placed or candidate removed), the solver resets to the first technique to ensure simpler solutions are not missed.

## Architecture

### Two-Layer Solver Structure

- **`TechniqueSolver`**: Applies only human-like techniques, no backtracking
- **`BacktrackSolver`**: Uses `TechniqueSolver` first, falls back to backtracking when stuck

This separation allows:

- Testing technique-only solving
- Evaluating puzzle difficulty (what techniques are needed)
- Generating puzzles with specific technique requirements

### Technique Trait

```rust
trait Technique {
    fn apply(&self, grid: &mut CandidateGrid) -> Result<bool, SolverError>;
}
```

- **Stateless design**: Techniques hold no state
- **Returns**: `true` if progress made, `false` if not applicable
- **Error**: Returns error on contradiction detection

### Progress Strategy

Both cell placement and candidate removal trigger reset to first technique:

- **Candidate removed** → Reset to first technique
- **Cell placed** → Reset to first technique  
- **No change** → Try next technique

This ensures simpler techniques are always preferred.

### Solver Results

- **Solved**: Puzzle solved successfully
- **Unsolvable**: No solution exists (contradiction detected or exhausted)
- **Multiple solutions**: For solution enumeration (puzzle generation validation)

### Backtracking

- Triggered when all techniques fail to make progress
- Clones `CandidateGrid` (144 bytes, acceptable cost)
- No depth limit (max 81 levels, safe for stack)
- Implemented as special logic in `BacktrackSolver`, not as a `Technique`

## Module Structure

```text
sudoku-solver/
├── lib.rs                    # Public API, re-exports
├── technique/
│   ├── mod.rs                # Technique trait, common types
│   ├── naked_single.rs       # NakedSingle implementation + tests
│   └── hidden_single.rs      # HiddenSingle implementation + tests
├── technique_solver.rs       # TechniqueSolver implementation
├── backtrack_solver.rs       # BacktrackSolver implementation
└── error.rs                  # SolverError definition
```

**File responsibilities**:

- `technique/mod.rs`: Technique trait, shared types
- `technique/naked_single.rs`, `hidden_single.rs`: Individual technique implementations with integrated tests
- `technique_solver.rs`: Iterates through techniques, resets on progress
- `backtrack_solver.rs`: Wraps TechniqueSolver, adds backtracking
- `error.rs`: Error types (detailed during implementation)

## Implementation Order

1. Basic types: Technique trait, SolverError, result types
2. NakedSingle: Implementation + tests
3. HiddenSingle: Implementation + tests
4. TechniqueSolver: Driver loop + integration tests
5. BacktrackSolver: Backtracking logic

## Testing Strategy

- **Unit tests**: Each technique module includes its own tests
- **Integration tests**: Full solver tests with known puzzles
- **Test data**: Easy, medium, hard puzzles with expected solutions

## Future Considerations

Deferred until needed:

- Technique statistics (application count, success rate)
- Difficulty scoring based on techniques used
- Dynamic technique selection (for puzzle generation)
- Additional advanced techniques (X-Wing, Swordfish, etc.)
