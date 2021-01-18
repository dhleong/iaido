use async_trait::async_trait;
use futures::{future::FutureExt, select, pin_mut};

use crate::{app::{self, App}, input::KeymapContext};
use crate::ui::{UI, UiEvent, UiEvents};
use crate::input::{Key, Keymap, KeySource};

struct AppKeySource<'a, U: UI, UE: UiEvents, K: KeySource> {
    app: App<U>,
    events: UE,
    keys: &'a mut K,
}

#[async_trait]
impl<'a, U: UI + Send + Sync, UE: UiEvents + Send + Sync, K: KeySource + Send + Sync> KeySource for AppKeySource<'a, U, UE, K> {
    async fn next(&mut self) -> Option<Key> {
        loop {
            self.app.render();

            let mut key = self.keys.next().fuse();
            let event = self.events.next().fuse();

            pin_mut!(event);

            select! {
                key = key => match key {
                    Some(key) => return Some(key),
                    _ => {}
                },

                ev = event => match ev {
                    Some(UiEvent::Redraw) => {}, // nop; just loop and redraw
                    None => {}, // ?
                },

                complete => break,
            };
        };

        None
    }
}

impl<'a, U: UI + Send + Sync, UE: UiEvents + Send + Sync, K: KeySource + Send + Sync> KeymapContext for AppKeySource<'a, U, UE, K> {
    fn state_mut(&mut self) -> &mut app::State {
        &mut self.app.state
    }
}

pub async fn app_loop<U, UE, K, KM>(
    app: App<U>,
    events: UE,
    mut keys: K,
    map: KM,
)
    where U: UI + Send + Sync,
          UE: UiEvents + Send + Sync,
          K: KeySource + Send + Sync,
          KM: Keymap,
{
    let mut app_keys = AppKeySource {
        app,
        events,
        keys: &mut keys,
    };

    loop {
        if let Some(()) = map.process(&mut app_keys).await {
            // continue
        } else {
            break;
        }
    }
}
