mod app;
mod connection;
mod editing;
mod input;
mod tui;
mod ui;

use app::looper::app_loop;
use input::maps::vim::VimKeymap;
use std::io;

use editing::{motion::linewise::ToLineEndMotion, motion::Motion, CursorPosition};

fn main() -> Result<(), io::Error> {
    let ui = tui::create_ui()?;
    let state = app::State::default();
    let mut app = app::App::new(state, ui);

    let buffer = app.state.current_buffer_mut();
    buffer.append(tui::text::Text::raw("test 1"));
    buffer.append(tui::text::Text::raw("lorem ipsum dolor sit amet"));
    buffer.append(tui::text::Text::raw("Bacon ipsum dolor amet fatback hamburger capicola, andouille kielbasa prosciutto doner pork loin turducken kevin. Pork belly chislic leberkas ground round cow meatloaf beef. Landjaeger ground round ham chislic brisket buffalo pork loin meatloaf tail drumstick tongue spare ribs."));

    // make sure we have an initial measurement
    {
        app.state
            .tabpages
            .current_tab_mut()
            .current_window_mut()
            .set_focused(false);
        app.render();
    }

    app.state.echo("Test".into());

    let page = app.state.tabpages.current_tab_mut();
    let bottom_id = page.hsplit();

    if let Some(mut bottom_win) = app.state.bufwin_by_id(bottom_id) {
        bottom_win.scroll_lines(1);
        bottom_win.window.set_inserting(true);
        bottom_win.window.cursor = CursorPosition { line: 1, col: 0 }
    }

    {
        ToLineEndMotion.apply_cursor(&mut app.state);
    }

    app_loop(app, tui::events::TuiEvents::default(), VimKeymap::default());

    Ok(())
}
