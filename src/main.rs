mod app;
mod editing;
mod tui;

use std::io;

use app::App;

fn main() -> Result<(), io::Error> {
    let mut app = App::new();

    {
        let mut page = app.tabpages.current_tab_mut();
        let bottom_id = page.hsplit();

        if let Some(bottom_win) = page.by_id_mut(bottom_id) {
            bottom_win.set_scroll(1, 0);
            // bottom_win.set_inserting(true);
        }
    }

    let buffer = app.current_buffer_mut();
    buffer.append(tui::text::Text::raw("test 1"));
    buffer.append(tui::text::Text::raw("lorem ipsum dolor sit amet"));

    if let Ok(mut ui) = tui::create_ui() {
        ui.render(&mut app)?
    }

    if let Ok(_) = crossterm::event::read() {
        return Ok(());
    }

    Ok(())
}
