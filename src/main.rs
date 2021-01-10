mod app;
mod editing;
mod tui;

use std::io;

use app::App;

fn main() -> Result<(), io::Error> {
    let mut app = App::new();

    {
        let mut page = app.tabpages.current_tab_mut();

        page.hsplit();
    }

    let buffer = app.current_buffer_mut();
    buffer.append(tui::text::Text::raw("test"));

    if let Ok(mut ui) = tui::create_ui() {
        ui.render(&mut app)?
    }

    if let Ok(_) = crossterm::event::read() {
        return Ok(());
    }

    Ok(())
}
