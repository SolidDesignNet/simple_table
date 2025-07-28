use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

use fltk::{
    draw::{draw_frame, draw_text2, set_draw_color},
    enums::{Align, Color},
    frame::Frame,
    group::Scroll,
    prelude::*,
};
use timer::Guard;

use crate::simple_model::SimpleModel;
pub struct JoeTable {
    pub scroll: Scroll,
    pub table: Frame,
}
impl JoeTable {
    pub fn new<T: SimpleModel + 'static>(mut model: T) -> Self {
        let scroll = Scroll::default_fill();
        let mut table = Frame::default_fill();
        let mut s2 = scroll.clone();
        table.draw(move |table| {
            let scroll = &mut s2;
            {
                let width = (0..model.column_count())
                    .map(|c| model.column_width(c) as i32)
                    .sum();
                let height = model
                    .all_row_height()
                    .map(|r| model.row_count() as i32 * r as i32)
                    .unwrap_or_else(|| {
                        (0..model.row_count() as i32)
                            .map(|row| model.row_height(row) as i32)
                            .sum()
                    });
                table.set_size(width, height);
            }
            let first_row = 0;
            let last_row = model.row_count() as i32;
            for row in first_row..last_row {
                let mut x = 0 - scroll.xposition();
                // use the all_row_height if available, otherwise use the row specific height
                let (height, y) = model
                    .all_row_height()
                    .map(|h| (h as i32, h as i32 * row))
                    .unwrap_or_else(|| {
                        (
                            model.row_height(row) as i32,
                            (0..row).map(|row| model.row_height(row) as i32).sum(),
                        )
                    });
                let y = y - scroll.yposition();

                // FIXME could optimize out columns that are not displayed
                for col in 0i32..model.column_count() as i32 {
                    let width = model.column_width(col as usize) as i32;
                    draw_frame("LLTT", x, y, width, height);

                    // should we clip?
                    set_draw_color(Color::Black);
                    let cell = model.cell(row, col).unwrap_or_default();
                    draw_text2(&cell, x, y, width, height, Align::Left);
                    x += width;
                }
            }
        });
        Self { scroll, table }
    }

    /// Redraw using a timer.  When the table is dropped, the timer task will be dropped.
    /// The Timer is passed in, so multiple events can share the timer.
    pub fn redraw_on(&self, timer: &timer::Timer, duration: chrono::Duration) {
        let guard: Arc<Mutex<Option<Guard>>> = Arc::new(Mutex::new(None));
        let table = self.table.clone();
        let mut scroll = self.scroll.clone();
        guard
            .clone()
            .lock()
            .unwrap()
            .replace(timer.schedule_repeating(duration, move || {
                if table.visible_r() {
                    scroll.redraw();
                    fltk::app::awake();
                } else {
                    // No longer visible, so stop timer
                    guard.lock().unwrap().take();
                }
            }));
    }
}

impl Deref for JoeTable {
    type Target = dyn WidgetExt;

    fn deref(&self) -> &Self::Target {
        &self.scroll
    }
}
