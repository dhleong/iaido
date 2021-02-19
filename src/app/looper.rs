use std::time::Duration;

use crate::input::{Key, KeyError, KeySource, Keymap};
use crate::ui::{UiEvent, UiEvents, UI};
use crate::{
    app::{self, App},
    editing::text::TextLines,
    input::KeymapContext,
};

struct AppKeySource<U: UI, UE: UiEvents> {
    app: App<U>,
    events: UE,
}

impl<U: UI, UE: UiEvents> KeySource for AppKeySource<U, UE> {
    fn poll_key(&mut self, duration: Duration) -> Result<bool, KeyError> {
        match self.events.poll_event(duration) {
            Ok(Some(UiEvent::Key(_))) => Ok(true),
            Ok(_) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    fn next_key(&mut self) -> Result<Option<Key>, KeyError> {
        let mut dirty = true;
        loop {
            loop {
                if dirty {
                    self.app.render();
                }

                // process incoming data from connections
                if let Some(mut connections) = self.app.state.connections.take() {
                    dirty = connections.process(&mut self.app.state);
                    self.app.state.connections = Some(connections);
                }

                // TODO: poll other main event loop sources?
                match self.events.poll_event(Duration::from_millis(100))? {
                    Some(_) => break,
                    None => {}
                }
            }

            match self.events.next_event()? {
                UiEvent::Key(key) => return Ok(Some(key)),
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

pub fn app_loop<U, UE, KM>(app: App<U>, events: UE, mut map: KM)
where
    U: UI,
    UE: UiEvents,
    KM: Keymap,
{
    let mut app_keys = AppKeySource { app, events };

    loop {
        if let Err(e) = map.process(&mut app_keys) {
            let error = format!("ERR: {:?}", e);
            app_keys.state_mut().echo(TextLines::raw(error));
            // TODO fatal errors?
            continue;
        }

        // TODO check if we need to change maps, etc.

        if !app_keys.app.state.running {
            // goodbye!
            break;
        }
    }
}
