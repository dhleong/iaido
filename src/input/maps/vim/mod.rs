mod normal;

use std::collections::HashMap;

use normal::vim_normal_mode;

use crate::{
    editing::motion::MotionRange,
    input::{Key, KeyError, Keymap, KeymapContext},
};

use super::{KeyHandlerContext, KeyResult};

type KeyHandler<'a> = super::KeyHandler<'a, VimKeymapState>;
type OperatorFn = dyn Fn(KeyHandlerContext<'_, VimKeymapState>, MotionRange) -> KeyResult;

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
    pub pending_linewise_operator_key: Option<Key>,
    pub operator_fn: Option<Box<OperatorFn>>,
}

impl Default for VimKeymapState {
    fn default() -> Self {
        Self {
            pending_linewise_operator_key: None,
            operator_fn: None,
        }
    }
}

// ======= Keymap =========================================

pub struct VimKeymap {
    keymap: VimKeymapState,
}

impl Default for VimKeymap {
    fn default() -> Self {
        Self {
            keymap: Default::default(),
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
                            keymap: &mut self.keymap,
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
    // base case:
    ($root:ident ->) => {};

    // normal keymap:
    (
        $root:ident ->
        $keys:literal =>
            |$ctx_name:ident| $body:expr,
        $($tail:tt)*
    ) => {
        $root.insert(&$keys.into_keys(), key_handler!(VimKeymapState |$ctx_name| $body));
        vim_branches! { $root -> $($tail)* }
    };

    // operators:
    (
        $root:ident ->
        $keys:literal =>
            operator |$ctx_name:ident, $motion_name:ident| $body:expr,
        $($tail:tt)*
    ) => {{
        $root.insert(&$keys.into_keys(), key_handler!(VimKeymapState |$ctx_name| {
            use crate::editing::motion::Motion;

            if let Some(pending_key) = $ctx_name.keymap.pending_linewise_operator_key {
                $ctx_name.keymap.pending_linewise_operator_key = None;
                if pending_key == $keys.into() {
                    // execute linewise action directly:
                    let motion_impl = crate::editing::motion::linewise::FullLineMotion;
                    let $motion_name = motion_impl.range($ctx_name.state());
                    return $body;
                } else {
                    // different pending operator key; abort
                    return Ok(());
                }
            }

            // no pending linewise op; save a closure for motion use:
            $ctx_name.keymap.pending_linewise_operator_key = Some($keys.into());
            $ctx_name.keymap.operator_fn = Some(Box::new(|mut $ctx_name, $motion_name| {
                $body
            }));
            Ok(())
        }));
        vim_branches! { $root -> $($tail)* }
    }};

    // motions:
    (
        $root:ident ->
        $keys:literal =>
            motion $factory:expr,
        $($tail:tt)*
    ) => {
        $root.insert(&$keys.into_keys(), key_handler!(VimKeymapState |ctx| {
            use crate::editing::motion::Motion;
            let motion = $factory;
            if let Some(op) = ctx.keymap.operator_fn.take() {
                // execute pending operator fn
                let range = motion.range(ctx.state());
                let subcontext = crate::input::maps::KeyHandlerContext {
                    context: ctx.context,
                    keymap: ctx.keymap,
                };
                op(subcontext, range)
            } else {
                // no operator fn? just move the cursor
                motion.apply_cursor(ctx.state_mut());
                Ok(())
            }
        }));
        vim_branches! { $root -> $($tail)* }
    };
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

#[macro_export]
macro_rules! vim_motion {
    ($struct:expr) => {
        |ctx| {}
    };
}
