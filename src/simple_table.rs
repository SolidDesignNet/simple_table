use std::{
    collections::HashMap,
    ffi::CStr,
    sync::{Arc, Mutex},
};

use fltk::{
    draw::{self, begin_line, end_line, set_draw_color, vertex},
    enums::{self, Color, Event, Font},
    misc::Tooltip,
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
impl Order {
    fn next(&self) -> Order {
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

pub trait DrawDelegate {
    fn draw(&self, row: i32, col: i32, x: i32, y: i32, w: i32, h: i32, selecte: bool);
}

pub trait SimpleModel: Send {
    fn row_count(&mut self) -> usize;
    fn column_count(&mut self) -> usize;
    fn header(&mut self, col: usize) -> String;
    fn column_width(&mut self, col: usize) -> u32;
    // if cell returns None, then cell_delegate is called
    fn cell(&mut self, row: i32, col: i32) -> Option<String>;
    fn cell_delegate(&mut self, _row: i32, _col: i32) -> Option<Box<dyn DrawDelegate>> {
        None
    }
    fn hover(&self, row: i32, col: i32) -> Option<String> {
        None
    }
    fn sort(&mut self, _col: usize, _order: Order) {}
}

pub struct SimpleTable<T>
where
    T: SimpleModel + Send,
{
    pub table: Table,
    pub model: Arc<Mutex<Box<T>>>,

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
static mut TOOLTIP_BUFFER: [u8; 256] = [0; 256];

impl<T> SimpleTable<T>
where
    T: SimpleModel + Send+'static,
{
    pub fn new(mut table: Table, mut model: Box<T>) -> SimpleTable<T> {
        // initialize table
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

        let mut old_sort_col = -1;
        let mut sort_order = Order::Ascending;
        let mut tooltip_cell = (-1, -1);

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
                        m.lock().unwrap().sort(col as usize, sort_order);
                        t.damage();
                        true
                    } else {
                        false
                    }
                }
                Event::Move => {
                    // handle dynamic tooltip
                    // fltk hover support sucks.  Take a look at https://github.com/fltk-rs/fltk-rs/discussions/942#discussioncomment-1881750 to replace with a better popup
                    if let Some((TableContext::Cell, row, col, _)) = t.cursor2rowcol() {
                        if (row, col) != tooltip_cell {
                            tooltip_cell = (row, col);

                            let hover = m.lock().unwrap().hover(row, col);
                            hover.map(|my_string| {
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
                            });
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
                            let (value, dd) = {
                                let mut m = model.lock().unwrap();
                                // otherwise we'll draw the text
                                (
                                    m.cell(row, col).unwrap_or_default(),
                                    m.cell_delegate(row, col),
                                )
                            };
                            draw::push_clip(x, y, w, h);
                            let selected = t.is_selected(row, col);
                            if selected {
                                draw::set_draw_color(enums::Color::from_u32(0x00D3_D3D3));
                            } else {
                                draw::set_draw_color(enums::Color::White);
                            }
                            draw::draw_rectf(x, y, w, h);
                            if dd.is_some() {
                                (*dd.unwrap()).draw(row, col, x, y, w, h, t.is_selected(row, col));
                            } else {
                                let str = value.as_str();
                                let calc_height =
                                    (4 + draw::height()) * (1 + str.matches("\n").count() as i32);
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
        let (rc, cc) = {
            let mut simple_model = self.model.lock().unwrap();
            (simple_model.row_count(), simple_model.column_count())
        };
        self.table.set_rows(rc as i32);
        self.table.set_cols(cc as i32);
        // table.set_damage(true); // FIXME verify that it's requiredS
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
                    {
                        let (rc, cc) = {
                            let mut simple_model = model.lock().unwrap();
                            (simple_model.row_count(), simple_model.column_count())
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
}

fn update_min_height(
    row_heights: &mut HashMap<i32, i32>,
    row: i32,
    calc_height: i32,
    t: &mut Table,
) {
    let height = row_heights.get(&row).map(|x| *x);
    if height.is_none() || calc_height > height.unwrap() {
        //Row height for all cells in row
        t.set_row_height(row, calc_height);
        t.set_damage(true);
        row_heights.insert(row, calc_height);
    }
}

pub struct SparkLine {
    pub data: Vec<f64>,
}
impl SparkLine {
    pub fn new(data: Vec<f64>) -> SparkLine {
        SparkLine { data }
    }
}
impl DrawDelegate for SparkLine {
    fn draw(&self, row: i32, _col: i32, x: i32, y: i32, w: i32, h: i32, _selected: bool) {
        if self.data.len() < 2 {
            return;
        }
        let colors = [Color::Red, Color::Blue, Color::Green];
        let color = colors[row as usize % colors.len()];
        set_draw_color(color);
        let mut max = *self
            .data
            .iter()
            .max_by(|x, y| x.partial_cmp(y).unwrap())
            .unwrap() as f64;
        let mut min = *self
            .data
            .iter()
            .min_by(|x, y| x.partial_cmp(y).unwrap())
            .unwrap() as f64;
        if max == min {
            max = max + 1.0;
            min = min - 1.0;
        }

        let y_ratio = h as f64 / (max - min);
        let x_ratio = w as f64 / self.data.len() as f64;

        let mut x = x as f64;
        let top = (h + y) as f64;

        begin_line();
        for i in &self.data {
            vertex(x, top - ((*i - min) * y_ratio));
            x = x + x_ratio;
        }
        end_line();
    }
}
