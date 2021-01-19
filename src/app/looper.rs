use async_trait::async_trait;
use futures::{future::FutureExt, select, pin_mut};

use crate::{app::{self, App}, input::KeymapContext, editing::text::TextLines};
use crate::ui::{UI, UiEvent, UiEvents};
use crate::input::{Key, KeyError, Keymap, KeySource};

struct AppKeySource<U: UI, UE: UiEvents> {
    app: App<U>,
    events: UE,
}

#[async_trait]
impl<U: UI + Send + Sync, UE: UiEvents + Send + Sync> KeySource for AppKeySource<U, UE> {
    async fn next_key(&mut self) -> Result<Option<Key>, KeyError> {

        loop {
            self.app.render();

            let event = self.events.next_event().fuse();

            pin_mut!(event);

            select! {
                ev = event => match ev {
                    Ok(UiEvent::Key(key)) => return Ok(Some(key)),
                    _ => {},
                },

                complete => break,
            };
        };

        Ok(None)
    }
}

impl<U: UI + Send + Sync, UE: UiEvents + Send + Sync> KeymapContext for AppKeySource<U, UE> {
    fn state_mut(&mut self) -> &mut app::State {
        &mut self.app.state
    }
}

pub async fn app_loop<U, UE, KM>(
    app: App<U>,
    events: UE,
    map: KM,
)
    where U: UI + Send + Sync,
          UE: UiEvents + Send + Sync,
          KM: Keymap,
{
    let mut app_keys = AppKeySource {
        app,
        events,
    };

    loop {
        if let Err(e) = map.process(&mut app_keys).await {
            let error = format!("IAIDO:ERR: {:?}", e);
            app_keys.state_mut().current_buffer_mut().append(TextLines::raw(error));
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
