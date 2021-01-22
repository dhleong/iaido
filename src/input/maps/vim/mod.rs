mod normal;

use std::{collections::HashMap, sync::Arc, sync::Mutex};

use async_trait::async_trait;
use normal::vim_normal_mode;

use crate::input::{KeyError, Keymap, KeymapContext, Key, KeySource};

use super::KeyHandlerContext;

type KeyHandler<'a> = super::KeyHandler<'a, VimKeymapState>;

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

pub struct VimKeymap {
    normal: KeyTreeNode,
}

impl VimKeymap {
}

impl Default for VimKeymap {
    fn default() -> Self {
        Self {
            normal: vim_normal_mode(),
        }
    }
}

#[async_trait]
impl Keymap for VimKeymap {
    async fn process<K: KeymapContext + Send + Sync + 'static>(&self, context: &'static mut K) -> Result<(), KeyError> {
        // let mut tree = if context.state().current_window().inserting {
        //     // TODO insert mode:
        //     &self.normal
        // } else {
        //     &self.normal
        // };
        let root = vim_normal_mode();
        let mut tree = &root;

        let mut state = VimKeymapState::default();
        let mut handler_context = Arc::new(Mutex::new(KeyHandlerContext {
            context: Box::new(context),
            state: &mut state,
        }));

        loop {
            // TODO timeout when tree is ambiguous
            let key_result = async {
                let mutex = handler_context.clone();
                let mut ctx = mutex.lock().unwrap();
                ctx.next_key().await
            }.await;
            if let Some(key) = key_result? {
                if let Some(next) = tree.children.get(&key) {
                    if let Some(handler) = &next.handler {
                        invoke(handler_context.clone(), handler).await;
                        break;
                    } else {
                        // TODO set "pending chars" UI state
                        tree = next;
                        continue;
                    }
                }
            }
        }

        Ok(())
    }
}

async fn invoke<'a>(
    handler_context: Arc<Mutex<KeyHandlerContext<'a, VimKeymapState>>>,
    handler: &'a KeyHandler<'a>,
) -> Result<(), KeyError> {
    // let state = VimKeymapState::default(); // TODO
    // let mut handler_context = KeyHandlerContext {
    //     context: Box::new(context),
    //     state: &mut state,
    // };

    // FIXME: handle errors
    let mut ctx = handler_context.lock().unwrap();
    if let Err(e) = handler(&mut ctx).await {
        Err(KeyError::Other(e))
    } else {
        Ok(())
    }
}

pub struct KeyTreeNode {
    children: HashMap<Key, KeyTreeNode>,
    handler: Option<Box<KeyHandler<'static>>>,
}

impl KeyTreeNode {
    pub fn root() -> Self {
        Self {
            children: HashMap::new(),
            handler: None,
        }
    }

    pub fn insert(&mut self, keys: &[Key], handler: Box<KeyHandler<'static>>) {
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
