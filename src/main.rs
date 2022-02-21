use std::{cell::RefCell, rc::Rc};

use fltk::{
    app,
    group::{Pack, Scroll},
    prelude::*,
    window::Window,
};
use simple_table::{SimpleModel, SimpleTable};

mod simple_table;

struct PersonModel {}

impl SimpleModel for PersonModel {
    fn set_table(&self, _table: &SimpleTable) -> () {
        //todo!()
    }

    fn row_count(&self) -> usize {
        50
    }

    fn column_count(&self) -> usize {
        2
    }

    fn header(&self, col: usize) -> String {
        match col {
            0 => "name".to_string(),
            _ => "age".to_string(),
        }
    }

    fn column_width(&self, _col: usize) -> u32 {
        50
    }

    fn cell(&self, row: i32, col: i32) -> Option<String> {
        println!("cell {} {}", row, col);
        match col {
            0 => Some(format!("name {}", row)),
            _ => Some(format!("age:{}", row)),
        }
    }
}

fn main() {
    let app = app::App::default();
    let mut wind = Window::default().with_size(200, 300).with_label("Counter");
    // Vertical is default. You can choose horizontal using pack.set_type(PackType::Horizontal);
    let mut pack = Pack::default().with_size(200, 300).center_of(&wind);
    pack.set_spacing(10);
    let scroll = Scroll::default_fill();
    let _table = SimpleTable::new(Rc::new(RefCell::new(PersonModel {})));
    scroll.end();

    pack.end();
    wind.end();
    wind.show();

    app.run().unwrap();
}
