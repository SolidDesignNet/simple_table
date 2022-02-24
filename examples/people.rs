use std::{cell::RefCell, rc::Rc};

use fltk::{
    app,
    group::{Pack, Scroll},
    prelude::*,
    window::Window,
};
use simple_table::simple_table::*;

// Example BusinessObject representing a row
struct Person {
    name: &'static str,
    age: u32,
}

struct PersonModel {
    people: Vec<Person>,
    table: Option<Rc<RefCell<SimpleTable>>>,
}

impl SimpleModel for PersonModel {
    fn set_table(&mut self, table: Rc<RefCell<SimpleTable>>) -> () {
        self.table = Some(table);
    }

    fn row_count(&mut self) -> usize {
        self.people.len()
    }

    fn column_count(&mut self) -> usize {
        2
    }

    fn header(&mut self, col: usize) -> String {
        match col {
            0 => "name".to_string(),
            1 => "age".to_string(),
            _ => "XXX".to_string(),
        }
    }

    fn column_width(&mut self, _col: usize) -> u32 {
        50
    }

    fn cell(&mut self, row: i32, col: i32) -> Option<String> {
        match col {
            0 => Some(self.people[row as usize].name.to_string()),
            1 => Some(self.people[row as usize].age.to_string()),
            _ => None,
        }
    }
}

fn main() {
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
            name: "Mary",
            age: 35,
        },
        Person {
            name: "Judy",
            age: 25,
        },
    ];

    let app = app::App::default();
    let mut wind = Window::default().with_size(200, 300).with_label("Counter");
    // Vertical is default. You can choose horizontal using pack.set_type(PackType::Horizontal);
    let mut pack = Pack::default().with_size(200, 300).center_of(&wind);
    pack.set_spacing(10);
    let scroll = Scroll::default_fill();
    SimpleTable::new(PersonModel {
        people,
        table: None,
    });
    scroll.end();

    pack.end();
    wind.end();
    wind.show();

    app.run().unwrap();
}
