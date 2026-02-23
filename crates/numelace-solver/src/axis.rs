use numelace_core::{
    Digit, DigitPositions, HouseMask, Position, containers::Array9, index::CellIndexSemantics,
};

use crate::TechniqueGrid;

pub(crate) trait AxisOps {
    const LINE_POSITIONS: Array9<DigitPositions, CellIndexSemantics>;
    const CROSS_POSITIONS: Array9<DigitPositions, CellIndexSemantics>;

    fn line_mask(grid: &TechniqueGrid, index: u8, digit: Digit) -> HouseMask;
    fn cross_index(pos: Position) -> u8;
    fn make_pos(line: u8, cross: u8) -> Position;
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RowAxis;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ColumnAxis;

impl AxisOps for RowAxis {
    const LINE_POSITIONS: Array9<DigitPositions, CellIndexSemantics> =
        DigitPositions::ROW_POSITIONS;
    const CROSS_POSITIONS: Array9<DigitPositions, CellIndexSemantics> =
        DigitPositions::COLUMN_POSITIONS;

    #[inline]
    fn line_mask(grid: &TechniqueGrid, index: u8, digit: Digit) -> HouseMask {
        grid.row_mask(index, digit)
    }

    #[inline]
    fn cross_index(pos: Position) -> u8 {
        pos.x()
    }

    #[inline]
    fn make_pos(line: u8, cross: u8) -> Position {
        Position::new(cross, line)
    }
}

impl AxisOps for ColumnAxis {
    const LINE_POSITIONS: Array9<DigitPositions, CellIndexSemantics> =
        DigitPositions::COLUMN_POSITIONS;
    const CROSS_POSITIONS: Array9<DigitPositions, CellIndexSemantics> =
        DigitPositions::ROW_POSITIONS;

    #[inline]
    fn line_mask(grid: &TechniqueGrid, index: u8, digit: Digit) -> HouseMask {
        grid.col_mask(index, digit)
    }

    #[inline]
    fn cross_index(pos: Position) -> u8 {
        pos.y()
    }

    #[inline]
    fn make_pos(line: u8, cross: u8) -> Position {
        Position::new(line, cross)
    }
}
