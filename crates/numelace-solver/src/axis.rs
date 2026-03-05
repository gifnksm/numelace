use numelace_core::{CellIndexIndexedArray, Digit, DigitPositions, House, HouseMask, Position};

use crate::TechniqueGrid;

pub(crate) trait AxisOps {
    const LINE_POSITIONS: CellIndexIndexedArray<DigitPositions>;
    const CROSS_POSITIONS: CellIndexIndexedArray<DigitPositions>;
    const LINE_HOUSES: CellIndexIndexedArray<House>;
    const CROSS_HOUSES: CellIndexIndexedArray<House>;

    fn line_mask(grid: &TechniqueGrid, index: u8, digit: Digit) -> HouseMask;
    fn cross_index(pos: Position) -> u8;
    fn make_pos(line: u8, cross: u8) -> Position;
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RowAxis;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ColumnAxis;

impl AxisOps for RowAxis {
    const LINE_POSITIONS: CellIndexIndexedArray<DigitPositions> = DigitPositions::ROW_POSITIONS;
    const CROSS_POSITIONS: CellIndexIndexedArray<DigitPositions> = DigitPositions::COLUMN_POSITIONS;
    const LINE_HOUSES: CellIndexIndexedArray<House> = House::ROWS;
    const CROSS_HOUSES: CellIndexIndexedArray<House> = House::COLUMNS;

    #[inline]
    fn line_mask(grid: &TechniqueGrid, index: u8, digit: Digit) -> HouseMask {
        grid.digit_positions(digit).positions_in_row(index)
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
    const LINE_POSITIONS: CellIndexIndexedArray<DigitPositions> = DigitPositions::COLUMN_POSITIONS;
    const CROSS_POSITIONS: CellIndexIndexedArray<DigitPositions> = DigitPositions::ROW_POSITIONS;
    const LINE_HOUSES: CellIndexIndexedArray<House> = House::COLUMNS;
    const CROSS_HOUSES: CellIndexIndexedArray<House> = House::ROWS;

    #[inline]
    fn line_mask(grid: &TechniqueGrid, index: u8, digit: Digit) -> HouseMask {
        grid.digit_positions(digit).positions_in_col(index)
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
