use std::{
    cmp::Ordering,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
    time::Instant,
    usize,
};

use fltk::{
    app::{self, sleep},
    draw::{draw_frame, draw_rect_fill, draw_text2, pop_clip, push_clip, set_draw_color},
    enums::{Align, Color, Event},
    frame::Frame,
    group::{Group, Pack, PackType, Scroll},
    prelude::*,
};
use timer::Guard;

use crate::simple_model::SimpleModel;

pub struct JoeTable<T: SimpleModel + 'static> {
    pub header: Frame,
    pub scroll: Scroll,
    pub table: Group,
    pub model: Arc<Mutex<T>>,
}

impl<T: SimpleModel + 'static> Clone for JoeTable<T> {
    fn clone(&self) -> Self {
        Self {
            header: self.header.clone(),
            scroll: self.scroll.clone(),
            table: self.table.clone(),
            model: self.model.clone(),
        }
    }
}
impl<T: SimpleModel> Deref for JoeTable<T> {
    type Target = Scroll;

    fn deref(&self) -> &Self::Target {
        &self.scroll
    }
}
impl<T: SimpleModel> DerefMut for JoeTable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scroll
    }
}
impl<T> JoeTable<T>
where
    T: SimpleModel + 'static,
{
    pub fn new(model: T) -> Self {
        let pack = Pack::default_fill().with_type(PackType::Vertical);
        let header = Frame::default_fill();
        let scroll = Scroll::default_fill();
        let mut table = Group::default_fill();
        scroll.end();
        pack.resizable(&scroll);
        pack.end();
        let model = Arc::new(Mutex::new(model));
        let mut this = Self {
            header,
            scroll,
            table: table.clone(),
            model: model.clone(),
        };
        {
            let model = model.clone();
            let this: JoeTable<T> = this.clone();
            table.handle(move |_table, e| {
                if Event::Released == e {
                    if let Some((row, col)) = this.pos_to_row_col(app::event_x(), app::event_y()) {
                        if col == 4 {
                            model
                                .lock()
                                .unwrap()
                                .cell_widget(row, col)
                                .unwrap()
                                .do_callback();
                            return true;
                        }
                    }
                }
                false
            });
        }
        this.init();
        this
    }

    /// Redraw using a timer.  When the table is dropped, the timer task will be dropped.
    /// The Timer is passed in, so multiple events can share the timer.
    pub fn redraw_on(&self, timer: &timer::Timer, duration: chrono::Duration) {
        let guard: Arc<Mutex<Option<Guard>>> = Arc::new(Mutex::new(None));
        let table = self.table.clone();
        let mut scroll = self.scroll.clone();
        let mut header = self.header.clone();
        guard
            .clone()
            .lock()
            .unwrap()
            .replace(timer.schedule_repeating(duration, move || {
                if table.visible_r() {
                    header.redraw();
                    scroll.redraw();
                    fltk::app::awake();
                } else {
                    // No longer visible, so stop timer
                    guard.lock().unwrap().take();
                }
            }));
    }

    fn init(&mut self) {
        self.init_header();
        self.init_table();
    }

    fn init_table(&mut self) {
        let scroll = self.scroll.clone();
        let model = self.model.clone();
        let mut header = self.header.clone();
        self.table.set_callback(|e| eprintln!(" table CB {e:?}"));
        self.table.draw(move |table| {
            let start = Instant::now();
            let mut model = model.lock().unwrap();
            let row_count = model.row_count() as i32;

            {
                // calculate total size for the scrolbar
                let width = (0..model.column_count())
                    .map(|c| model.column_width(c) as i32)
                    .sum();
                let height = model
                    .all_row_height()
                    .map(|r| row_count as i32 * r as i32)
                    .unwrap_or_else(|| {
                        (0..row_count as i32)
                            .map(|row| model.row_height(row) as i32)
                            .sum()
                    });
                table.set_size(width, height);
            }

            // calculate which rows need redrawn
            let (first_row, last_row) = model
                .all_row_height()
                .map(|h| {
                    let h = h as i32;
                    let first = scroll.yposition() / h;
                    let last = 2 + first + scroll.height() / h;
                    (first, i32::min(row_count, last))
                })
                .unwrap_or_else(|| {
                    let mut first = 0;
                    let mut h = scroll.yposition();
                    while h > table.y() {
                        h -= model.row_height(first) as i32;
                        first += 1;
                    }
                    let mut last = first;
                    let mut h = scroll.height();
                    while h >= table.y() {
                        h -= model.row_height(last) as i32;
                        last += 1;
                    }
                    (first, last)
                });
            eprintln!("table size {first_row} - {last_row}");

            for row in first_row..last_row {
                let mut x = table.x();
                // use the all_row_height if available, otherwise use the row specific height
                let height = Self::row_height(&mut model, row);
                let y = table.y() + Self::row_y(&mut model, row);

                // FIXME could optimize out columns that are not displayed
                for col in 0i32..model.column_count() as i32 {
                    let width = model.column_width(col as usize) as i32;
                    draw_frame("LLTT", x, y, width, height);

                    // should we clip?
                    push_clip(x, y, width - 1, height - 1);
                    set_draw_color(Color::Black);
                    if let Some(cell) = model.cell(row, col) {
                        draw_text2(&cell, x, y, width, height, Align::Left);
                    } else if let Some(cell) = model.cell_delegate(row, col) {
                        cell.draw(row, col, x, y, width, height, false);
                    } else if let Some(mut w) = model.cell_widget(row, col) {
                        w.set_pos(x, y);
                        w.set_size(width, height);
                        table.add(&w);
                        table.draw_child(&mut w);
                        table.remove(&w);
                    } else {
                        draw_rect_fill(x, y, width, height, Color::Blue);
                    }
                    pop_clip();
                    x += width;
                }
            }
            header.redraw();

            sleep(0.01);

            eprintln!("JoeTable redraw {:?}", Instant::now().duration_since(start));
        });
    }

    fn init_header(&mut self) {
        let model = self.model.clone();
        let table = self.table.clone();
        self.header.set_size(self.width_total(), 20);
        self.header.draw(move |frame| {
            let mut model = model.lock().unwrap();
            let height = frame.height();
            let mut x = table.x();
            let y = frame.y();
            eprintln!(" header x:{x} height:{height}");
            for col in 0..model.column_count() {
                let width = model.column_width(col) as i32;
                draw_rect_fill(x, y, width, height, Color::White);
                draw_frame("AADD", x, y, width, height);
                // should we clip?
                set_draw_color(Color::Black);
                let cell = model.header(col);
                draw_text2(&cell, x, y, width, height, Align::Left);
                x += width;
            }
        });
    }
    fn width_total(&self) -> i32 {
        let mut model = self.model.lock().unwrap();
        (0..model.column_count())
            .map(|c| model.column_width(c) as i32)
            .sum()
    }

    fn pos_to_row_col(&self, event_x: i32, event_y: i32) -> Option<(i32, i32)> {
        let x = event_x - self.table.x();
        let y = event_y - self.table.y();
        let model = &mut self.model.lock().unwrap();

        let row = bin_search(model.row_count(), &mut |row: usize| {
            let row = row as i32;
            let row_y = Self::row_y(model, row);
            let row_y2 = row_y + Self::row_height(model, row);
            if y.cmp(&row_y) == Ordering::Less {
                Ordering::Less
            } else if y.cmp(&row_y2) == Ordering::Greater {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });

        let column = bin_search(model.column_count(), &mut |col| {
            let col = col as i32;
            let col_x = Self::col_x(model, col);
            let col_x2 = col_x + Self::col_width(model, col);
            if x.cmp(&col_x) == Ordering::Less {
                Ordering::Less
            } else if x.cmp(&col_x2) == Ordering::Greater {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });
        Some((row as i32, column as i32))
    }

    fn col_width(model: &mut std::sync::MutexGuard<'_, T>, col: i32) -> i32 {
        model.column_width(col as usize) as i32
    }
    fn col_x(model: &mut std::sync::MutexGuard<'_, T>, col: i32) -> i32 {
        let mut x = 0;
        for col in 0..col {
            x += model.column_width(col as usize) as i32
        }
        x
    }

    fn row_height(model: &mut std::sync::MutexGuard<'_, T>, row: i32) -> i32 {
        model
            .all_row_height()
            .map(|h| h as i32)
            .unwrap_or_else(|| model.row_height(row) as i32)
    }

    fn row_y(model: &mut std::sync::MutexGuard<'_, T>, row: i32) -> i32 {
        model
            .all_row_height()
            .map(|h| h as i32 * row)
            .unwrap_or_else(|| model.row_height(row) as i32)
    }
}

// replace with library fn when found.  The only known binary search is on slices, which would force us to have an allocation for every row.
fn bin_search(size: usize, measure_fn: &mut impl FnMut(usize) -> Ordering) -> usize {
    let mut left = 0;
    let mut right = size - 1;
    let mut m = 0;
    while left <= right {
        m = left + (right - left) / 2;
        match measure_fn(m) {
            Ordering::Greater => left = m + 1,
            Ordering::Equal => return m,
            Ordering::Less => right = m - 1,
        }
    }
    m
}
fn bin_find(size: usize, measure_fn: &mut impl FnMut(usize) -> Ordering) -> Option<usize> {
    let m = bin_search(size, measure_fn);
    if measure_fn(m) == Ordering::Equal {
        Some(m)
    } else {
        None
    }
}
