# Backlog

Casual backlog for ideas, experiments, and future work. Order is not strict.
This backlog is the single source of truth for tasks and ideas.

## Ideas / Wishlist

### Gameplay

- Candidate notes (player notes) ✅
- Undo/redo ✅
- Reset current puzzle
- Timer and statistics (e.g., solve time, mistakes, hints used)
- Puzzle paste/import (text paste/manual input; no difficulty/uniqueness assumptions)

### Puzzle & solver

- Difficulty-based puzzle generation
- Seeded generation and regenerate by seed
- Technique explanations for hints (may overlap with hint system)
- Solver technique extensions (pairs, pointing, box/line, X-Wing)

### Optional assist features

- Selection row/column/box highlight ✅
- Same digit highlight ✅
- Highlight peers of same-digit cells (row/column/box) ✅
- Mistake highlighting (row/col/box conflicts) ✅
- Block rule-violating input (optional) ✅
  - Indicate blocked candidates on keypad buttons (optional) ✅
  - Allow toggling blocked-candidate indicator (optional)
- Ghost input preview for blocked actions ✅
- Assist: on digit entry, remove that digit’s note from all peers (same row, column, or box) (optional) ✅
- Notes auto-fill for possible digits (optional)
  - For selected cell
  - For all cells
- Hint system (scope TBD: show applicable technique, target cells, or apply a step)

### Application features

- Digit count/tally display (per digit) ✅
- Digit count integrated number pad/buttons (mouse-only input) ✅
- Light/Dark mode toggle ✅
- Mouse-only input (number pad/buttons) ✅
- New Game confirmation dialog ✅
- Input discoverability (make shortcuts/keybinds easier to find; format TBD)
- UI clarity & visual polish (general improvements; details TBD)
- Keypad buttons show which action will occur (set/clear digit or set/clear note)
- Feature toggles UI for (optional) assists ✅
- UI ViewModel-based split/refactor ✅
- App logic refactor for testability (action extraction + action_handler + view_model_builder split) ✅
- Auto-save and resume (board state + settings) ✅
- WASM build (run in web browser)

## Bugs / Fixes

- None listed yet

## Notes

- It’s OK to list “maybe” ideas here, even if you’re not sure you’ll build them.
- Keep items short and lightweight.
- Move decisions and rationale to DESIGN_LOG.
