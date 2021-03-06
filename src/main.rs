mod app;
mod connection;
mod editing;
mod input;
mod tui;
mod ui;

pub mod log;

use app::looper::app_loop;
use backtrace::Backtrace;
use input::maps::vim::VimKeymap;
use std::{
    io, panic,
    sync::{Arc, Mutex},
    time::Duration,
};

use editing::{motion::linewise::ToLineEndMotion, motion::Motion, CursorPosition};

fn main_loop() -> io::Result<()> {
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

    let page = app.state.tabpages.current_tab_mut();
    let bottom_id = page.hsplit();

    if let Some(mut bottom_win) = app.state.bufwin_by_id(bottom_id) {
        bottom_win.scroll_lines(1);
        bottom_win.window.cursor = CursorPosition { line: 1, col: 0 }
    }

    {
        ToLineEndMotion.apply_cursor(&mut app.state);
    }

    app_loop(app, tui::events::TuiEvents::default(), VimKeymap::default());

    Ok(())
}

struct PanicData {
    info: String,
    trace: Backtrace,
}

fn main() -> io::Result<()> {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mutex: Arc<Mutex<Option<PanicData>>> = Arc::new(Mutex::new(None));
    let panic_lock = Arc::clone(&mutex);
    panic::set_hook(Box::new(move |info| {
        let mut lock = panic_lock.lock().unwrap();
        let trace = Backtrace::new();
        *lock = Some(PanicData {
            info: format!("{}", info),
            trace,
        });
    }));

    let result = rt.block_on(async { panic::catch_unwind(main_loop) });

    // make sure any hanging jobs get killed
    rt.shutdown_timeout(Duration::from_millis(50));

    match result {
        Ok(Ok(_)) => Ok(()),
        Ok(e) => e,
        Err(panic) => {
            println!();

            let lock = mutex.lock().unwrap();
            if let Some(ref data) = *lock {
                println!("PANIC! {:?}", data.info);
                println!("Backtrace:\n {:?}", data.trace);
            } else {
                println!("PANIC! {:?}", panic);
                println!("(backtrace unavailable)");
            }

            Err(io::ErrorKind::Other.into())
        }
    }
}
