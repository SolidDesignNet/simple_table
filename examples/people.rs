use std::time::Instant;

use fltk::{app, prelude::*, window::Window};
use fltk_theme::{SchemeType, WidgetScheme};
use simple_table::{
    simple_model::{Order, SimpleModel},
    simple_table::*,
};
use timer::Timer;

/// Example BusinessObject representing a row
struct Person {
    name: &'static str,
    age: u32,
}

/// Example model
struct PersonModel {
    people: Vec<Person>,
    start: Instant,
}

/// Example model implementation
/// Just displays some names, then numbers.  Demonstrates a multiline cell, dynamically added cells, and sorting.
impl SimpleModel for PersonModel {
    fn sort(&mut self, col: usize, order: Order) {
        self.people.sort_by(|a, b| {
            order.apply(match col {
                0 => a.name.cmp(b.name),
                1 => a.age.cmp(&b.age),
                _ => std::cmp::Ordering::Equal,
            })
        });
    }

    fn row_info(&mut self) -> simple_table::simple_model::RowInfo {
        let count = Instant::now().duration_since(self.start).as_millis() / 200;
        simple_table::simple_model::RowInfo {
            count: self.people.len() + count as usize,
            height: simple_table::simple_model::RowHeight::All(40),
        }
    }

    fn column_info(&mut self) -> simple_table::simple_model::ColumnInfo {
        simple_table::simple_model::ColumnInfo {
            details: vec![
                simple_table::simple_model::ColumnDetail {
                    header: "Name".to_string(),
                    width: 240,
                },
                simple_table::simple_model::ColumnDetail {
                    header: "Age".to_string(),
                    width: 60,
                },
            ],
        }
    }

    fn get_cell(&mut self, row: i32, col: i32) -> simple_table::simple_model::SimpleCell {
        if row >= self.people.len() as i32 {
            // make up data outside of defined range
            match col {
                0 => simple_table::simple_model::SimpleCell::Text(row.to_string()),
                1 => simple_table::simple_model::SimpleCell::Text((row * row).to_string()),
                _ => simple_table::simple_model::SimpleCell::None,
            }
        } else {
            // real data example
            match col {
                0 => simple_table::simple_model::SimpleCell::Text(
                    self.people[row as usize].name.to_string(),
                ),
                1 => simple_table::simple_model::SimpleCell::Text(
                    self.people[row as usize].age.to_string(),
                ),
                _ => simple_table::simple_model::SimpleCell::None,
            }
        }
    }
}

fn main() {
    WidgetScheme::new(SchemeType::SvgBased).apply();
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
        PersonModel {
            people,
            start: Instant::now(),
        },
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
