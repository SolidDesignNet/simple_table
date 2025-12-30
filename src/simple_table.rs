#[cfg(feature = "hover")]
use std::ffi::CStr;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[cfg(feature = "hover")]
use fltk::misc::Tooltip;
use fltk::{
    draw::{self},
    enums::{self, Event, Font},
    prelude::{TableExt, WidgetBase, WidgetExt},
    table::{Table, TableContext},
};
use timer::Guard;

use crate::simple_model::{Order, SimpleCell, SimpleModel};

/// Define a FLTK table with a data model
pub struct SimpleTable<T>
where
    T: SimpleModel + Send,
{
    pub table: Table,
    pub model: Arc<Mutex<T>>,

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
#[cfg(feature = "hover")]
static mut TOOLTIP_BUFFER: [u8; 256] = [0; 256];

impl<T> SimpleTable<T>
where
    T: SimpleModel + Send + 'static,
{
    pub fn new(mut table: Table, mut model: T) -> SimpleTable<T> {
        // initialize table
        {
            table.set_cols(model.column_info().details.len() as i32);
            table.set_col_header(true);
            for i in 0..model.column_info().details.len() {
                table.set_col_width(i as i32, model.column_info().details[i].width as i32);
            }
            table.set_col_resize(true);
        }
        let model = Arc::new(Mutex::new(model));
        {
            let model = model.clone();
            let mut old_sort_col = -1;
            let mut sort_order = Order::Ascending;
            #[cfg(feature = "hover")]
            let tooltip_cell = (-1, -1);
            table.handle(move |t, ev: Event| {
                match ev {
                    Event::Push => {
                        // handle sorting
                        if let Some((TableContext::ColHeader, _row, col, _)) = t.cursor2rowcol() {
                            if col != old_sort_col {
                                sort_order = Order::Ascending;
                                old_sort_col = col;
                            } else {
                                sort_order = sort_order.next();
                            }
                            model.lock().unwrap().sort(col as usize, sort_order);
                            t.damage();
                            true
                        } else {
                            false
                        }
                    }
                    #[cfg(feature = "hover")]
                    Event::Move => {
                        // handle dynamic tooltip
                        // fltk hover support sucks.  Take a look at https://github.com/fltk-rs/fltk-rs/discussions/942#discussioncomment-1881750 to replace with a better popup
                        if let Some((TableContext::Cell, row, col, _)) = t.cursor2rowcol() {
                            if (row, col) != tooltip_cell {
                                tooltip_cell = (row, col);

                                let hover = model.lock().unwrap().hover(row, col);
                                if let Some(my_string) = hover {
                                    Tooltip::enable(true);
                                    // Copy char* into global.  FLTK hover uses a static CStr.
                                    unsafe {
                                        TOOLTIP_BUFFER
                                            .as_mut_ptr()
                                            .copy_from(my_string.as_ptr(), my_string.len() + 1)
                                    };

                                    let static_tip = unsafe {
                                        CStr::from_bytes_until_nul(TOOLTIP_BUFFER.as_ref()).unwrap()
                                    };
                                    Tooltip::enter_area(t, 0, 0, 80, 80, static_tip);
                                };
                            }
                        }
                        false
                    }
                    /* other events to be handled */
                    _ => false,
                }
            });
        }
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
                            let column_info = model.lock().unwrap().column_info();
                            let txt = column_info.details[col as usize].header.as_str();
                            draw_header(txt, x, y, w, h)
                        }
                        //TableContext::RowHeader => J1939Table::draw_header(&format!("{}", row + 1), x, y, w, h), // Row titles
                        TableContext::RowHeader => {}
                        TableContext::Cell => {
                            draw::push_clip(x, y, w, h);
                            let selected = t.is_selected(row, col);
                            // FIXME use L&F
                            if selected {
                                draw::set_draw_color(enums::Color::from_u32(0x00D3_D3D3));
                            } else {
                                draw::set_draw_color(enums::Color::White);
                            }
                            draw::draw_rectf(x, y, w, h);
                            match model.lock().unwrap().get_cell(row, col) {
                                SimpleCell::Delegate(dd) => {
                                    dd.draw(row, col, x, y, w, h, t.is_selected(row, col));
                                }
                                SimpleCell::Text(value) => {
                                    let str = value.as_str();
                                    let calc_height = (4 + draw::height())
                                        * (1 + str.matches("\n").count() as i32);
                                    update_min_height(&mut row_heights, row, calc_height, t);
                                    draw::set_draw_color(enums::Color::Gray0);
                                    draw::draw_text2(
                                        str,
                                        x + 2,
                                        y + 2,
                                        w - 4,
                                        h - 4,
                                        enums::Align::Left,
                                    );
                                }
                                SimpleCell::Widget(widget) => todo!(),
                                SimpleCell::None => todo!(),
                            }
                            draw::set_draw_color(enums::Color::Light3);
                            draw::draw_rect(x, y, w, h);
                            draw::pop_clip();
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
        self.table.clear();
        self.redraw();
    }
    pub fn set_font(&mut self, font: enums::Font, size: i32) {
        self.font = font;
        self.font_size = size;
    }

    // Mark for redraw immediately.
    pub fn redraw(&mut self) {
        let (row_count, col_count) = {
            let mut simple_model = self.model.lock().unwrap();
            (
                simple_model.row_info().count,
                simple_model.column_info().details.len(),
            )
        };
        self.table.set_rows(row_count as i32);
        self.table.set_cols(col_count as i32);
        self.table.set_damage(true); // FIXME verify that it's required
        fltk::app::awake();
    }

    /// Redraw using a timer.  When the table is dropped, the timer task will be dropped.
    /// The Timer is passed in, so multiple events can share the timer.
    pub fn redraw_on(&mut self, timer: &timer::Timer, duration: chrono::Duration) {
        let model = self.model.clone();
        let table = Arc::new(Mutex::new(self.table.clone()));
        let guard: Arc<Mutex<Option<Guard>>> = Arc::new(Mutex::new(None));
        guard
            .clone()
            .lock()
            .unwrap()
            .replace(timer.schedule_repeating(duration, move || {
                let mut table = table.lock().unwrap();
                if table.visible_r() {
                    // FIXME why can't this call be made?
                    //self.redraw();
                    {
                        let (rc, cc) = {
                            let mut simple_model = model.lock().unwrap();
                            (
                                simple_model.row_info().count,
                                simple_model.column_info().details.len(),
                            )
                        };
                        table.set_rows(rc as i32);
                        table.set_cols(cc as i32);
                        // table.set_damage(true); // FIXME verify that it's requiredS
                        fltk::app::awake();
                    };
                } else {
                    // No longer visible, so stop timer
                    guard.lock().unwrap().take();
                }
            }));
    }

    pub fn copy(&self, col_delimiter: &str, row_delimier: &str) -> String {
        let model = &mut self.model.lock().unwrap();
        let mut str = String::new();
        for row in 0..(model.row_info().count as i32) {
            for col in 0..(model.column_info().details.len() as i32) {
                let c = model.get_cell(row, col).as_str().unwrap_or("").to_string();
                str.push_str(&c);

                str.push_str(col_delimiter);
            }
            str.push_str(row_delimier);
        }
        str
    }
}

fn update_min_height(
    row_heights: &mut HashMap<i32, i32>,
    row: i32,
    calc_height: i32,
    table: &mut Table,
) {
    let height = row_heights.get(&row).copied();
    if height.is_none() || calc_height > height.unwrap() {
        //Row height for all cells in row
        table.set_row_height(row, calc_height);
        table.set_damage(true);
        row_heights.insert(row, calc_height);
    }
}
