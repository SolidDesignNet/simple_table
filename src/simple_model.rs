use std::ops::Range;

use fltk::widget::Widget;

// Sort order
#[derive(Debug, Clone, Copy)]
pub enum Order {
    Ascending,
    Descending,
    None,
}
impl Order {
    pub fn next(&self) -> Order {
        match self {
            Order::Ascending => Order::Descending,
            Order::Descending => Order::None,
            Order::None => Order::Ascending,
        }
    }
    pub fn apply(&self, o: std::cmp::Ordering) -> std::cmp::Ordering {
        match self {
            Order::Descending => o.reverse(),
            _ => o,
        }
    }
}

/// Custom renderer without the overhead of creating a full widget
pub trait DrawDelegate {
    fn draw(&self, row: i32, col: i32, x: i32, y: i32, w: i32, h: i32, selected: bool);
}

pub enum RowHeight {
    All(u32),
    PerRow(Box<dyn Fn(usize) -> u32>),
}
impl RowHeight {
    pub fn for_row(&self, row: u32) -> u32 {
        match self {
            RowHeight::All(h) => *h,
            RowHeight::PerRow(f) => f(row as usize),
        }
    }
    pub fn for_range(&self, range: Range<u32>) -> u32 {
        match self {
            RowHeight::All(h) => h * (range.end - range.start),
            RowHeight::PerRow(f) => range.map(|r| f(r as usize)).sum(),
        }
    }
}

pub struct RowInfo {
    pub count: usize,
    pub height: RowHeight,
}

pub struct ColumnDetail {
    pub header: String,
    pub width: u32,
}

pub struct ColumnInfo {
    pub details: Vec<ColumnDetail>,
}
impl ColumnInfo {
    /// Total width of all columns
    pub fn total_width(&self) -> usize {
        self.details.iter().map(|d| d.width as usize).sum()
    }
}

pub enum SimpleCell {
    Text(String),
    Delegate(Box<dyn DrawDelegate>),
    Widget(Widget),
    None,
}

impl SimpleCell {
    /// Used by copy/paste and export functions
    pub fn as_str(&self) -> Option<&str> {
        match self {
            SimpleCell::Text(s) => Some(s),
            _ => None,
        }
    }
}

/// Table model trait. Implementations of this trait will describe how to display a table.
// FIXME use i32 or u32 consistently!
pub trait SimpleModel {
    fn row_info(&mut self) -> RowInfo;
    fn column_info(&mut self) -> ColumnInfo;

    fn get_cell(&mut self, row: i32, col: i32) -> SimpleCell;

    /// Popup help.
    fn hover(&self, _row: i32, _col: i32) -> Option<String> {
        None
    }
    /// Optional sorting. Activated by clicking on a header.
    fn sort(&mut self, _col: usize, _order: Order) {}
}
