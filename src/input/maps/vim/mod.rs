mod normal;

use std::collections::HashMap;

use async_trait::async_trait;
use normal::vim_normal_mode;

use crate::{input::{KeyError, Keymap, KeymapContext, KeyCode, Key}, editing::text::TextLines};

type KeyHandler = super::KeyHandler<'static, VimKeymapState>;

pub struct VimKeymap {
    normal: KeyTreeNode,
}

impl Default for VimKeymap {
    fn default() -> Self {
        Self {
            normal: vim_normal_mode(),
        }
    }
}

pub struct VimKeymapState {
    pub pending_motion_action_key: Option<Key>,
}

impl Default for VimKeymapState {
    fn default() -> Self {
        Self {
            pending_motion_action_key: None,
        }
    }
}

#[async_trait]
impl Keymap for VimKeymap {
    async fn process<K: KeymapContext + Send + Sync>(&self, context: &mut K) -> Result<(), KeyError> {
        loop {
            match context.next_key().await? {
                Some(Key { code: KeyCode::Enter, .. }) => {
                    context.state_mut().running = false;
                    return Ok(())
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
    }
}

pub struct KeyTreeNode {
    children: HashMap<Key, KeyTreeNode>,
    handler: Option<Box<KeyHandler>>,
}

impl KeyTreeNode {
    pub fn root() -> Self {
        Self {
            children: HashMap::new(),
            handler: None,
        }
    }

    pub fn insert(&mut self, keys: &[Key], handler: Box<KeyHandler>) {
        if keys.is_empty() {
            self.handler = Some(handler);
        } else {
            let first_key = keys[0];
            let node = self.children.entry(first_key).or_insert(KeyTreeNode::root());
            node.insert(&keys[1..], handler);
        }
    }
}

/// Syntactic sugar for declaring a key handler
#[macro_export]
macro_rules! key_handler {
    (|$ctx_name:ident| $body:expr) => {{
        use futures::FutureExt;
        use crate::input::KeymapContext;
        Box::new(|$ctx_name: &mut crate::input::maps::KeyHandlerContext<crate::input::maps::vim::VimKeymapState>| async move {
            let result: crate::input::maps::KeyResult = $body;
            result
        }.boxed_local())
    }};
}
