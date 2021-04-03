mod app;
mod connection;
mod editing;
mod input;
mod tui;
mod ui;

pub mod log;

use app::looper::app_loop;
use backtrace::Backtrace;
use input::{keys::KeysParsable, maps::vim::VimKeymap, RemapMode, Remappable};
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
    app.render();

    let page = app.state.tabpages.current_tab_mut();
    let bottom_id = page.hsplit();

    if let Some(mut bottom_win) = app.state.bufwin_by_id(bottom_id) {
        bottom_win.scroll_lines(1);
        bottom_win.window.cursor = CursorPosition { line: 1, col: 0 }
    }

    {
        ToLineEndMotion.apply_cursor(&mut app.state);
    }

    let mut keymap = VimKeymap::default();
    keymap.remap_keys(
        RemapMode::VimNormal,
        "gc".into_keys(),
        ":connect ".into_keys(),
    );

    app_loop(app, tui::events::TuiEvents::default(), keymap);

    Ok(())
}

struct PanicData {
    info: String,
    trace: Backtrace,
}

fn main() -> io::Result<()> {
    // Prepare to capture any panics
    let panic_data = prepare_panic_capture();

    // Run the main loop inside Tokio
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut result = rt.block_on(async { panic::catch_unwind(main_loop) });

    // Done! Make sure any hanging jobs get killed
    rt.shutdown_timeout(Duration::from_millis(50));

    // If a non-main thread panicked, the result from main_loop will
    // still be `Ok`; we don't usually use the panic err (hopefully
    // we will have captured the data in the mutex above) but we need
    // *something* so we know about the panic
    if *crate::app::looper::PANICKED.lock().unwrap() {
        if let Ok(_) = result {
            result = Err(Box::new("Non-main thread panic!"));
        }
    }

    match result {
        Err(panic) => {
            println!();

            let lock = panic_data.lock().unwrap();
            if let Some(ref data) = *lock {
                println!("PANIC! {:?}", data.info);
                println!("Backtrace:\n {:?}", data.trace);
            } else {
                println!("PANIC! {:?}", panic);
                println!("(backtrace unavailable)");
            }

            Err(io::ErrorKind::Other.into())
        }

        Ok(Ok(_)) => Ok(()),
        Ok(e) => e,
    }
}

fn prepare_panic_capture() -> Arc<Mutex<Option<PanicData>>> {
    let mutex: Arc<Mutex<Option<PanicData>>> = Arc::new(Mutex::new(None));

    let panic_lock = Arc::clone(&mutex);
    panic::set_hook(Box::new(move |info| {
        let mut lock = panic_lock.lock().unwrap();
        let trace = Backtrace::new();

        let mut panicked = crate::app::looper::PANICKED.lock().unwrap();
        *panicked = true;

        *lock = Some(PanicData {
            info: format!("{}", info),
            trace,
        });
    }));

    return mutex;
}
