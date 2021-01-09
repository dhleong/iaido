mod app;
mod editing;
mod tui;

use crate::tui::window::TuiWindowFactory;
use app::App;

fn main() {
    let mut app = App::new(&TuiWindowFactory {});

    let mut page = app.tabpages.current_tab_mut();
    {
        let window = page.current_window();
        println!("window = {}", window);
    }

    let second_id = page.split();
    let second = page.by_id(second_id).unwrap();
    println!("Hello, world {} {} {}!", page.id, second_id, second);
}
