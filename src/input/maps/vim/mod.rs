mod insert;
mod mode_stack;
mod motions;
mod normal;
mod prompt;
mod tree;

use insert::vim_insert_mode;
use normal::vim_normal_mode;
use tree::KeyTreeNode;

use crate::{
    editing::motion::MotionRange,
    input::{Key, KeyError, Keymap, KeymapContext},
};

use self::mode_stack::VimModeStack;

use super::{KeyHandlerContext, KeyResult};

type KeyHandler = super::KeyHandler<VimKeymapState>;
type OperatorFn = dyn Fn(KeyHandlerContext<'_, VimKeymapState>, MotionRange) -> KeyResult;

// ======= modes ==========================================

pub struct VimMode {
    pub id: String,
    pub mappings: KeyTreeNode,
    pub default_handler: Option<Box<KeyHandler>>,
    pub after_handler: Option<Box<KeyHandler>>,
}

impl VimMode {
    pub fn new<Id: Into<String>>(id: Id, mappings: KeyTreeNode) -> Self {
        Self {
            id: id.into(),
            mappings,
            default_handler: None,
            after_handler: None,
        }
    }

    pub fn on_after(mut self, handler: Box<KeyHandler>) -> Self {
        self.after_handler = Some(handler);
        self
    }

    pub fn on_default(mut self, handler: Box<KeyHandler>) -> Self {
        self.default_handler = Some(handler);
        self
    }
}

impl std::fmt::Debug for VimMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[VimMode]")
    }
}

// ======= Keymap state ===================================

pub struct VimKeymapState {
    pub pending_linewise_operator_key: Option<Key>,
    pub operator_fn: Option<Box<OperatorFn>>,
    mode_stack: VimModeStack,
}

impl Default for VimKeymapState {
    fn default() -> Self {
        Self {
            pending_linewise_operator_key: None,
            operator_fn: None,
            mode_stack: VimModeStack::default(),
        }
    }
}

impl VimKeymapState {
    pub fn push_mode(&mut self, mode: VimMode) {
        self.mode_stack.push(mode);
    }

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
        let (mode, mode_from_stack) = if let Some(mode) = self.keymap.mode_stack.take_top() {
            (mode, true)
        } else if context.state().current_window().inserting {
            (vim_insert_mode(), false)
        } else {
            (vim_normal_mode(), false)
        };

        let mut current = &mode.mappings;
        let mut at_root = true;
        let mut result = Ok(());

        loop {
            if let Some(key) = context.next_key()? {
                // useful for testing:
                // context.state_mut().echo(format!("{:?}", key).into());

                if let Some(next) = current.children.get(&key) {
                    // TODO timeouts with nested handlers
                    if let Some(handler) = next.get_handler() {
                        result = handler(KeyHandlerContext {
                            context: Box::new(context),
                            keymap: &mut self.keymap,
                            key,
                        });
                        break;
                    } else {
                        // deeper into the tree
                        current = next;
                        at_root = false;
                    }
                } else if at_root {
                    // use the default mapping, if any
                    if let Some(handler) = &mode.default_handler {
                        result = handler(KeyHandlerContext {
                            context: Box::new(context),
                            keymap: &mut self.keymap,
                            key,
                        });
                        break;
                    }
                }
            } else {
                // no key read:
                break;
            }
        }

        if let Some(handler) = &mode.after_handler {
            handler(KeyHandlerContext {
                context: Box::new(context),
                keymap: &mut self.keymap,
                key: '\0'.into(),
            })?;
        }

        if mode_from_stack {
            // return the moved mode value back to the stack...
            self.keymap.mode_stack.return_top(mode);
        }

        result
    }
}

// ======= Tree-building macros ===========================

#[macro_export]
macro_rules! vim_branches {
    // base case:
    ($root:ident ->) => {
        use crate::input::maps::vim::VimKeymapState;
    };

    // normal keymap:
    (
        $root:ident ->
        $keys:literal =>
            |$ctx_name:ident| $body:expr,
        $($tail:tt)*
    ) => {
        $root.insert(&$keys.into_keys(), crate::key_handler!(VimKeymapState |$ctx_name| $body));
        crate::vim_branches! { $root -> $($tail)* }
    };

    // normal keymap with move:
    (
        $root:ident ->
        $keys:literal =>
            move |$ctx_name:ident| $body:expr,
        $($tail:tt)*
    ) => {
        $root.insert(&$keys.into_keys(), crate::key_handler!(VimKeymapState move |$ctx_name| $body));
        crate::vim_branches! { $root -> $($tail)* }
    };

    // immutable normal keymap (for completeness):
    (
        $root:ident ->
        $keys:literal =>
            |?mut $ctx_name:ident| $body:expr,
        $($tail:tt)*
    ) => {
        $root.insert(&$keys.into_keys(), crate::key_handler!(VimKeymapState |?mut $ctx_name| $body));
        crate::vim_branches! { $root -> $($tail)* }
    };

    // operators:
    (
        $root:ident ->
        $keys:literal =>
            operator |$ctx_name:ident, $motion_name:ident| $body:expr,
        $($tail:tt)*
    ) => {{
        $root.insert(&$keys.into_keys(), crate::key_handler!(VimKeymapState |$ctx_name| {
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
        crate::vim_branches! { $root -> $($tail)* }
    }};

    // motions:
    (
        $root:ident ->
        $keys:literal =>
            motion $factory:expr,
        $($tail:tt)*
    ) => {
        $root.insert(&$keys.into_keys(), crate::key_handler!(VimKeymapState |ctx| {
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
        crate::vim_branches! { $root -> $($tail)* }
    };
}

#[macro_export]
macro_rules! vim_tree {
    ( $( $SPEC:tt )* ) => {{
        use crate::input::maps::vim::KeyTreeNode;
        use crate::input::keys::KeysParsable;

        let mut root = KeyTreeNode::root();

        crate::vim_branches! { root -> $($SPEC)* }

        root
    }};
}

#[macro_export]
macro_rules! vim_motion {
    ($struct:expr) => {
        |ctx| {}
    };
}
