use async_trait::async_trait;
use futures::{future::FutureExt, select, pin_mut};

use crate::{app::{self, App}, input::KeymapContext};
use crate::ui::UI;
use crate::input::{Key, Keymap, KeySource};

struct AppKeySource<'a, U: UI, K: KeySource> {
    app: App<U>,
    keys: &'a mut K,
}

#[async_trait]
impl<'a, U: UI + Send + Sync, K: KeySource + Send + Sync> KeySource for AppKeySource<'a, U, K> {
    async fn next(&mut self) -> Option<Key> {
        loop {
            self.app.render();

            let mut key = self.keys.next().fuse();
            // let event = events.next().fuse();

            // pin_mut!(event);

            select! {
                key = key => match key {
                    Some(key) => return Some(key),
                    _ => {}
                },

                // _ = event => {
                //     // nop; just trigger a redraw
                // },

                complete => break,
            };
        };

        None
    }
}

impl<'a, U: UI + Send + Sync, K: KeySource + Send + Sync> KeymapContext for AppKeySource<'a, U, K> {
    fn state_mut(&mut self) -> &mut app::State {
        &mut self.app.state
    }
}

pub async fn app_loop<U, KM, K>(app: App<U>, map: KM, mut keys: K) where U: UI + Send + Sync, KM: Keymap, K: KeySource + Send + Sync {
    let mut app_keys = AppKeySource {
        app,
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
