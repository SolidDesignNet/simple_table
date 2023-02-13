use std::time::Instant;

use fltk::{app, prelude::*, window::Window};
use simple_table::simple_table::*;
use timer::Timer;

// Example BusinessObject representing a row
struct Person {
    name: &'static str,
    age: u32,
}

// Example model
struct PersonModel {
    people: Vec<Person>,
    start: Instant,
}

// Example model implementation
impl SimpleModel for PersonModel {
    fn row_count(&mut self) -> usize {
        //self.people.len()
        // demonstration of dynamic size
        Instant::now().duration_since(self.start).as_millis() as usize / 200
    }

    fn column_count(&mut self) -> usize {
        2
    }

    fn header(&mut self, col: usize) -> String {
        match col {
            0 => "Name".to_string(),
            1 => "Age".to_string(),
            _ => "XXX".to_string(),
        }
    }

    fn column_width(&mut self, col: usize) -> u32 {
        match col {
            0 => 120,
            _ => 60,
        }
    }

    fn cell(&mut self, row: i32, col: i32) -> Option<String> {
        if row >= self.people.len() as i32 {
            // make up data outside of defined range
            match col {
                0 => Some(row.to_string()),
                1 => Some((row * row).to_string()),
                _ => None,
            }
        } else {
            // real data example
            match col {
                0 => Some(self.people[row as usize].name.to_string()),
                1 => Some(self.people[row as usize].age.to_string()),
                _ => None,
            }
        }
    }
}

fn main() {
    // data that would normally come from a DB or other source
    let people = vec![
        Person {
            name: "Joe",
            age: 50,
        },
        Person {
            name: "Bob",
            age: 35,
        },
        Person {
            name: "Mary Sue\n Goldstien\n Oquendo\nSmith Orthope",
            age: 35,
        },
        Person {
            name: "Judy",
            age: 25,
        },
    ];

    // create an app with a scroll with a table of PersonModel
    let app = app::App::default();
    let mut wind = Window::default().with_size(200, 300).with_label("Counter");
    let mut table = SimpleTable::new(
        fltk::table::Table::default_fill(),
        Box::new(PersonModel {
            people,
            start: Instant::now(),
        }),
    );
    wind.resizable(&table.table);
    wind.end();
    wind.show();

    // repaint the table on a schedule, to demonstrate updating models.
    let timer = Timer::new(); // requires variable, so that it isn't dropped.
    table.redraw_on(&timer, chrono::Duration::milliseconds(200));

    // run the app
    app.run().unwrap();
}
