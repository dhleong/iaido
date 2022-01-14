use lazy_static::lazy_static;

use std::{sync::Mutex, time::Duration};

use crate::{
    app::{self, App},
    cli::{self, CliInit},
    editing::text::TextLines,
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

use super::{dispatcher::Dispatcher, jobs::Jobs};

struct AppKeySource<U: UI, UE: UiEvents> {
    app: App<U>,
    events: UE,
}

impl<U: UI, UE: UiEvents> AppKeySource<U, UE> {
    fn process_async(
        &mut self,
        keymap: &mut Option<Box<&mut dyn BoxableKeymap>>,
    ) -> Result<bool, KeyError> {
        let mut dirty = false;

        // New main loop processor:
        dirty |= Dispatcher::process(&mut self.app.state)?;

        // process messages from jobs
        dirty |= Jobs::process(&mut self.app.state)?;

        if let Some(ref mut keymap) = keymap {
            // ... and from scripts
            let mut context = CommandHandlerContext::new_blank(self, keymap);
            dirty |= ScriptingManager::process(&mut context)?;
        } else {
            panic!("No keymap provided");
        }

        Ok(dirty)
    }
}

impl<U: UI, UE: UiEvents> KeySource for AppKeySource<U, UE> {
    fn poll_key_with_map(
        &mut self,
        duration: Duration,
        mut keymap: Option<Box<&mut dyn BoxableKeymap>>,
    ) -> Result<bool, KeyError> {
        if self.process_async(&mut keymap)? {
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
        let mut dirty = true;
        loop {
            loop {
                if dirty {
                    self.app.render();
                }

                dirty = self.process_async(&mut keymap)?;

                // Finally, check for input:
                match self.events.poll_event(Duration::from_millis(100))? {
                    Some(_) => break,
                    None => {}
                }
            }

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

            dirty = true;
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
    let error = format!("ERR: {:?}", e);
    for line in error.split("\n") {
        app_keys.state_mut().echom(TextLines::raw(line.to_string()));
    }
}
