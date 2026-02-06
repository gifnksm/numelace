use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub(crate) struct UndoRedoStack<T> {
    stack: VecDeque<T>,
    capacity: usize,
    cursor: usize,
}

impl<T> UndoRedoStack<T> {
    #[must_use]
    pub(crate) fn new(capacity: usize) -> Self {
        assert!(capacity > 0);
        Self {
            stack: VecDeque::with_capacity(capacity),
            capacity,
            cursor: 0,
        }
    }

    pub(crate) fn push(&mut self, item: T) {
        if self.stack.is_empty() {
            self.stack.push_back(item);
            self.cursor = 0;
            return;
        }

        let truncate_len = self.cursor + 1;
        if truncate_len < self.stack.len() {
            self.stack.truncate(truncate_len);
        }

        if self.stack.len() == self.capacity {
            self.stack.pop_front();
            if self.cursor > 0 {
                self.cursor -= 1;
            }
        }

        self.stack.push_back(item);
        self.cursor = self.stack.len() - 1;
    }

    #[must_use]
    pub(crate) fn can_undo(&self) -> bool {
        !self.stack.is_empty() && self.cursor > 0
    }

    pub(crate) fn undo(&mut self) -> bool {
        if self.can_undo() {
            self.cursor -= 1;
            true
        } else {
            false
        }
    }

    #[must_use]
    pub(crate) fn can_redo(&self) -> bool {
        !self.stack.is_empty() && self.cursor + 1 < self.stack.len()
    }

    pub(crate) fn redo(&mut self) -> bool {
        if self.can_redo() {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    pub(crate) fn clear(&mut self) {
        self.stack.clear();
        self.cursor = 0;
    }

    #[must_use]
    pub(crate) fn current(&self) -> Option<&T> {
        self.stack.get(self.cursor)
    }
}

#[cfg(test)]
mod tests {
    use super::UndoRedoStack;

    #[test]
    fn undo_redo_roundtrip() {
        let mut history = UndoRedoStack::new(10);
        history.push(1);
        history.push(2);
        history.push(3);

        assert_eq!(history.current(), Some(&3));
        assert!(history.undo());
        assert_eq!(history.current(), Some(&2));
        assert!(history.undo());
        assert_eq!(history.current(), Some(&1));
        assert!(history.redo());
        assert_eq!(history.current(), Some(&2));
        assert!(history.redo());
        assert_eq!(history.current(), Some(&3));
        assert!(!history.redo());
    }

    #[test]
    fn redo_clears_after_push() {
        let mut history = UndoRedoStack::new(10);
        history.push(1);
        history.push(2);
        history.push(3);

        assert!(history.undo());
        assert_eq!(history.current(), Some(&2));
        history.push(4);

        assert!(!history.redo());
        assert!(history.undo());
        assert_eq!(history.current(), Some(&2));
        assert!(history.redo());
        assert_eq!(history.current(), Some(&4));
    }

    #[test]
    fn capacity_drops_oldest_and_adjusts_cursor() {
        let mut history = UndoRedoStack::new(3);
        history.push(1);
        history.push(2);
        history.push(3);
        history.push(4);

        assert_eq!(history.current(), Some(&4));
        assert!(history.undo());
        assert_eq!(history.current(), Some(&3));
        assert!(history.undo());
        assert_eq!(history.current(), Some(&2));
        assert!(!history.undo());
    }

    #[test]
    fn undo_redo_stops_at_bounds() {
        let mut history = UndoRedoStack::new(10);
        history.push(1);
        history.push(2);
        history.push(3);

        assert!(history.undo());
        assert!(history.undo());
        assert_eq!(history.current(), Some(&1));
        assert!(!history.undo());
        assert_eq!(history.current(), Some(&1));

        assert!(history.redo());
        assert!(history.redo());
        assert_eq!(history.current(), Some(&3));
        assert!(!history.redo());
        assert_eq!(history.current(), Some(&3));
    }

    #[test]
    fn empty_history_returns_none_and_false() {
        let mut history: UndoRedoStack<i32> = UndoRedoStack::new(5);

        assert_eq!(history.current(), None);
        assert!(!history.undo());
        assert!(!history.redo());
        assert_eq!(history.current(), None);
    }

    #[test]
    fn clear_resets_history_state() {
        let mut history = UndoRedoStack::new(5);
        history.push(1);
        history.push(2);
        history.push(3);

        history.clear();

        assert_eq!(history.current(), None);
        assert!(!history.undo());
        assert!(!history.redo());

        history.push(4);
        assert_eq!(history.current(), Some(&4));
        assert!(!history.undo());
    }
}
