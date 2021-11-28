mod insert;
mod mode_stack;
mod motion;
mod motions;
mod normal;
mod prompt;
mod tree;

use std::any::Any;
use std::{collections::HashMap, ops, rc::Rc};

use insert::vim_insert_mode;
use normal::vim_normal_mode;
use tree::KeyTreeNode;

use crate::{
    app::widgets::Widget,
    editing::motion::MotionRange,
    editing::Id,
    input::{
        commands::CommandHandlerContext,
        completion::{state::BoxedCompleter, Completer},
        history::StringHistories,
        maps::vim::normal::search::VimSearchState,
        BoxableKeymap, Key, KeyError, Keymap, KeymapConfig, KeymapContext, RemapMode, Remappable,
    },
};

use self::mode_stack::VimModeStack;

use super::{KeyHandlerContext, KeyResult, UserKeyHandler};

type KeyHandler = super::KeyHandler<VimKeymap>;
type OperatorFn = dyn Fn(&mut KeyHandlerContext<'_, VimKeymap>, MotionRange) -> KeyResult;

// ======= modes ==========================================

pub struct VimMode {
    pub id: String,
    pub mappings: KeyTreeNode,
    pub default_handler: Option<Box<KeyHandler>>,
    pub after_handler: Option<Box<KeyHandler>>,
    pub completer: Option<Rc<dyn Completer>>,
}

impl VimMode {
    pub fn new<Id: Into<String>>(id: Id, mappings: KeyTreeNode) -> Self {
        Self {
            id: id.into(),
            mappings,
            default_handler: None,
            after_handler: None,
            completer: None,
        }
    }

    pub fn with_completer(mut self, completer: Option<Rc<dyn Completer>>) -> Self {
        self.completer = completer;
        self
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

impl ops::Add<Option<&KeyTreeNode>> for VimMode {
    type Output = VimMode;

    fn add(self, rhs: Option<&KeyTreeNode>) -> Self::Output {
        if let Some(rhs) = rhs {
            self + rhs
        } else {
            self
        }
    }
}

impl ops::Add<Option<KeyTreeNode>> for VimMode {
    type Output = VimMode;

    fn add(self, rhs: Option<KeyTreeNode>) -> Self::Output {
        self + rhs.as_ref()
    }
}

impl ops::Add<&KeyTreeNode> for VimMode {
    type Output = VimMode;

    fn add(self, rhs: &KeyTreeNode) -> Self::Output {
        let mut new = VimMode::new(self.id, &self.mappings + rhs);
        new.after_handler = self.after_handler;
        new.completer = self.completer;
        new.default_handler = self.default_handler;
        new
    }
}

impl std::fmt::Debug for VimMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[VimMode]")
    }
}

// ======= Keymap =========================================

#[derive(Default)]
pub struct VimKeymap {
    pub pending_linewise_operator_key: Option<Key>,
    pub operator_fn: Option<Box<OperatorFn>>,
    mode_stack: VimModeStack,
    keys_buffer: Vec<Key>,
    pub selected_register: Option<char>,
    active_completer: Option<Rc<dyn Completer>>,
    user_maps: HashMap<RemapMode, KeyTreeNode>,
    buffer_maps: HashMap<Id, HashMap<RemapMode, KeyTreeNode>>,
    pub histories: StringHistories,
    pub search: VimSearchState,
}

impl VimKeymap {
    pub fn completer(&self) -> Option<BoxedCompleter> {
        if let Some(completer) = self.active_completer.clone() {
            return Some(BoxedCompleter::from(completer));
        }
        None
    }

    pub fn push_mode(&mut self, mode: VimMode) {
        self.mode_stack.push(mode);
    }

    pub fn reset(&mut self) {
        self.pending_linewise_operator_key = None;
        self.operator_fn = None;
        self.selected_register = None;
        self.keys_buffer.clear();
    }

    fn render_keys_buffer<'a, K: KeymapContext>(&'a mut self, context: &'a mut K) {
        context.state_mut().keymap_widget = Some(Widget::Spread(vec![
            Widget::Space,
            Widget::Space,
            Widget::Literal(render_keys_buffer(&self.keys_buffer).into()),
        ]));
    }

    fn buffer_maps(
        &self,
        buf_id: Id,
        config: KeymapConfig,
        mode: &RemapMode,
    ) -> Option<KeyTreeNode> {
        let user_maps = self.user_maps.get(&mode);
        let buffer_maps = self
            .buffer_maps
            .get(&buf_id)
            .and_then(|maps| maps.get(&mode));

        if !config.allow_remap {
            return buffer_maps.and_then(|maps| Some(maps.clone()));
        }

        match (user_maps, buffer_maps) {
            (None, None) => None,
            (Some(user), None) => Some(user.clone()),
            (None, Some(buffer)) => Some(buffer.clone()),
            (Some(user), Some(buffer)) => Some(user + buffer),
        }
    }
}

impl Keymap for VimKeymap {
    fn process<'a, K: KeymapContext>(&'a mut self, context: &'a mut K) -> Result<(), KeyError> {
        let buf_id = context.state().current_buffer().id();
        let buffer_source = context.state().current_buffer().source().clone();
        let (mode, mode_from_stack, show_keys) = if let Some(mode) = self.mode_stack.take_top() {
            context.state_mut().keymap_widget = None;
            (mode, true, false)
        } else if context.state().current_window().inserting {
            context.state_mut().keymap_widget = Some(Widget::Literal("--INSERT--".into()));
            (
                vim_insert_mode(&buffer_source)
                    + self.buffer_maps(buf_id, context.config(), &RemapMode::VimInsert),
                false,
                false,
            )
        } else {
            self.render_keys_buffer(context);
            (
                vim_normal_mode()
                    + self.buffer_maps(buf_id, context.config(), &RemapMode::VimNormal),
                false,
                true,
            )
        };

        if !show_keys && !self.keys_buffer.is_empty() {
            self.keys_buffer.clear();
        }

        let mut current = &mode.mappings;
        let mut at_root = true;
        let mut result = Ok(());
        self.active_completer = mode.completer.clone();

        loop {
            if let Some(key) = context.next_key_with_map(Some(Box::new(self)))? {
                if show_keys {
                    self.keys_buffer.push(key.clone());
                }

                // if there's a change in progress, add the key to it
                if !context.state().current_buffer().is_read_only() {
                    context
                        .state_mut()
                        .current_buffer_mut()
                        .push_change_key(key);

                    if show_keys {
                        // NOTE: render here since some key handlers
                        // also read from keysource
                        self.render_keys_buffer(context);
                    }
                }

                if let Some(next) = current.children.get(&key) {
                    // TODO timeouts with nested handlers
                    if let Some(handler) = next.get_handler() {
                        result = handler(KeyHandlerContext {
                            context: Box::new(context),
                            keymap: self,
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
                            keymap: self,
                            key,
                        });
                        break;
                    }
                } else {
                    // no possible mapping; stop
                    self.keys_buffer.clear();
                    break;
                }
            } else {
                // no key read:
                break;
            }
        }

        if let Some(handler) = &mode.after_handler {
            if self.mode_stack.contains(&mode.id) {
                handler(KeyHandlerContext {
                    context: Box::new(context),
                    keymap: self,
                    key: '\0'.into(),
                })?;
            }
        }

        self.active_completer = None;

        if mode_from_stack {
            // return the moved mode value back to the stack...
            self.mode_stack.return_top(mode);
        }

        result
    }
}

impl BoxableKeymap for VimKeymap {
    fn remap_keys(&mut self, mode: RemapMode, from: Vec<Key>, to: Vec<Key>) {
        crate::input::remap_keys_to_fn(self, mode, from, to)
    }

    fn buf_remap_keys_user_fn(
        &mut self,
        buf_id: Id,
        mode: RemapMode,
        from: Vec<Key>,
        handler: Box<UserKeyHandler>,
    ) {
        self.buf_remap_keys_fn(
            buf_id,
            mode,
            from,
            Box::new(move |mut ctx| {
                handler(CommandHandlerContext {
                    context: Box::new(&mut ctx.context),
                    keymap: Box::new(ctx.keymap),
                    input: "".to_string(),
                })
            }),
        );
    }

    fn remap_keys_user_fn(
        &mut self,
        mode: RemapMode,
        from: Vec<Key>,
        handler: Box<UserKeyHandler>,
    ) {
        self.remap_keys_fn(
            mode,
            from,
            Box::new(move |mut ctx| {
                handler(CommandHandlerContext {
                    context: Box::new(&mut ctx.context),
                    keymap: Box::new(ctx.keymap),
                    input: "".to_string(),
                })
            }),
        );
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

fn insert_mapping(
    map: &mut HashMap<RemapMode, KeyTreeNode>,
    mode: RemapMode,
    keys: Vec<Key>,
    handler: Box<KeyHandler>,
) {
    let tree = if let Some(tree) = map.get_mut(&mode) {
        tree
    } else {
        let new_tree = KeyTreeNode::root();
        map.insert(mode.clone(), new_tree);
        map.get_mut(&mode).unwrap()
    };

    tree.insert(&keys, handler);
}

impl Remappable<VimKeymap> for VimKeymap {
    fn remap_keys_fn(&mut self, mode: RemapMode, keys: Vec<Key>, handler: Box<KeyHandler>) {
        insert_mapping(&mut self.user_maps, mode, keys, handler)
    }

    fn buf_remap_keys_fn(
        &mut self,
        id: Id,
        mode: RemapMode,
        keys: Vec<Key>,
        handler: Box<KeyHandler>,
    ) {
        let maps = self.buffer_maps.entry(id).or_insert(HashMap::default());
        insert_mapping(maps, mode, keys, handler)
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
        $root.insert(&$keys.into_keys(), crate::key_handler!(VimKeymap |$ctx_name| $body));
        crate::vim_branches! { $root -> $($tail)* }
    };

    // "change" keymaps check for read-only buffer and begin a change:
    (
        $root:ident ->
        $keys:literal =>
            change |$ctx_name:ident| $body:expr,
        $($tail:tt)*
    ) => {
        $root.insert(&$keys.into_keys(), crate::key_handler!(VimKeymap |$ctx_name| {
            if $ctx_name.state().current_buffer().is_read_only() {
                return Err(KeyError::ReadOnlyBuffer);
            }
            $ctx_name.state_mut().request_redraw();
            $ctx_name.state_mut().current_bufwin().begin_keys_change($keys);
            $body
        }));
        crate::vim_branches! { $root -> $($tail)* }
    };

    // normal keymap with move:
    (
        $root:ident ->
        $keys:literal =>
            move |$ctx_name:ident| $body:expr,
        $($tail:tt)*
    ) => {
        $root.insert(&$keys.into_keys(), crate::key_handler!(VimKeymap move |$ctx_name| $body));
        crate::vim_branches! { $root -> $($tail)* }
    };

    // immutable normal keymap with move:
    (
        $root:ident ->
        $keys:literal =>
            move |?mut $ctx_name:ident| $body:expr,
        $($tail:tt)*
    ) => {
        $root.insert(&$keys.into_keys(), crate::key_handler!(VimKeymap move |?mut $ctx_name| $body));
        crate::vim_branches! { $root -> $($tail)* }
    };

    // immutable normal keymap (for completeness):
    (
        $root:ident ->
        $keys:literal =>
            |?mut $ctx_name:ident| $body:expr,
        $($tail:tt)*
    ) => {
        $root.insert(&$keys.into_keys(), crate::key_handler!(VimKeymap |?mut $ctx_name| $body));
        crate::vim_branches! { $root -> $($tail)* }
    };

    // operators:
    (
        $root:ident ->
        $keys:literal =>
            operator |$ctx_name:ident, $motion_name:ident| $body:expr,
        $($tail:tt)*
    ) => {{
        $root.insert(&$keys.into_keys(), crate::key_handler!(VimKeymap |$ctx_name| {
            use crate::editing::motion::Motion;

            if $ctx_name.state().current_buffer().is_read_only() {
                return Err(KeyError::ReadOnlyBuffer);
            }

            // operators always start a change
            $ctx_name.state_mut().request_redraw();
            $ctx_name.state_mut().current_bufwin().begin_keys_change($keys);

            if let Some(pending_key) = $ctx_name.keymap.pending_linewise_operator_key.take() {
                let operator_result = if pending_key == $keys.into() {
                    // execute linewise action directly:
                    let motion_impl = crate::editing::motion::linewise::FullLineMotion;
                    let $motion_name = motion_impl.range($ctx_name.state());
                    $body
                } else {
                    // different pending operator key; abort
                    Ok(())
                };
                $ctx_name.state_mut().current_buffer_mut().end_change();
                return operator_result;
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
            motion |$ctx_name:ident| $factory:expr,
        $($tail:tt)*
    ) => {
        $root.insert(&$keys.into_keys(), crate::key_handler!(VimKeymap |$ctx_name| {
            let motion = $factory;
            crate::input::maps::vim::motion::apply_motion($ctx_name, motion)
        }));
        crate::vim_branches! { $root -> $($tail)* }
    };

    (
        $root:ident ->
        $keys:literal =>
            motion |?mut $ctx_name:ident| $factory:expr,
        $($tail:tt)*
    ) => {
        $root.insert(&$keys.into_keys(), crate::key_handler!(VimKeymap |?mut $ctx_name| {
            let motion = $factory;
            crate::input::maps::vim::motion::apply_motion($ctx_name, motion)
        }));
        crate::vim_branches! { $root -> $($tail)* }
    };

    (
        $root:ident ->
        $keys:literal =>
            motion $factory:expr,
        $($tail:tt)*
    ) => {
        crate::vim_branches! { $root ->
            $keys => motion |?mut ctx| $factory,
            $($tail)*
        };
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
