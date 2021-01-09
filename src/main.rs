mod app;
mod editing;
mod tui;

use crate::tui::{Display, Renderable};
use app::App;
use editing::Size;

fn main() {
    let app = App::new();

    // {
    //     let mut page = app.tabpages.current_tab_mut();
    //     {
    //         let window = page.current_window();
    //         println!("window = {}", window);
    //     }
    //
    //     let second_id = page.split();
    //     let second = page.by_id(second_id).unwrap();
    //     println!("Hello, world {} {} {}!", page.id, second_id, second);
    // }

    {
        let mut display = Display {
            size: Size { w: 40, h: 40 },
        };
        app.tabpages.render(&mut display);
    }
}
