use lazy_static::lazy_static;

use std::{sync::Mutex, time::Duration};

use crate::{
    app::{self, App},
    editing::text::TextLines,
    input::{commands::CommandHandlerContext, KeymapContext},
};
use crate::{
    input::BoxableKeymap,
    ui::{UiEvent, UiEvents, UI},
};
use crate::{
    input::{Key, KeyError, KeySource, Keymap},
    script::ScriptingManager,
};

use super::jobs::Jobs;

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

        // process incoming data from connections
        if let Some(mut connections) = self.app.state.connections.take() {
            dirty |= connections.process(&mut self.app.state);
            self.app.state.connections = Some(connections);
        }

        // process messages from jobs
        dirty |= Jobs::process(&mut self.app.state)?;

        if let Some(ref mut keymap) = keymap {
            // ... and from scripts
            let mut context = CommandHandlerContext::new(self, keymap, "".to_string());
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

                // finally, check for input:
                match self.events.poll_event(Duration::from_millis(10))? {
                    Some(_) => break,
                    None => {}
                }
            }

            match self.events.next_event()? {
                UiEvent::Key(key) => {
                    // if dirty, render one more time before returning the key
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

pub fn app_loop<U, UE, KM>(app: App<U>, events: UE, mut map: KM)
where
    U: UI,
    UE: UiEvents,
    KM: Keymap + BoxableKeymap,
{
    let mut app_keys = AppKeySource { app, events };

    ScriptingManager::init(&mut app_keys, &mut map);

    loop {
        if let Err(e) = map.process(&mut app_keys) {
            let error = format!("ERR: {:?}", e);
            for line in error.split("\n") {
                app_keys.state_mut().echom(TextLines::raw(line.to_string()));
            }
            // TODO fatal errors?
            continue;
        }

        // TODO check if we need to change maps, etc.

        let panicked = *PANICKED.lock().unwrap();
        if !app_keys.app.state.running || panicked {
            // goodbye!
            break;
        }
    }

    // kill any still-running jobs when the user wants to quit:
    app_keys.state_mut().jobs.cancel_all();
}
