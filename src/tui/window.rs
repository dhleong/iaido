use super::Renderable;
use crate::editing::window::Window;

impl Renderable for Window {
    fn render(&self, display: &mut super::Display) {
        // todo!()
        println!("test {}", display);
    }
}
