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

- [X] Difficulty-based puzzle generation
- [X] Seeded generation and regenerate by seed
  - [X] Reproducible generation requires seed + enabled techniques (or tier)
- [ ] Technique explanations for hints (may overlap with hint system)
- [ ] Refine difficulty evaluation to better match play experience (generation + UI difficulty labeling)
- [X] Add `solve_puzzle` CLI example in numelace-solver (step output, backtracking trace, grid rendering)
- Generator-aligned technique expansion (solver + generator)
  - Tier 0 (fundamental):
    - [X] Naked Single
    - [X] Hidden Single
  - Tier 1 (basic):
    - [X] Locked Candidates (Pointing)
    - [X] Locked Candidates (Claiming)
  - Tier 2 (intermediate):
    - [X] Naked Pair
    - [X] Hidden Pair
    - [X] Naked Triple
    - [X] Hidden Triple
  - Tier 3 (upper-intermediate):
    - [X] Naked Quad
    - [X] Hidden Quad
    - [X] X-Wing
    - [X] Skyscraper
    - [X] 2-String Kite
    - [X] Y-Wing
  - Tier 4 (advanced):
    - [X] Swordfish
    - [X] Jellyfish
    - [X] Remote Pair
    - [X] X-Chain
    - [X] XY-Chain
    - [X] XYZ-Wing
  - Tier 5 (expert):
    - [X] WXYZ-Wing
    - [ ] BUG+1 (Bivalue Universal Grave +1)

### Optional assist features

- [X] Selection row/column/box highlight
- [X] Same digit highlight
- [ ] Strong link visualization toggle (lines between linked candidates)
- [ ] Freehand drawing memo overlay
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
- [X] Hint system (incremental)
  - [X] Core flow wiring (candidate grid + inconsistency check)
  - [X] Check Solvability UI + dialog + shortcut (Ctrl+K)
  - [X] Hint stage 1 highlight (condition cells)
  - [X] Hint stage 2 technique + condition pairs
  - [ ] Hint stage 2: show strong links explicitly (lines between linked candidates)
  - [X] Hint stage 3 apply step + ghost eliminations
  - [X] Rollback flow for inconsistent boards

### Application features

- UI improvements (umbrella)
  - [ ] Input discoverability (make shortcuts/keybinds easier to find; format TBD)
  - [ ] Simpler digit input system (e.g., flick-style overlay input; needs idea exploration)
  - [X] Toolbar improvements (avoid overflow; theme toggle should scale with screen and be consistent/easy to tap)
  - [ ] UI clarity & visual polish (layout/spacing/contrast/icon consistency)
  - [X] Light/Dark mode toggle
  - [X] Mouse-only input (number pad/buttons)
  - [X] Digit count/tally display (per digit)
  - [X] Digit count integrated number pad/buttons (mouse-only input)
  - [X] Keypad buttons show which action will occur for notes (add/remove)
  - [X] Feature toggles UI for (optional) assists
  - [X] Smartphone UI optimization (touch targets, layout adjustments, modal sizing)
  - [X] Settings modal (front-and-center modal with close button + outside click)
  - [X] New Game confirmation dialog
- App architecture & refactoring (umbrella)
  - [X] UI ViewModel-based split/refactor
  - [X] App logic refactor for testability (action extraction + action_handler + view_model_builder split)
- State & persistence (umbrella)
  - [X] Auto-save and resume (board state + settings)
- Platform & distribution (umbrella)
  - [X] WASM build (run in web browser)
  - [X] Publish web build via GitHub Actions + GitHub Pages
  - [X] Replace template app icons with Numelace branding

## Bugs / Fixes

- [X] Debug: debug builds hit a `debug_assert` with the message `add_space makes no sense in a grid layout`

## Notes

- It’s OK to list “maybe” ideas here, even if you’re not sure you’ll build them.
- Keep items short and lightweight.
- Move decisions and rationale to DESIGN_LOG.
