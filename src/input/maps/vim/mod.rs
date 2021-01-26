mod insert;
mod motions;
mod normal;
mod tree;

use insert::vim_insert_mode;
use normal::vim_normal_mode;
use tree::KeyTreeNode;

use crate::{
    editing::motion::MotionRange,
    input::{Key, KeyError, Keymap, KeymapContext},
};

use super::{KeyHandlerContext, KeyResult};

type KeyHandler<'a> = super::KeyHandler<'a, VimKeymapState>;
type OperatorFn = dyn Fn(KeyHandlerContext<'_, VimKeymapState>, MotionRange) -> KeyResult;

// ======= modes ==========================================

pub struct VimMode<'a> {
    pub mappings: KeyTreeNode<'a>,
    pub default_handler: Option<Box<KeyHandler<'a>>>,
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

impl VimKeymapState {
    fn reset(&mut self) {
        self.pending_linewise_operator_key = None;
        self.operator_fn = None;
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
        let mode = if context.state().current_window().inserting {
            vim_insert_mode()
        } else {
            vim_normal_mode()
        };
        let mut current = &mode.mappings;
        let mut at_root = true;

        loop {
            if let Some(key) = context.next_key()? {
                if let Some(next) = current.children.get(&key) {
                    // TODO timeouts with nested handlers
                    if let Some(handler) = next.get_handler() {
                        return handler(KeyHandlerContext {
                            context: Box::new(context),
                            keymap: &mut self.keymap,
                            key,
                        });
                    } else {
                        // deeper into the tree
                        current = next;
                        at_root = false;
                    }
                } else if at_root {
                    // use the default mapping, if any
                    if let Some(handler) = mode.default_handler {
                        return handler(KeyHandlerContext {
                            context: Box::new(context),
                            keymap: &mut self.keymap,
                            key,
                        });
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
            let operator_fn = ctx.keymap.operator_fn.take();
            ctx.keymap.reset(); // always clear

            if let Some(op) = operator_fn {
                // execute pending operator fn
                let range = motion.range(ctx.state());
                let subcontext = crate::input::maps::KeyHandlerContext {
                    context: ctx.context,
                    keymap: ctx.keymap,
                    key: ctx.key,
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
        use crate::input::maps::vim::KeyTreeNode;
        use crate::input::keys::KeysParsable;

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
