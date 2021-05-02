use crate::{
    app::App,
    editing::{
        motion::{linewise::ToLineEndMotion, Motion},
        CursorPosition,
    },
    tui::Tui,
};

pub fn perform_demo(app: &mut App<Tui>) {
    let buffer = app.state.current_buffer_mut();
    buffer.append(tui::text::Text::raw("test 1"));
    buffer.append(tui::text::Text::raw("lorem ipsum dolor sit amet"));
    buffer.append(tui::text::Text::raw("Bacon ipsum dolor amet fatback hamburger capicola, andouille kielbasa prosciutto doner pork loin turducken kevin. Pork belly chislic leberkas ground round cow meatloaf beef. Landjaeger ground round ham chislic brisket buffalo pork loin meatloaf tail drumstick tongue spare ribs."));

    // make sure we have an initial measurement
    app.render();

    let page = app.state.tabpages.current_tab_mut();
    let bottom_id = page.hsplit();

    if let Some(mut bottom_win) = app.state.bufwin_by_id(bottom_id) {
        bottom_win.scroll_lines(1);
        bottom_win.window.cursor = CursorPosition { line: 1, col: 0 }
    }

    ToLineEndMotion.apply_cursor(&mut app.state);
}
