use fltk::{
    draw, enums,
    prelude::{GroupExt, TableExt, WidgetBase, WidgetExt},
    table::{Table, TableContext},
};
use std::cell::RefCell;
use std::rc::Rc;

pub trait SimpleModel {
    fn set_table(&self, table: &SimpleTable) -> ();
    fn row_count(&self) -> usize;
    fn column_count(&self) -> usize;
    fn header(&self, col: usize) -> String;
    fn column_width(&self, col: usize) -> u32;
    fn cell(&self, row: i32, col: i32) -> Option<String>;
}
pub struct SimpleTable {
    pub table: Rc<RefCell<Table>>,
    pub model: Rc<RefCell<dyn SimpleModel>>,
}

impl SimpleTable {
    fn draw_header(txt: &str, x: i32, y: i32, w: i32, h: i32) {
        draw::push_clip(x, y, w, h);
        draw::draw_box(
            enums::FrameType::ThinUpBox,
            x,
            y,
            w,
            h,
            enums::Color::FrameDefault,
        );
        draw::set_draw_color(enums::Color::Black);
        draw::draw_text2(txt, x, y, w, h, enums::Align::Center);
        draw::pop_clip();
    }

    // The selected flag sets the color of the cell to a grayish color, otherwise white
    fn draw_data(txt: &str, x: i32, y: i32, w: i32, h: i32, selected: bool) {
        draw::push_clip(x, y, w, h);
        if selected {
            draw::set_draw_color(enums::Color::from_u32(0x00D3_D3D3));
        } else {
            draw::set_draw_color(enums::Color::White);
        }
        draw::draw_rectf(x, y, w, h);
        draw::set_draw_color(enums::Color::Gray0);
        draw::draw_text2(txt, x, y, w, h, enums::Align::Center);
        draw::draw_rect(x, y, w, h);
        draw::pop_clip();
    }

    pub fn new(model: Rc<RefCell<dyn SimpleModel>>) -> SimpleTable {
        let mut simple_table = SimpleTable {
            table: Rc::new(RefCell::new(Table::default_fill())),
            model,
        };
        simple_table.init();
        simple_table.redraw();
        simple_table
    }

    fn init(&mut self) {
        let m = self.model.borrow_mut();
        m.set_table(self);
        let mut table = self.table.borrow_mut();
        table.set_cols(m.column_count() as i32);
        table.set_col_header(true);
        for i in 0..m.column_count() {
            table.set_col_width(i as i32, m.column_width(i) as i32);
        }
        table.set_col_resize(true);

        let simple_model = self.model.clone();
        table.draw_cell(move |t, ctx, row, col, x, y, w, h| {
            match ctx {
                TableContext::StartPage => draw::set_font(enums::Font::Helvetica, 14),
                TableContext::ColHeader => {
                    if let Some(s) = simple_model.borrow_mut().cell(row, col) {
                        Self::draw_header(s.as_str(), x, y, w, h)
                    }
                }
                //TableContext::RowHeader => J1939Table::draw_header(&format!("{}", row + 1), x, y, w, h), // Row titles
                TableContext::RowHeader => {}
                TableContext::Cell => {
                    let txt = simple_model.borrow_mut().cell(row, col).unwrap_or_default();
                    Self::draw_data(txt.as_str(), x, y, w, h, t.is_selected(row, col));
                }
                TableContext::None => {}
                TableContext::EndPage => {}
                TableContext::Table => {}
                TableContext::RcResize => {}
            }
        });
        table.end();
    }
    pub fn redraw(&mut self) {
        let row_count = self.model.borrow().row_count() as i32;
        let mut table = self.table.borrow_mut();
        table.set_rows(row_count);
        table.redraw();
    }
}
