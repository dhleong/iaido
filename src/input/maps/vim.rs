use crate::{
    editing::text::TextLines,
    input::{Key, KeyCode, KeyError, Keymap, KeymapContext},
};

pub struct VimKeymap {}

impl Default for VimKeymap {
    fn default() -> Self {
        Self {}
    }
}

impl Keymap for VimKeymap {
    fn process<K: KeymapContext>(&self, context: &mut K) -> Result<(), KeyError> {
        loop {
            match context.next_key()? {
                Some(Key {
                    code: KeyCode::Enter,
                    ..
                }) => {
                    context.state_mut().running = false;
                    return Ok(());
                }

                Some(Key { code, .. }) => {
                    let b = context.state_mut().current_buffer_mut();
                    match code {
                        KeyCode::Char(c) => {
                            b.append(TextLines::raw(c.to_string()));
                        }

                        _ => {}
                    };
                }

                _ => {}
            };
        }
    }
}
