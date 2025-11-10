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
/// Custom renderer
pub trait DrawDelegate {
    fn draw(&self, row: i32, col: i32, x: i32, y: i32, w: i32, h: i32, selected: bool);
}

/// Table model trait. Implementations of this trait will describe how to display a table.
// FIXME use i32 or u32 consistently!
pub trait SimpleModel {
    /// How many rows in the table?
    fn row_count(&mut self) -> usize;
    /// How many columns in the table?
    fn column_count(&mut self) -> usize;
    /// Table header (column titles)
    fn header(&mut self, col: usize) -> String;
    /// default column widths.  They are resizable by the user, but default to this size.
    fn column_width(&mut self, col: usize) -> u32;

    fn all_row_height(&mut self) -> Option<u32> {
        Some(20)
    }
    fn row_height(&mut self, _row: i32) -> u32 {
        self.all_row_height().unwrap_or_default()
    }

    /// if cell returns None, then cell_delegate is called
    fn cell(&mut self, row: i32, col: i32) -> Option<String>;
    /// Custom renderer.
    fn cell_delegate(&mut self, _row: i32, _col: i32) -> Option<Box<dyn DrawDelegate>> {
        None
    }
    fn cell_widget(&mut self, _row: i32, _col: i32) -> Option<Widget> {
        None
    }
    /// Popup help.
    fn hover(&self, _row: i32, _col: i32) -> Option<String> {
        None
    }
    /// Optional sorting. Activated by clicking on a header.
    fn sort(&mut self, _col: usize, _order: Order) {}
}
