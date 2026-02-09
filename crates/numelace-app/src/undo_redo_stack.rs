use std::{
    collections::{VecDeque, vec_deque},
    num::NonZero,
};

#[derive(Debug, Clone)]
pub(crate) struct UndoRedoStack<T> {
    stack: VecDeque<T>,
    capacity: NonZero<usize>,
    cursor: usize,
}

impl<T> UndoRedoStack<T> {
    #[must_use]
    pub(crate) fn new(capacity: NonZero<usize>) -> Self {
        Self {
            stack: VecDeque::new(),
            capacity,
            cursor: 0,
        }
    }

    #[must_use]
    pub(crate) fn capacity(&self) -> NonZero<usize> {
        self.capacity
    }

    #[must_use]
    pub(crate) fn cursor(&self) -> usize {
        self.cursor
    }

    #[must_use]
    pub(crate) fn entries(&self) -> vec_deque::Iter<'_, T> {
        self.stack.iter()
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

        if self.stack.len() == self.capacity.get() {
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

    #[must_use]
    pub(crate) fn iter_from_current(&self) -> impl DoubleEndedIterator<Item = &T> {
        self.stack.iter().take(self.cursor + 1).rev()
    }

    pub(crate) fn restore_from_parts(&mut self, mut stack: VecDeque<T>, cursor: usize) {
        if stack.len() > self.capacity.get() {
            let overflow = stack.len() - self.capacity.get();
            for _ in 0..overflow {
                stack.pop_front();
            }
            let adjusted_cursor = cursor.saturating_sub(overflow);
            self.stack = stack;
            self.cursor = adjusted_cursor.min(self.stack.len().saturating_sub(1));
            return;
        }

        self.stack = stack;
        self.cursor = cursor.min(self.stack.len().saturating_sub(1));
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZero;

    use super::UndoRedoStack;

    #[test]
    fn undo_redo_roundtrip() {
        let mut history = UndoRedoStack::new(NonZero::new(10).unwrap());
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
        let mut history = UndoRedoStack::new(NonZero::new(10).unwrap());
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
        let mut history = UndoRedoStack::new(NonZero::new(3).unwrap());
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
        let mut history = UndoRedoStack::new(NonZero::new(10).unwrap());
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
        let mut history: UndoRedoStack<i32> = UndoRedoStack::new(NonZero::new(5).unwrap());

        assert_eq!(history.current(), None);
        assert!(!history.undo());
        assert!(!history.redo());
        assert_eq!(history.current(), None);
    }

    #[test]
    fn clear_resets_history_state() {
        let mut history = UndoRedoStack::new(NonZero::new(5).unwrap());
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
