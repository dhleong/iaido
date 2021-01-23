mod normal;

use std::collections::HashMap;

use normal::vim_normal_mode;

use crate::input::{Key, KeyError, Keymap, KeymapContext};

use super::KeyHandlerContext;

type KeyHandler<'a> = super::KeyHandler<'a, VimKeymapState>;

pub struct KeyTreeNode<'a> {
    children: HashMap<Key, KeyTreeNode<'a>>,
    handler: Option<Box<KeyHandler<'a>>>,
}

impl<'a> KeyTreeNode<'a> {
    pub fn root() -> Self {
        Self {
            children: HashMap::new(),
            handler: None,
        }
    }

    pub fn insert(&mut self, keys: &[Key], handler: Box<KeyHandler<'a>>) {
        if keys.is_empty() {
            self.handler = Some(handler);
        } else {
            let first_key = keys[0];
            let node = self
                .children
                .entry(first_key)
                .or_insert(KeyTreeNode::root());
            node.insert(&keys[1..], handler);
        }
    }
}

// ======= Keymap state ===================================

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

// ======= Keymap =========================================

pub struct VimKeymap {
    state: VimKeymapState,
}

impl Default for VimKeymap {
    fn default() -> Self {
        Self {
            state: Default::default(),
        }
    }
}

impl Keymap for VimKeymap {
    fn process<'a, K: KeymapContext>(&'a mut self, context: &'a mut K) -> Result<(), KeyError> {
        let tree = vim_normal_mode();
        let mut current = &tree;

        loop {
            if let Some(key) = context.next_key()? {
                if let Some(next) = current.children.get(&key) {
                    // TODO timeouts with nested handlers
                    if let Some(handler) = &next.handler {
                        return handler(KeyHandlerContext {
                            context: Box::new(context),
                            state: &mut self.state,
                        });
                    } else {
                        // deeper into the tree
                        current = next;
                    }
                }
            } else {
                // no key read:
                return Ok(());
            }
        }
    }
}

// ======= Tree-building macros ===========================

#[macro_export]
macro_rules! vim_branches {
    ($root:ident -> $keys:literal => |$ctx_name:ident| $body:expr) => {
        $root.insert(&$keys.into_keys(), key_handler!(VimKeymapState |$ctx_name| $body));
    };

    ($root:ident -> $keys:literal => |$ctx_name:ident| $body:expr, $($keysn:literal => |$ctx_namen:ident| $bodyn:expr),+) => {{
        vim_branches! { $root -> $keys => |$ctx_name| $body }
        vim_branches! { $root -> $($keysn => |$ctx_namen| $bodyn),+ }
    }};
}

#[macro_export]
macro_rules! vim_tree {
    ( $( $SPEC:tt )* ) => {{
        use crate::key_handler;
        use crate::vim_branches;

        let mut root = KeyTreeNode::root();

        vim_branches! { root -> $($SPEC)* }

        root
    }};
}
