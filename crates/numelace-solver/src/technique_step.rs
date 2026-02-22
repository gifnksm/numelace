//! Sudoku solving techniques.
//!
//! This module provides various techniques for solving Sudoku puzzles.
//! Each technique implements the [`Technique`] trait and can be applied to a [`TechniqueGrid`].

use std::fmt::Debug;

use numelace_core::{Digit, DigitPositions, DigitSet, Position};

use crate::TechniqueGrid;

/// Cells involved in a technique's applicability conditions.
pub type ConditionCells = DigitPositions;

/// Pairs of (cells, digits) involved in a technique's applicability conditions.
pub type ConditionDigitCells = Vec<(DigitPositions, DigitSet)>;

/// A hint step produced by a technique.
pub trait TechniqueStep: Debug + Send + Sync {
    /// Returns the name of the technique that produced this step.
    fn technique_name(&self) -> &'static str;

    /// Returns a boxed clone of the step.
    fn clone_box(&self) -> BoxedTechniqueStep;

    /// Returns the cells involved in the applicability conditions.
    ///
    /// These are the cells that justify applying the technique. Hint systems may
    /// use this to highlight relevant cells before naming the technique.
    fn condition_cells(&self) -> ConditionCells;

    /// Returns condition pairs of (cells, digits) involved in applicability.
    ///
    /// Each pair provides a set of cells and the digits that matter for the
    /// technique's conditions. Hint systems may use this as a more detailed
    /// explanation of the underlying logic.
    fn condition_digit_cells(&self) -> ConditionDigitCells;

    /// Returns the concrete changes produced by applying the technique.
    fn application(&self) -> Vec<TechniqueApplication>;
}

/// Shared data for technique steps without technique-specific payloads.
#[derive(Debug, Clone)]
pub struct TechniqueStepData {
    technique_name: &'static str,
    condition_cells: ConditionCells,
    condition_digit_cells: ConditionDigitCells,
    application: Vec<TechniqueApplication>,
}

impl TechniqueStepData {
    /// Creates a new `TechniqueStepData`.
    #[must_use]
    pub fn new(
        technique_name: &'static str,
        condition_cells: ConditionCells,
        condition_digit_cells: ConditionDigitCells,
        application: Vec<TechniqueApplication>,
    ) -> Self {
        Self {
            technique_name,
            condition_cells,
            condition_digit_cells,
            application,
        }
    }

    /// Creates a new `TechniqueStepData` from a before/after grid diff.
    #[must_use]
    pub fn from_diff(
        technique_name: &'static str,
        condition_cells: ConditionCells,
        condition_digit_cells: ConditionDigitCells,
        before: &TechniqueGrid,
        after: &TechniqueGrid,
    ) -> Self {
        Self::from_diff_with_extra(
            technique_name,
            condition_cells,
            condition_digit_cells,
            before,
            after,
            Vec::new(),
        )
    }

    /// Creates a new `TechniqueStepData` from a before/after grid diff,
    /// appending extra applications.
    #[must_use]
    pub fn from_diff_with_extra(
        technique_name: &'static str,
        condition_cells: ConditionCells,
        condition_digit_cells: ConditionDigitCells,
        before: &TechniqueGrid,
        after: &TechniqueGrid,
        mut extra_application: Vec<TechniqueApplication>,
    ) -> Self {
        let mut application = collect_applications_from_diff(before, after);
        application.append(&mut extra_application);
        Self::new(
            technique_name,
            condition_cells,
            condition_digit_cells,
            application,
        )
    }
}

impl TechniqueStep for TechniqueStepData {
    fn technique_name(&self) -> &'static str {
        self.technique_name
    }

    fn clone_box(&self) -> BoxedTechniqueStep {
        Box::new(self.clone())
    }

    fn condition_cells(&self) -> ConditionCells {
        self.condition_cells
    }

    fn condition_digit_cells(&self) -> ConditionDigitCells {
        self.condition_digit_cells.clone()
    }

    fn application(&self) -> Vec<TechniqueApplication> {
        self.application.clone()
    }
}

/// Concrete changes produced by applying a technique.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TechniqueApplication {
    /// Place a digit in a single cell.
    Placement {
        /// Cell to place the digit into.
        position: Position,
        /// Digit to place.
        digit: Digit,
    },
    /// Remove candidates from the specified positions.
    CandidateElimination {
        /// Positions where candidates are removed.
        positions: DigitPositions,
        /// Digits to remove from the specified positions.
        digits: DigitSet,
    },
}

/// A boxed technique step.
pub type BoxedTechniqueStep = Box<dyn TechniqueStep>;

impl Clone for BoxedTechniqueStep {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

fn collect_applications_from_diff(
    before: &TechniqueGrid,
    after: &TechniqueGrid,
) -> Vec<TechniqueApplication> {
    let mut app = vec![];
    for digit in DigitSet::FULL {
        let before_positions = before.digit_positions(digit);
        let after_positions = after.digit_positions(digit);
        debug_assert!(before_positions.is_superset(after_positions));
        let diff = before_positions.difference(after_positions);
        if !diff.is_empty() {
            app.push(TechniqueApplication::CandidateElimination {
                positions: diff,
                digits: DigitSet::from_elem(digit),
            });
        }
    }
    app
}
