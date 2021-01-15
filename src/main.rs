mod app;
mod editing;
mod tui;
mod ui;

use std::io;

fn main() -> Result<(), io::Error> {
    let ui = tui::create_ui()?;
    let state = app::State::default();
    let mut app = app::App::new(state, ui);

    {
        let mut page = app.state.tabpages.current_tab_mut();
        let bottom_id = page.hsplit();

        if let Some(bottom_win) = page.by_id_mut(bottom_id) {
            bottom_win.set_scroll(1, 0);
            bottom_win.set_inserting(true);
        }
    }

    let buffer = app.state.current_buffer_mut();
    buffer.append(tui::text::Text::raw("test 1"));
    buffer.append(tui::text::Text::raw("lorem ipsum dolor sit amet"));
    buffer.append(tui::text::Text::raw("Bacon ipsum dolor amet fatback hamburger capicola, andouille kielbasa prosciutto doner pork loin turducken kevin. Pork belly chislic leberkas ground round cow meatloaf beef. Landjaeger ground round ham chislic brisket buffalo pork loin meatloaf tail drumstick tongue spare ribs."));

    app.render();

    // await any key
    if let Ok(_) = crossterm::event::read() {
        // should we handle an event read error?
    }

    Ok(())
}
