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
    app::widgets::Widget,
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
    keys_buffer: Vec<Key>,
}

impl Default for VimKeymapState {
    fn default() -> Self {
        Self {
            pending_linewise_operator_key: None,
            operator_fn: None,
            mode_stack: VimModeStack::default(),
            keys_buffer: Vec::default(),
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
        self.keys_buffer.clear();
    }
}

// ======= Keymap =========================================

pub struct VimKeymap {
    keymap: VimKeymapState,
}

impl VimKeymap {
    fn render_keys_buffer<'a, K: KeymapContext>(&'a mut self, context: &'a mut K) {
        context.state_mut().keymap_widget = Some(Widget::Spread(vec![
            Widget::Space,
            Widget::Space,
            Widget::Literal(render_keys_buffer(&self.keymap.keys_buffer).into()),
        ]));
    }
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
        let buffer_source = context.state().current_buffer().source().clone();
        let (mode, mode_from_stack, show_keys) =
            if let Some(mode) = self.keymap.mode_stack.take_top() {
                context.state_mut().keymap_widget = None;
                (mode, true, false)
            } else if context.state().current_window().inserting {
                context.state_mut().keymap_widget = Some(Widget::Literal("--INSERT--".into()));
                (vim_insert_mode(&buffer_source), false, false)
            } else {
                self.render_keys_buffer(context);
                (vim_normal_mode(), false, true)
            };

        if !show_keys && !self.keymap.keys_buffer.is_empty() {
            self.keymap.keys_buffer.clear();
        }

        let mut current = &mode.mappings;
        let mut at_root = true;
        let mut result = Ok(());

        loop {
            if let Some(key) = context.next_key()? {
                if show_keys {
                    self.keymap.keys_buffer.push(key.clone());
                }

                // if there's a change in progress, add the key to it
                context
                    .state_mut()
                    .current_buffer_mut()
                    .push_change_key(key);

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

                        if show_keys {
                            self.render_keys_buffer(context);
                        }
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
            if self.keymap.mode_stack.contains(&mode.id) {
                handler(KeyHandlerContext {
                    context: Box::new(context),
                    keymap: &mut self.keymap,
                    key: '\0'.into(),
                })?;
            }
        }

        if mode_from_stack {
            // return the moved mode value back to the stack...
            self.keymap.mode_stack.return_top(mode);
        }

        result
    }
}

fn render_keys_buffer(keys: &Vec<Key>) -> String {
    let mut s = String::default();
    for k in keys {
        k.write_str(&mut s);
    }
    return s;
}

// ======= Tree-building macros ===========================

#[macro_export]
macro_rules! vim_branches {
    // base case:
    ($root:ident ->) => {
        // NOTE: including these here is convenient, but
        // breaks completion; best to just copy these imports:
        // use crate::input::maps::vim::VimKeymapState;
        // use crate::input::KeymapContext;
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

    // immutable normal keymap with move:
    (
        $root:ident ->
        $keys:literal =>
            move |?mut $ctx_name:ident| $body:expr,
        $($tail:tt)*
    ) => {
        $root.insert(&$keys.into_keys(), crate::key_handler!(VimKeymapState move |?mut $ctx_name| $body));
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

            // operators always start a change
            $ctx_name.state_mut().current_bufwin().begin_keys_change($keys);

            if let Some(pending_key) = $ctx_name.keymap.pending_linewise_operator_key.take() {
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
                let operator_fn_result = $body;

                $ctx_name.state_mut().current_buffer_mut().end_change();

                operator_fn_result
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
