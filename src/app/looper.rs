use lazy_static::lazy_static;

use std::{sync::Mutex, time::Duration};

use crate::{
    app::{self, App},
    cli::{self, CliInit},
    input::{
        commands::{connection::connect, CommandHandlerContext},
        maps::KeyResult,
        KeymapContext,
    },
};
use crate::{
    input::BoxableKeymap,
    ui::{UiEvent, UiEvents, UI},
};
use crate::{
    input::{Key, KeyError, KeySource, Keymap},
    script::ScriptingManager,
};

use super::dispatcher::Dispatcher;

struct AppKeySource<U: UI, UE: UiEvents> {
    app: App<U>,
    events: UE,
}

impl<U: UI, UE: UiEvents> KeySource for AppKeySource<U, UE> {
    fn poll_key_with_map(
        &mut self,
        duration: Duration,
        mut keymap: Option<Box<&mut dyn BoxableKeymap>>,
    ) -> Result<bool, KeyError> {
        let keymap = keymap.as_mut().expect("No keymap provided");
        let mut context = CommandHandlerContext::new_blank(self, keymap);
        let processed_events = Dispatcher::process_pending(&mut context)?;
        if processed_events > 0 {
            self.app.render();
        }

        match self.events.poll_event(duration) {
            Ok(Some(UiEvent::Key(_))) => Ok(true),
            Ok(_) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    fn next_key_with_map(
        &mut self,
        mut keymap: Option<Box<&mut dyn BoxableKeymap>>,
    ) -> Result<Option<Key>, KeyError> {
        loop {
            self.app.render();

            // NOTE: The Dispatcher will be notified when a key is pressed,
            // so this is effectively a sleep until there *might* be a
            // pending key
            let keymap = keymap.as_mut().expect("No keymap provided");
            let mut context = CommandHandlerContext::new_blank(self, keymap);
            let processed_events = Dispatcher::process_chunk(&mut context)?;

            // Check if there's a pending key
            match self.events.poll_event(Duration::from_millis(0)) {
                Err(_) => return Ok(None),
                Ok(None) => continue, // No pending key; go back to sleep
                _ => {}               // Pending key! Fall through to consume
            }

            // If there was just one event and there's a pending key, that
            // event was just from the key---nothing else happened
            let dirty = processed_events > 1;

            match self.events.next_event()? {
                UiEvent::Key(key) => {
                    // If dirty, render one more time before returning the key
                    if dirty {
                        self.app.render();
                    }
                    return Ok(Some(key));
                }
                _ => {}
            }
        }
    }
}

impl<U: UI, UE: UiEvents> KeymapContext for AppKeySource<U, UE> {
    fn state(&self) -> &app::State {
        &self.app.state
    }
    fn state_mut(&mut self) -> &mut app::State {
        &mut self.app.state
    }
}

lazy_static! {
    /// Tracks whether a non-main thread panicked
    pub static ref PANICKED: Mutex<bool> = Mutex::new(false);
}

pub fn app_loop<U, UE, KM>(app: App<U>, events: UE, mut map: KM, args: cli::Args)
where
    U: UI,
    UE: UiEvents,
    KM: Keymap + BoxableKeymap,
{
    let mut app_keys = AppKeySource { app, events };

    // Initializing scripting first:
    ScriptingManager::init(&mut app_keys, &mut map);

    // Perform any CLI-arg-driven init:
    if let Err(e) = handle_args(&mut app_keys, &mut map, args) {
        print_error(&mut app_keys, e);
    }

    // Main app loop:
    run_loop(&mut app_keys, map);

    // kill any still-running jobs when the user wants to quit:
    app_keys.state_mut().jobs.cancel_all();
}

fn handle_args<U, UE, KM>(
    app_keys: &mut AppKeySource<U, UE>,
    map: &mut KM,
    args: cli::Args,
) -> KeyResult
where
    U: UI,
    UE: UiEvents,
    KM: Keymap + BoxableKeymap,
{
    match args.init {
        Some(CliInit::Uri(uri)) => {
            let mut ctx = CommandHandlerContext::new_blank(app_keys, map);
            connect(&mut ctx, uri.to_string())?;
        }
        Some(CliInit::ScriptFile(path)) => {
            ScriptingManager::load_script(app_keys, map, path);
        }
        None => {} // nop
    }

    Ok(())
}

fn run_loop<U, UE, KM>(app_keys: &mut AppKeySource<U, UE>, mut map: KM)
where
    U: UI,
    UE: UiEvents,
    KM: Keymap + BoxableKeymap,
{
    loop {
        if let Err(e) = map.process(app_keys) {
            // TODO fatal errors?
            print_error(app_keys, e);
            continue;
        }

        // TODO check if we need to change maps, etc.

        let panicked = *PANICKED.lock().unwrap();
        if !app_keys.app.state.running || panicked {
            // goodbye!
            break;
        }
    }
}

fn print_error<U, UE>(app_keys: &mut AppKeySource<U, UE>, e: KeyError)
where
    U: UI,
    UE: UiEvents,
{
    app_keys.state_mut().echom_error(e);
}
