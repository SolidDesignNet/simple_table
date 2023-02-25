use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

use chrono::Duration;
use fltk::{
    app,
    draw::{draw_line, draw_point, draw_rect_fill, set_draw_color},
    enums::Color,
    prelude::*,
    window::Window,
};
use simple_table::simple_table::*;
use timer::Timer;

// Example BusinessObject representing a row
struct Signal {
    name: &'static str,
    values: Vec<f32>,
}

// Example model
struct SignalModel {
    signals: Arc<Mutex<Vec<Signal>>>,
}

// Example model implementation
impl SimpleModel for SignalModel {
    fn row_count(&mut self) -> usize {
        self.signals.lock().unwrap().len()
    }

    fn column_count(&mut self) -> usize {
        3
    }

    fn header(&mut self, col: usize) -> String {
        match col {
            0 => "Signal".to_string(),
            1 => "Value".to_string(),
            2 => "Spark".to_string(),
            _ => "XXX".to_string(),
        }
    }

    fn column_width(&mut self, col: usize) -> u32 {
        match col {
            0 => 120,
            1 => 60,
            2 => 240,
            _ => 60,
        }
    }

    fn cell(&mut self, row: i32, col: i32) -> Option<String> {
        match col {
            0 => Some(self.signals.lock().unwrap()[row as usize].name.to_string()),
            1 => Some(
                self.signals.lock().unwrap()[row as usize]
                    .values
                    .last()
                    .unwrap()
                    .to_string(),
            ),
            _ => None,
        }
    }

    fn cell_delegate(&mut self, row: i32, col: i32) -> Option<Box<dyn DrawDelegate>> {
        match col {
            2 => Some(Box::new(SparkLine {
                // we can do better TODO
                data: self.signals.lock().unwrap()[row as usize].values.clone(),
            })),
            _ => None,
        }
    }

    fn sort(&mut self, col: usize, order: Order) {
        //  todo!()
    }
}

pub struct SparkLine {
    data: Vec<f32>,
}

impl DrawDelegate for SparkLine {
    fn draw(&self, row: i32, col: i32, x: i32, y: i32, w: i32, h: i32, selected: bool) {
        let colors = [Color::Red, Color::Blue, Color::Green];
        let color = colors[row as usize % colors.len()];
        draw_rect_fill(x, y, w, h, Color::White);
        set_draw_color(color);
        let max = self
            .data
            .iter()
            .max_by(|x, y| x.partial_cmp(y).unwrap())
            .unwrap();
        let min = self
            .data
            .iter()
            .min_by(|x, y| x.partial_cmp(y).unwrap())
            .unwrap();

        let y_ratio = h as f32 / (max - min);
        let x_ratio = w as f32 / self.data.len() as f32;

        let mut dx = x as f32;

        let mut old_x = x as f32;
        let mut old_y = (h + y) as f32;
        for i in &self.data {
            let dy = (h + y) as f32 - ((i - min) * y_ratio);
            draw_line(old_x as i32, old_y as i32, dx as i32, dy as i32);
            old_x = dx;
            old_y = dy;
            dx = dx + x_ratio;
        }
    }
}

fn main() {
    // data that would normally come from a DB or other source
    let signal_model = SignalModel {
        signals: Arc::new(Mutex::new(vec![
            Signal {
                name: "Joe",
                values: vec![50.0],
            },
            Signal {
                name: "Bob",
                values: vec![35.0],
            },
            Signal {
                name: "Mary Sue\n Goldstien\n Oquendo\nSmith Orthope",
                values: vec![35.0],
            },
            Signal {
                name: "Judy",
                values: vec![25.0],
            },
        ])),
    };
    let mutex = signal_model.signals.clone();
    std::thread::spawn(move || loop {
        mutex.lock().unwrap().iter_mut().for_each(|mut s| {
            let other = s.values.last().unwrap() + (rand::random::<f32>() - 0.5);
            s.values.push(other)
        });
        std::thread::sleep(std::time::Duration::from_secs(1));
    });
    // create an app with a scroll with a table of PersonModel
    let app = app::App::default();
    let mut wind = Window::default().with_size(200, 300).with_label("Counter");
    let mut table = SimpleTable::new(fltk::table::Table::default_fill(), Box::new(signal_model));
    wind.resizable(&table.table);
    wind.end();
    wind.show();

    // repaint the table on a schedule, to demonstrate updating models.
    let timer = Timer::new(); // requires variable, so that it isn't dropped.
    table.redraw_on(&timer, chrono::Duration::milliseconds(200));

    // run the app
    app.run().unwrap();
}
