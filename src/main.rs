mod app;
mod cli;
mod connection;
mod demo;
mod editing;
mod game;
mod input;
mod script;
mod tui;
mod ui;

pub mod log;

use crate::ui::backtrace::PanicData;
use app::looper::app_loop;
use backtrace::Backtrace;
use editing::Resizable;
use input::maps::vim::VimKeymap;
use std::{
    io, panic,
    sync::{Arc, Mutex},
    time::Duration,
};

fn main_loop() -> io::Result<()> {
    let args = cli::args();

    let ui = tui::create_ui()?;
    let state = app::State::default();
    let mut app = app::App::new(state, ui);

    // Ensure app state has the initial size
    app.state.resize(app.ui.size()?);

    if args.demo {
        demo::perform_demo(&mut app);
    }

    let dispatcher = app.state.dispatcher.sender.clone();
    app_loop(
        app,
        tui::events::TuiEvents::start_with_dispatcher(dispatcher),
        VimKeymap::default(),
        args,
    );

    Ok(())
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
                println!("Backtrace:\n {}", data);
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
