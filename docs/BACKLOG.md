# Backlog

Casual backlog for ideas, experiments, and future work. Order is not strict.
This backlog is the single source of truth for tasks and ideas.

## Ideas / Wishlist

### Gameplay

- [X] Candidate notes (player notes)
- [X] Undo/redo
- [X] Reset current puzzle
- [ ] Timer and statistics (e.g., solve time, mistakes, hints used)
- [ ] Puzzle paste/import (text paste/manual input; no difficulty/uniqueness assumptions)

### Puzzle & solver

- [ ] Difficulty-based puzzle generation
- [ ] Seeded generation and regenerate by seed
- [ ] Technique explanations for hints (may overlap with hint system)
- [ ] Generator-aligned technique expansion (solver + generator; pairs, pointing, box/line, X-Wing)

### Optional assist features

- [X] Selection row/column/box highlight
- [X] Same digit highlight
- [X] Highlight peers of same-digit cells (row/column/box)
- [X] Mistake highlighting (row/col/box conflicts)
- [X] Block rule-violating input (optional)
  - [X] Indicate blocked candidates on keypad buttons (optional)
  - [ ] Allow toggling blocked-candidate indicator (optional)
- [X] Ghost input preview for blocked actions
- [X] Assist: on digit entry, remove that digit’s note from all peers (same row, column, or box) (optional)
- [X] Notes auto-fill for possible digits (optional)
  - [X] For selected cell
  - [X] For all cells
  - [X] Auto-fill notes on new game/reset (optional)
- [ ] Hint system (incremental)
  - [X] Core flow wiring (candidate grid + inconsistency check)
  - [ ] Check Solvability UI + dialog
  - [ ] Hint stage 1 highlight (condition cells)
  - [ ] Hint stage 2 technique + condition pairs
  - [ ] Hint stage 3 apply step + ghost eliminations
  - [ ] Rollback flow for inconsistent boards

### Application features

- [X] Digit count/tally display (per digit)
- [X] Digit count integrated number pad/buttons (mouse-only input)
- [X] Light/Dark mode toggle
- [X] Mouse-only input (number pad/buttons)
- [X] New Game confirmation dialog
- [ ] Input discoverability (make shortcuts/keybinds easier to find; format TBD)
- [ ] UI clarity & visual polish (general improvements; details TBD)
- [X] Keypad buttons show which action will occur for notes (add/remove)
- [X] Feature toggles UI for (optional) assists
- [X] UI ViewModel-based split/refactor
- [X] App logic refactor for testability (action extraction + action_handler + view_model_builder split)
- [X] Auto-save and resume (board state + settings)
- [X] WASM build (run in web browser)
- [X] Publish web build via GitHub Actions + GitHub Pages
- [X] Replace template app icons with Numelace branding
- [X] Smartphone UI optimization (touch targets, layout adjustments, modal sizing)
- [X] Settings modal (front-and-center modal with close button + outside click)

## Bugs / Fixes

- [X] Debug: debug builds hit a `debug_assert` with the message `add_space makes no sense in a grid layout`

## Notes

- It’s OK to list “maybe” ideas here, even if you’re not sure you’ll build them.
- Keep items short and lightweight.
- Move decisions and rationale to DESIGN_LOG.
