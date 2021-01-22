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
    fn next_key(&mut self) -> Result<Option<Key>, KeyError> {
        loop {
            self.app.render();

            loop {
                match self.events.poll_event(Duration::from_millis(100)) {
                    Ok(result) if result => break,
                    Err(e) => return Err(e.into()),
                    _ => {}
                }
                // TODO: poll other main event loop sources?
            }

            match self.events.next_event() {
                Ok(UiEvent::Key(key)) => return Ok(Some(key)),
                Err(e) => return Err(e.into()),
                _ => {}
            }
        }
    }
}

impl<U: UI, UE: UiEvents> KeymapContext for AppKeySource<U, UE> {
    fn state_mut(&mut self) -> &mut app::State {
        &mut self.app.state
    }
}

pub fn app_loop<U, UE, KM>(app: App<U>, events: UE, map: KM)
where
    U: UI,
    UE: UiEvents,
    KM: Keymap,
{
    let mut app_keys = AppKeySource { app, events };

    loop {
        if let Err(e) = map.process(&mut app_keys) {
            let error = format!("IAIDO:ERR: {:?}", e);
            app_keys
                .state_mut()
                .current_buffer_mut()
                .append(TextLines::raw(error));
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
