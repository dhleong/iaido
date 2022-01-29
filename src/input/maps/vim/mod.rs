mod cmdline;
mod insert;
mod mode_stack;
mod motion;
mod motions;
mod normal;
mod object;
mod op;
mod prompt;
mod tree;
mod util;

use std::any::Any;
use std::{collections::HashMap, ops, rc::Rc};

use insert::vim_insert_mode;
use normal::vim_normal_mode;
use tree::KeyTreeNode;

use crate::input::source::memory::MemoryKeySource;
use crate::input::KeymapContextWithKeys;
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

#[derive(Clone)]
pub struct VimMode {
    pub id: String,
    pub mappings: KeyTreeNode,
    pub shows_keys: bool,
    pub keymap_widget: Option<Widget>,
    pub allows_linewise: bool,
    pub default_handler: Option<Rc<KeyHandler>>,
    pub after_handler: Option<Rc<KeyHandler>>,
    pub exit_handler: Option<Rc<KeyHandler>>,
    pub completer: Option<Rc<dyn Completer>>,
}

impl VimMode {
    pub fn new<Id: Into<String>>(id: Id, mappings: KeyTreeNode) -> Self {
        Self {
            id: id.into(),
            mappings,
            allows_linewise: true,
            shows_keys: false,
            keymap_widget: None,
            default_handler: None,
            after_handler: None,
            exit_handler: None,
            completer: None,
        }
    }

    pub fn with_completer(mut self, completer: Option<Rc<dyn Completer>>) -> Self {
        self.completer = completer;
        self
    }

    pub fn with_allows_linewise(mut self, allows_linewise: bool) -> Self {
        self.allows_linewise = allows_linewise;
        self
    }

    pub fn with_shows_keys(mut self, shows_keys: bool) -> Self {
        self.shows_keys = shows_keys;
        self
    }

    pub fn on_after(mut self, handler: Box<KeyHandler>) -> Self {
        self.after_handler = Some(Rc::new(handler));
        self
    }

    pub fn on_exit(mut self, handler: Box<KeyHandler>) -> Self {
        self.exit_handler = Some(Rc::new(handler));
        self
    }

    pub fn on_default(mut self, handler: Box<KeyHandler>) -> Self {
        self.default_handler = Some(Rc::new(handler));
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
        let mut new = self.clone();
        new.mappings = &self.mappings + rhs;
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
    pub operator_fn: Option<Box<OperatorFn>>,
    mode_stack: VimModeStack,
    keys_buffer: Vec<Key>,
    pub selected_register: Option<char>,
    active_completer: Option<Rc<dyn Completer>>,
    user_maps: HashMap<RemapMode, KeyTreeNode>,
    buffer_maps: HashMap<Id, HashMap<RemapMode, KeyTreeNode>>,
    pub histories: StringHistories,
    pub search: VimSearchState,
    count: u32,
    count_multiplier: Option<u32>,
}

impl VimKeymap {
    pub fn allows_linewise(&self) -> bool {
        if let Some(top) = self.mode_stack.peek() {
            top.allows_linewise
        } else {
            // Assume true, I guess?
            true
        }
    }

    pub fn completer(&self) -> Option<BoxedCompleter> {
        if let Some(completer) = self.active_completer.clone() {
            return Some(BoxedCompleter::from(completer));
        }
        None
    }

    pub fn push_mode(&mut self, mode: VimMode) {
        self.mode_stack.push(mode);
    }

    pub fn pop_mode(&mut self, mode_id: &str) {
        self.mode_stack.pop_if(mode_id);
    }

    pub fn reset(&mut self) {
        self.operator_fn = None;
        self.selected_register = None;
        self.keys_buffer.clear();
        self.count = 0;
        self.count_multiplier = None;
    }

    pub fn take_count(&mut self) -> u32 {
        let count = self.count;
        self.count = 0;
        let given_count = match count {
            0 => 1,
            _ => count,
        };

        // NOTE: Per `:help motion`, if a count is given on both the operator and the motion, they
        // are multiplied together
        if let Some(multiplier) = self.count_multiplier.take() {
            multiplier * given_count
        } else {
            given_count
        }
    }

    fn render_keys_buffer<'a, K: KeymapContext>(&'a mut self, context: &'a mut K) {
        let keys = Widget::Literal(render_keys_buffer(&self.keys_buffer).into());
        context.state_mut().keymap_widget = Some(match &context.state().keymap_widget {
            // Reuse manually-provided widgets:
            Some(Widget::Spread(items)) if items.len() == 3 => {
                Widget::Spread(vec![items[0].clone(), items[1].clone(), keys])
            }

            // Overwrite:
            _ => Widget::Spread(vec![Widget::Space, Widget::Space, keys]),
        });
    }

    fn buffer_maps(
        &self,
        buf_id: Id,
        config: KeymapConfig,
        mode: &RemapMode,
    ) -> Option<KeyTreeNode> {
        if !config.allow_remap {
            return None;
        }

        let user_maps = self.user_maps.get(&mode);
        let buffer_maps = self
            .buffer_maps
            .get(&buf_id)
            .and_then(|maps| maps.get(&mode));

        match (user_maps, buffer_maps) {
            (None, None) => None,
            (Some(user), None) => Some(user.clone()),
            (None, Some(buffer)) => Some(buffer.clone()),
            (Some(user), Some(buffer)) => Some(user + buffer),
        }
    }

    fn push_count_digit(&mut self, digit: u32) {
        self.count = self.count * 10 + digit;
    }

    fn has_pending_state(&self) -> bool {
        self.operator_fn.is_some() || self.selected_register.is_some() || self.count > 0
    }
}

fn pick_completer<K: KeymapContext>(mode: &VimMode, context: &mut K) -> Option<Rc<dyn Completer>> {
    let buf_id = context.state().current_buffer().id();
    if let Some(completer) = mode.completer.clone() {
        Some(completer)
    } else if let Some(completer) = context
        .state_mut()
        .connections
        .with_buffer_engine(buf_id, |eng| eng.completer.clone())
    {
        Some(Rc::new(completer))
    } else {
        None
    }
}

impl Keymap for VimKeymap {
    fn process<'a, K: KeymapContext>(&'a mut self, context: &'a mut K) -> Result<(), KeyError> {
        let buf_id = context.state().current_buffer().id();
        let buffer_source = context.state().current_buffer().source().clone();
        let mode = if let Some(mode) = self.mode_stack.peek() {
            if !mode.shows_keys {
                context.state_mut().keymap_widget = None;
            }
            if let Some(widget) = &mode.keymap_widget {
                context.state_mut().keymap_widget = Some(widget.clone());
            }
            mode.clone()
        } else if context.state().current_window().inserting {
            context.state_mut().keymap_widget = Some(Widget::Literal("--INSERT--".into()));
            vim_insert_mode(&buffer_source)
                + self.buffer_maps(buf_id, context.config(), &RemapMode::VimInsert)
        } else {
            context.state_mut().keymap_widget = None;
            self.render_keys_buffer(context);
            vim_normal_mode() + self.buffer_maps(buf_id, context.config(), &RemapMode::VimNormal)
        };

        if !mode.shows_keys && !self.keys_buffer.is_empty() {
            self.keys_buffer.clear();
        }

        let mut current = &mode.mappings;
        let mut at_root = true;
        let mut result = Ok(());
        let mut last_key: Key = '\0'.into();
        self.active_completer = pick_completer(&mode, context);

        loop {
            if let Some(key) = context.next_key_with_map(Some(Box::new(self)))? {
                if mode.shows_keys {
                    self.keys_buffer.push(key.clone());
                }

                last_key = key;

                // Clear the popup menu, if any
                context.state_mut().pum = None;

                // if there's a change in progress, add the key to it
                if !context.state().current_buffer().is_read_only() {
                    context
                        .state_mut()
                        .current_buffer_mut()
                        .push_change_key(key);

                    if mode.shows_keys {
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

                        if mode.shows_keys && !self.has_pending_state() {
                            self.keys_buffer.clear();
                            self.render_keys_buffer(context);
                        }
                        break;
                    } else {
                        // deeper into the tree
                        current = next;
                        at_root = false;

                        if mode.shows_keys {
                            self.render_keys_buffer(context);
                        }
                    }
                } else if at_root {
                    if let Some(handler) = &mode.default_handler {
                        // use the default mapping, if any
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

        if self.has_pending_state() && mode.shows_keys {
            self.render_keys_buffer(context);
        }

        if let Some(handler) = &mode.after_handler {
            if self.mode_stack.contains(&mode.id) {
                handler(KeyHandlerContext {
                    context: Box::new(context),
                    keymap: self,
                    key: last_key,
                })?;
            }
        }

        self.active_completer = None;

        // Call the mode's "Exit" handler if it's no longer on the stack
        if !self.mode_stack.contains(&mode.id) {
            self.mode_stack.pop_if(&mode.id);

            if let Some(on_exit) = mode.exit_handler {
                on_exit(KeyHandlerContext {
                    context: Box::new(context),
                    keymap: self,
                    key: last_key,
                })?;
            }
        }

        if mode.shows_keys {
            self.render_keys_buffer(context);
        }

        result
    }
}

impl BoxableKeymap for VimKeymap {
    fn enter_user_mode(&mut self, mode_name: String) -> bool {
        let remap_mode = RemapMode::User(mode_name.clone());
        if let Some(mappings) = self.user_maps.get(&remap_mode) {
            let mode_id = mode_name.clone();
            let mut mode = VimMode::new(
                mode_name.clone(),
                crate::vim_tree! {
                    "<esc>" => move |?mut ctx| {
                        ctx.keymap.pop_mode(&mode_id);
                        Ok(())
                    },
                } + mappings.clone(),
            );
            mode.shows_keys = true;
            mode.keymap_widget = Some(Widget::Spread(vec![
                Widget::Literal(format!("--{}--", mode_name).into()),
                Widget::Space,
                Widget::Space,
            ]));
            self.push_mode(mode);
            return true;
        }

        return false;
    }

    fn process_keys(&mut self, context: &mut KeymapContextWithKeys<MemoryKeySource>) -> KeyResult {
        self.process(context)
    }

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
macro_rules! operator_handler {
    (
        $keys:literal => $ctx_name:ident, $motion_name:ident, $body:expr,
        $after_body:expr
    ) => {{
        // Save a closure for motion use
        let count = $ctx_name.keymap.take_count();
        $ctx_name.keymap.count_multiplier = Some(count);
        $ctx_name.keymap.operator_fn = Some(Box::new(|mut $ctx_name, $motion_name| {
            let operator_fn_result = $body;

            $after_body

            operator_fn_result
        }));

        // Enter Operator-Pending mode
        let allows_linewise = $ctx_name.keymap.allows_linewise();
        let op_mode =
            crate::input::maps::vim::op::vim_operator_pending_mode(allows_linewise, $keys.into());
        $ctx_name.keymap.push_mode(op_mode);
        Ok(())
    }};
}

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
            crate::input::maps::vim::util::verify_can_edit(&$ctx_name)?;
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
            crate::input::maps::vim::util::verify_can_edit(&$ctx_name)?;

            // Operators always start a change
            $ctx_name.state_mut().request_redraw();
            $ctx_name.state_mut().current_bufwin().begin_keys_change($keys);

            crate::operator_handler!($keys => $ctx_name, $motion_name, $body, {
                $ctx_name.state_mut().current_buffer_mut().end_change();
            })
        }));
        crate::vim_branches! { $root -> $($tail)* }
    }};

    (
        $root:ident ->
        $keys:literal =>
            operator ?change |$ctx_name:ident, $motion_name:ident| $body:expr,
        $($tail:tt)*
    ) => {{
        $root.insert(&$keys.into_keys(), crate::key_handler!(VimKeymap |$ctx_name| {
            crate::operator_handler!($keys => $ctx_name, $motion_name, $body, {})
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{editing::motion::tests::window, input::keys::KeysParsable};
    use indoc::indoc;

    #[test]
    fn enter_user_mode() {
        let ctx = window(indoc! {"
            Take my |love
        "});
        let mut vim = VimKeymap::default();
        vim.remap_keys_fn(
            RemapMode::User("user".to_string()),
            "gh".into_keys(),
            Box::new(|mut ctx| {
                ctx.state_mut().current_window_mut().cursor = (0, 0).into();
                Ok(())
            }),
        );

        vim.enter_user_mode("user".to_string());

        ctx.feed_keys(vim, "gh").assert_visual_match(indoc! {"
            |Take my love
        "});
    }

    #[test]
    fn remap_in_user_mode_shouldnt_panic() {
        let ctx = window(indoc! {"
            Take my |love
        "});
        let mut vim = VimKeymap::default();
        vim.remap_keys_fn(
            RemapMode::User("user".to_string()),
            "gh".into_keys(),
            Box::new(|mut ctx| {
                ctx.state_mut().current_window_mut().cursor = (0, 0).into();
                Ok(())
            }),
        );

        vim.remap_keys(
            RemapMode::User("user".to_string()),
            "g0".into_keys(),
            "gh".into_keys(),
        );

        vim.enter_user_mode("user".to_string());

        ctx.feed_keys(vim, "g0").assert_visual_match(indoc! {"
            |Take my love
        "});
    }
}
