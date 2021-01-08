mod editing;
mod tui;

use std::rc::Rc;

use crate::tui::window::TuiWindowFactory;
use editing::{buffers::Buffers, tabpage::Tabpage};

fn main() {
    let mut buffers = Buffers::new();
    let windows = Rc::new(TuiWindowFactory {});
    let mut page = Tabpage::new(0, windows, &mut buffers);
    let window = page.current_window();
    let second = page.split();

    println!("Hello, world {} {} {}!", page.id, window, second);
}
