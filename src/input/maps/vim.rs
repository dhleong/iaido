use async_trait::async_trait;

use crate::{input::{Keymap, KeymapContext, KeyCode, Key}, editing::text::TextLines};

pub struct VimKeymap {}

impl Default for VimKeymap {
    fn default() -> Self {
        Self {}
    }
}

#[async_trait]
impl Keymap for VimKeymap {
    async fn process<K: KeymapContext + Send + Sync>(&self, context: &mut K) -> Option<()> {
        loop {
            match context.next_key().await {
                Some(Key { code: KeyCode::Enter, .. }) => {
                    break;
                },
                Some(Key { code, .. }) => {
                    let b = context.state_mut().current_buffer_mut();
                    match code {
                        KeyCode::Char(c) => {
                            b.append(TextLines::raw(c.to_string()));
                        },

                        _ => {},
                    };
                },

                _ => {}
            };
        }
        None
    }
}
