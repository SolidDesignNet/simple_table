use std::sync::{Arc, Mutex};

use fltk::{app, prelude::*, window::Window};
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
            2 => Some(Box::new(SparkLine::new(
                // we can do better TODO
                self.signals.lock().unwrap()[row as usize].values.clone(),
            ))),
            _ => None,
        }
    }

    fn sort(&mut self, _col: usize, _order: Order) {
        //  todo!()
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
        mutex.lock().unwrap().iter_mut().for_each(|s| {
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
