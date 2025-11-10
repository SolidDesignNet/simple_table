use fltk::{draw::{begin_line, end_line, set_draw_color, vertex}, enums::Color};

use crate::simple_model::DrawDelegate;

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
        let mut max = self
            .data
            .iter()
            .max_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal))
            .copied()
            .unwrap_or(0.0);
        let mut min = self
            .data
            .iter()
            .min_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal))
            .copied()
            .unwrap_or(0.0);
        if max == min {
            max += 1.0;
            min -= 1.0;
        }

        let y_ratio = h as f64 / (max - min);
        let x_ratio = w as f64 / self.data.len() as f64;

        let mut x = x as f64;
        let top = (h + y) as f64;

        begin_line();
        for i in &self.data {
            vertex(x, top - ((*i - min) * y_ratio));
            x += x_ratio;
        }
        end_line();
    }
}
