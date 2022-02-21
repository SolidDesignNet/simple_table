use std::{cell::RefCell, rc::Rc};

use fltk::{
    app,
    button::Button,
    frame::Frame,
    group::{Pack, Scroll},
    prelude::*,
    window::Window,
};
use fltk_evented::Listener;
use simple_table::{SimpleModel, SimpleTable};

mod simple_table;

struct Person {
    first: String,
    age: i32,
}

struct PersonModel {}

impl SimpleModel<Person> for PersonModel {
    fn set_table(&self, table: Rc<RefCell<simple_table::SimpleTable<Person>>>) -> () {
        todo!()
    }

    fn row_count(&self) -> usize {
        5
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
        20
    }

    fn cell(&self, row: i32, col: i32) -> Option<String> {
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
    let mut but_inc: Listener<_> = Button::default().with_size(0, 40).with_label("+").into();
    let frame = Frame::default().with_size(0, 40).with_label("0");
    let mut but_dec: Listener<_> = Button::default().with_size(0, 40).with_label("-").into();

    let scroll = Scroll::default().with_size(100, 100);
    let _table = SimpleTable::new(Rc::new(RefCell::new(PersonModel {})));
    scroll.end();

    pack.end();
    wind.end();
    wind.show();

    let count = Rc::new(RefCell::new(0));
    let frame = Rc::new(RefCell::new(frame));
    {
        let c = count.clone();
        let f = frame.clone();
        but_inc.on_click(move |_b| {
            c.replace_with(|&mut old| old + 1);
            (f.as_ref().borrow_mut()).set_label(&c.borrow().to_string());
        });
    }
    {
        let c = count.clone();
        let f = frame.clone();
        but_dec.on_click(move |_b| {
            c.replace_with(|&mut old| old - 1);
            f.as_ref().borrow_mut().set_label(&c.borrow().to_string());
        });
    }

    app.run().unwrap();
}
