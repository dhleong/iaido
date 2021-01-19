use async_trait::async_trait;
use futures::{future::FutureExt, select, pin_mut};

use crate::{app::{self, App}, input::KeymapContext};
use crate::ui::{UI, UiEvent, UiEvents};
use crate::input::{Key, Keymap, KeySource};

struct AppKeySource<U: UI, UE: UiEvents> {
    app: App<U>,
    events: UE,
}

#[async_trait]
impl<U: UI + Send + Sync, UE: UiEvents + Send + Sync> KeySource for AppKeySource<U, UE> {
    async fn next_key(&mut self) -> Option<Key> {

        loop {
            self.app.render();

            let event = self.events.next_event().fuse();

            pin_mut!(event);

            select! {
                ev = event => match ev {
                    Some(UiEvent::Key(key)) => return Some(key),
                    _ => {},
                },

                complete => break,
            };
        };

        None
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
        if let Some(()) = map.process(&mut app_keys).await {
            // continue
        } else {
            break;
        }
    }
}
