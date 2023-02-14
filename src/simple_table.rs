use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use fltk::{
    draw,
    enums::{self, Event, Font},
    prelude::{TableExt, WidgetBase, WidgetExt},
    table::{Table, TableContext},
};
use timer::Guard;

#[derive(Debug, Clone, Copy)]
pub enum Order {
    Ascending,
    Descending,
    None,
}
pub trait SimpleModel: Send {
    fn row_count(&mut self) -> usize;
    fn column_count(&mut self) -> usize;
    fn header(&mut self, col: usize) -> String;
    fn column_width(&mut self, col: usize) -> u32;
    fn cell(&mut self, row: i32, col: i32) -> Option<String>;
    fn sort(&mut self, col: usize, order: Order);
}

pub struct SimpleTable {
    pub table: Table,
    pub model: Arc<Mutex<Box<dyn SimpleModel + Send>>>,

    font: Font,
    font_size: i32,
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
    draw::draw_text2(txt, x, y, w, h, enums::Align::Left);
    draw::set_draw_color(enums::Color::Light3);
    draw::draw_rect(x, y, w, h);
    draw::pop_clip();
}

impl SimpleTable {
    pub fn new(mut table: Table, mut model: Box<dyn SimpleModel + Send>) -> SimpleTable {
        // initialize table
        //let mut table = Table::default();
        {
            table.set_cols(model.column_count() as i32);
            table.set_col_header(true);
            for i in 0..model.column_count() {
                table.set_col_width(i as i32, model.column_width(i) as i32);
            }
            table.set_col_resize(true);
        }
        let model = Arc::new(Mutex::new(model));
        let m = model.clone();
        let t = table.clone();

        let mut old_col = -1;
        let mut order = Order::Ascending;
        table.handle(move |widget, ev: Event| {
            match ev {
                Event::Push => {
                    if let Some(click) = widget.cursor2rowcol() {
                        if click.0 == TableContext::ColHeader {
                            let col = click.2;
                            eprintln!("sorting {} {:?} old {}", col, order, old_col);
                            if col != old_col {
                                order = Order::Ascending;
                                old_col = col;
                            } else {
                                order = match order {
                                    Order::Ascending => Order::Descending,
                                    Order::Descending => Order::None,
                                    Order::None => Order::Ascending,
                                };
                            }
                            m.lock().unwrap().sort(col as usize, order);
                            t.damage();
                            return true;
                        }
                    }
                    false
                }
                /* other events to be handled */
                _ => false,
            }
        });
        let mut simple_table = SimpleTable {
            table,
            font: enums::Font::Courier,
            font_size: 12,
            model,
        };
        {
            let model = simple_table.model.clone();
            let font = simple_table.font;
            let font_size = simple_table.font_size;
            let mut row_heights: HashMap<i32, i32> = HashMap::new();
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
                        TableContext::StartPage => draw::set_font(font, font_size),
                        TableContext::ColHeader => {
                            let txt = model.lock().unwrap().header(col as usize);
                            draw_header(&txt, x, y, w, h)
                        }
                        //TableContext::RowHeader => J1939Table::draw_header(&format!("{}", row + 1), x, y, w, h), // Row titles
                        TableContext::RowHeader => {}
                        TableContext::Cell => {
                            let value = model.lock().unwrap().cell(row, col).unwrap_or_default();
                            let str = value.as_str();
                            let calc_height =
                                draw::height() * (1 + str.matches("\n").count() as i32);
                            let height = row_heights.get(&row).map(|x| *x);
                            if height.is_none() || calc_height > height.unwrap() {
                                //Row height for all cells in row
                                t.set_row_height(row, calc_height);
                                t.set_damage(true);
                                row_heights.insert(row, calc_height);
                            }
                            draw_data(str, x, y, w, h, t.is_selected(row, col));
                        }
                        TableContext::None => {}
                        TableContext::EndPage => {}
                        TableContext::Table => {}
                        TableContext::RcResize => {
                            row_heights.clear();
                        }
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
    pub fn set_font(&mut self, font: enums::Font, size: i32) {
        self.font = font;
        self.font_size = size;
    }

    // Mark for redraw immediately.
    pub fn redraw(&mut self) {
        redraw_impl(self.model.clone(), self.table.clone());
    }

    /// Redraw using a timer.  When the table is dropped, the timer task will be dropped.
    /// The Timer is passed in, so multiple events can share the timer.
    pub fn redraw_on(&mut self, timer: &timer::Timer, duration: chrono::Duration) {
        let model = self.model.clone();
        let table = self.table.clone();
        let guard: Arc<Mutex<Option<Guard>>> = Arc::new(Mutex::new(None));
        guard
            .clone()
            .lock()
            .unwrap()
            .replace(timer.schedule_repeating(duration, move || {
                let model = model.clone();
                let table = table.clone();
                if table.visible_r() {
                    // update from fltk thread
                    fltk::app::awake_callback(move || redraw_impl(model.clone(), table.clone()));
                } else {
                    // No longer visible, so stop timer
                    guard.lock().unwrap().take();
                }
            }));
    }
}

/// Private call that sets up for a redraw of the table.
fn redraw_impl(model: Arc<Mutex<Box<dyn SimpleModel + Send>>>, mut table: Table) {
    let (rc, cc) = {
        let mut simple_model = model.lock().unwrap();
        (simple_model.row_count(), simple_model.column_count())
    };
    table.set_rows(rc as i32);
    table.set_cols(cc as i32);
    // table.set_damage(true); // FIXME verify that it's requiredS
    fltk::app::awake();
}
