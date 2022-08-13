use std::sync::{Arc, Mutex};

use fltk::{
    draw, enums,
    prelude::{GroupExt, TableExt, WidgetBase, WidgetExt},
    table::{Table, TableContext},
};

pub trait SimpleModel {
    fn row_count(&mut self) -> usize;
    fn column_count(&mut self) -> usize;
    fn header(&mut self, col: usize) -> String;
    fn column_width(&mut self, col: usize) -> u32;
    fn cell(&mut self, row: i32, col: i32) -> Option<String>;
}

pub struct SimpleTable {
    pub table: Table,
    pub model: Arc<Mutex<Box<dyn SimpleModel+Send>>>,
}

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

impl SimpleTable {
    pub fn new(mut model: Box<dyn SimpleModel+Send>) -> SimpleTable {
        // initialize table
        let mut table = Table::default_fill();
        {
            table.set_cols(model.column_count() as i32);
            table.set_col_header(true);
            for i in 0..model.column_count() {
                table.set_col_width(i as i32, model.column_width(i) as i32);
            }
            table.set_col_resize(true);
        }

        table.end();

        let mut simple_table = SimpleTable {
            table: table,
            model: Arc::new(Mutex::new(model)),
        };
        {
            let model = simple_table.model.clone();
            simple_table.table.draw_cell(
                move |t: &mut Table,
                      ctx: TableContext,
                      row: i32,
                      col: i32,
                      x: i32,
                      y: i32,
                      w: i32,
                      h: i32| {
                    match ctx {
                        TableContext::StartPage => draw::set_font(enums::Font::Helvetica, 14),
                        TableContext::ColHeader => {
                            let txt = model.lock().unwrap().header(col as usize);
                            draw_header(&txt, x, y, w, h)
                        }
                        //TableContext::RowHeader => J1939Table::draw_header(&format!("{}", row + 1), x, y, w, h), // Row titles
                        TableContext::RowHeader => {}
                        TableContext::Cell => {
                            let mut simple_model = model.lock().unwrap();
                            let cell = simple_model.cell(row, col);
                            let value = cell.unwrap_or_default();
                            let str = value.as_str();
                            let height = draw::height() * (1 + str.matches("\n").count() as i32);
                            if height > h {
                                t.set_row_height(row, height);
                                t.set_damage(true);
                            }
                            draw_data(str, x, y, w, h, t.is_selected(row, col));
                        }
                        TableContext::None => {}
                        TableContext::EndPage => {}
                        TableContext::Table => {}
                        TableContext::RcResize => {}
                    }
                },
            );
        }
        simple_table.redraw();
        simple_table
    }
    
    pub fn reset(&mut self) {
         TableExt::clear(&mut self.table);
         self.redraw();
     }

    pub fn redraw(&mut self) {
        let mut simple_model = self.model.lock().unwrap();
        self.table.set_rows(simple_model.row_count() as i32);
        self.table.set_cols(simple_model.column_count() as i32);
        fltk::app::awake();
    }
}
