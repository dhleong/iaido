pub mod char;
pub mod end;
pub mod find;
pub mod linewise;
pub mod repeated;
pub mod search;
mod util;
pub mod word;

use bitflags::bitflags;

use crate::app::bufwin::BufWin;

use super::{text::EditableLine, window::Window, Buffer, CursorPosition};

bitflags! {
    pub struct MotionFlags: u8 {
        const NONE = 0;
        const LINEWISE  = 0b01;
        const EXCLUSIVE = 0b10;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionRange(pub CursorPosition, pub CursorPosition, pub MotionFlags);
impl MotionRange {
    pub fn is_empty(&self) -> bool {
        self.0 == self.1
    }

    pub fn lines(&self) -> (usize, usize) {
        let &MotionRange(
            CursorPosition {
                line: first_line, ..
            },
            CursorPosition {
                line: last_line, ..
            },
            ..,
        ) = self;
        (first_line, last_line)
    }
}

pub type SimpleMotionRange = (CursorPosition, CursorPosition);

impl From<SimpleMotionRange> for MotionRange {
    fn from(simple: SimpleMotionRange) -> Self {
        let (start, end) = simple;
        MotionRange(start, end, MotionFlags::NONE)
    }
}

impl From<((usize, usize), (usize, usize))> for MotionRange {
    fn from(simple: ((usize, usize), (usize, usize))) -> Self {
        let (start, end) = simple;
        MotionRange(start.into(), end.into(), MotionFlags::NONE)
    }
}

pub trait MotionContext {
    fn buffer(&self) -> &Box<dyn Buffer>;
    fn buffer_mut(&mut self) -> &mut Box<dyn Buffer>;
    fn cursor(&self) -> CursorPosition;
    fn window(&self) -> &Box<Window>;
    fn window_mut(&mut self) -> &mut Box<Window>;

    fn bufwin(&mut self) -> BufWin;

    fn with_cursor(&self, cursor: CursorPosition) -> PositionedMotionContext<Self> {
        PositionedMotionContext { base: self, cursor }
    }
}

pub struct PositionedMotionContext<'a, T: MotionContext + ?Sized> {
    base: &'a T,
    cursor: CursorPosition,
}

impl<'a, T: MotionContext> MotionContext for PositionedMotionContext<'a, T> {
    fn buffer(&self) -> &Box<dyn Buffer> {
        self.base.buffer()
    }
    fn buffer_mut(&mut self) -> &mut Box<dyn Buffer> {
        panic!("PositionedMotionContext should not be used mutatively")
    }
    fn bufwin(&mut self) -> BufWin {
        panic!("PositionedMotionContext should not be used mutatively")
    }
    fn cursor(&self) -> CursorPosition {
        self.cursor
    }
    fn window(&self) -> &Box<Window> {
        self.base.window()
    }
    fn window_mut(&mut self) -> &mut Box<Window> {
        panic!("PositionedMotionContext should not be used mutatively")
    }
}

pub trait Motion {
    fn destination<T: MotionContext>(&self, context: &T) -> CursorPosition;

    fn flags(&self) -> MotionFlags {
        MotionFlags::NONE
    }

    fn range<T: MotionContext>(&self, context: &T) -> MotionRange {
        let flags = self.flags();
        let linewise = flags.contains(MotionFlags::LINEWISE);
        let inclusive = !flags.contains(MotionFlags::EXCLUSIVE);

        let start = context.cursor();
        let mut end = self.destination(context);
        if inclusive {
            end.col += 1;
        }

        normalize_range(
            context.buffer(),
            if linewise && end < start {
                MotionRange(
                    end.start_of_line(),
                    start.end_of_line(context.buffer()),
                    flags,
                )
            } else if linewise {
                MotionRange(
                    start.start_of_line(),
                    end.end_of_line(context.buffer()),
                    flags,
                )
            } else if end < start {
                MotionRange(end, start, flags)
            } else {
                MotionRange(start, end, flags)
            },
        )
    }

    fn apply_cursor<T: MotionContext>(&self, context: &mut T) {
        let new_cursor = self.destination(context);
        let new_cursor = context.window().clamp_cursor(context.buffer(), new_cursor);
        context.window_mut().cursor = new_cursor;
    }

    fn delete_range<T: MotionContext>(&self, context: &mut T) {
        let range = self.range(context);
        context.buffer_mut().delete_range(range);
        context.window_mut().cursor = context.window().clamp_cursor(context.buffer(), range.0);
    }
}

pub trait DirectionalMotion {
    fn is_forward(&self) -> bool;
}

fn normalize_range(buffer: &Box<dyn Buffer>, range: MotionRange) -> MotionRange {
    let MotionRange(start, end, flags) = range;
    if flags.contains(MotionFlags::EXCLUSIVE) && end.line > start.line && end.col == 0 {
        // see :help exclusive
        let mut motion = MotionRange(
            start,
            start.end_of_line(buffer),
            flags - MotionFlags::EXCLUSIVE,
        );

        // if "before the first non-blank in a line" the motion
        // becomes linewise
        // see :help exclusive-linewise
        let first_non_blank_before = buffer
            .get(start.line)
            .position(0..(start.col + 1), |c| !char::is_whitespace(c));

        if first_non_blank_before.is_none() {
            motion.0 = motion.0.start_of_line();
            motion.2 |= MotionFlags::LINEWISE;
        }

        return motion;
    }

    // default to the original range
    return range;
}

#[cfg(test)]
pub mod tests {
    use std::{cmp::max, time::Duration};

    use crate::{
        app, delegate_keysource,
        editing::{
            buffer::{MemoryBuffer, UndoableBuffer},
            ids::FIRST_USER_BUFFER_ID,
            text::TextLine,
            window::Window,
            Buffer, HasId, Resizable, Size,
        },
        input::{
            commands::{
                registry::{CommandRegistry, CommandSpec},
                CommandHandlerContext,
            },
            completion::{tests::StaticCompleter, CompletableContext},
            source::memory::MemoryKeySource,
            BoxableKeymap, KeySource, Keymap, KeymapContext,
        },
        tui::{
            rendering::display::tests::TestableDisplay, Display, LayoutContext, RenderContext,
            Renderable,
        },
    };

    use super::*;
    use std::any::Any;

    pub struct TestKeymapContext {
        keys: MemoryKeySource,
        state: app::State,
    }

    pub struct TestBoxableKeymap;

    impl BoxableKeymap for TestBoxableKeymap {
        fn enter_user_mode(&mut self, _mode: String) -> bool {
            todo!()
        }

        fn remap_keys(
            &mut self,
            _mode: crate::input::RemapMode,
            _from: Vec<crate::input::Key>,
            _to: Vec<crate::input::Key>,
        ) {
            todo!()
        }

        fn buf_remap_keys_user_fn(
            &mut self,
            _id: crate::editing::Id,
            _mode: crate::input::RemapMode,
            _keys: Vec<crate::input::Key>,
            _handler: Box<crate::input::maps::UserKeyHandler>,
        ) {
            todo!()
        }

        fn remap_keys_user_fn(
            &mut self,
            _mode: crate::input::RemapMode,
            _keys: Vec<crate::input::Key>,
            _handler: Box<crate::input::maps::UserKeyHandler>,
        ) {
            todo!()
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn process_keys(
            &mut self,
            _context: &mut crate::input::KeymapContextWithKeys<MemoryKeySource>,
        ) -> crate::input::maps::KeyResult {
            todo!()
        }
    }

    pub struct TestKeyHandlerContext {
        pub context: TestKeymapContext,
        pub keymap: TestBoxableKeymap,
    }

    impl TestKeyHandlerContext {
        pub fn empty() -> Self {
            Self::from_keys("")
        }
        pub fn from_keys(keys: &'static str) -> Self {
            TestKeyHandlerContext {
                context: TestKeymapContext {
                    state: app::State::default(),
                    keys: MemoryKeySource::from_keys(keys),
                },
                keymap: TestBoxableKeymap,
            }
        }

        pub fn command_context(&mut self, input: &'static str) -> CommandHandlerContext {
            CommandHandlerContext::new(&mut self.context, &mut self.keymap, input.to_string())
        }
    }

    impl KeymapContext for TestKeymapContext {
        fn state(&self) -> &app::State {
            &self.state
        }

        fn state_mut(&mut self) -> &mut app::State {
            &mut self.state
        }
    }

    impl KeySource for TestKeymapContext {
        delegate_keysource!(keys);
    }

    pub struct TestWindow {
        pub window: Box<Window>,
        pub buffer: Box<dyn Buffer>,
        commands: CommandRegistry,
    }

    impl TestWindow {
        pub fn motion<T: Motion>(&mut self, motion: T) {
            motion.apply_cursor(self);
        }

        pub fn select<T: Motion>(&self, motion: T) -> String {
            let range = motion.range(self);
            self.buffer().get_range(range).get_contents()
        }

        pub fn set_inserting(&mut self, inserting: bool) {
            self.window.set_inserting(inserting);
        }

        pub fn mock_command_completions(
            &mut self,
            command_name: &'static str,
            completions: Vec<&'static str>,
        ) {
            let completion_strings = completions.iter().map(|s| s.to_string()).collect();
            let mut spec = CommandSpec::handler(Box::new(|_ctx| Ok(())));
            spec.push_arg_completer(Box::new(StaticCompleter::new(completion_strings)));
            self.commands.insert(command_name.to_string(), spec);
        }

        pub fn feed_keys_with_state<K: Keymap>(
            mut self,
            mut keymap: K,
            mut state: app::State,
            keys: &str,
        ) -> (Self, K, app::State) {
            let key_source = MemoryKeySource::from_keys(keys);

            self.buffer = state.buffers.replace(self.buffer);

            let window = state.current_window_mut();
            window.cursor = self.window.cursor;
            window.resize(self.window.size);

            let mut context = TestKeymapContext {
                keys: key_source,
                state,
            };

            while let Ok(has_next) = context.poll_key(Duration::from_millis(0)) {
                if !has_next {
                    break;
                }
                if let Err(e) = keymap.process(&mut context) {
                    panic!("Error processing {}: {:?}", keys.to_string(), e);
                }
            }

            // take back our buffer; copy over cursor
            self.buffer = context.state.buffers.replace(self.buffer);
            self.window.cursor = context.state.current_window_mut().cursor;

            (self, keymap, context.state)
        }

        pub fn feed_keys_for_state<K: Keymap>(self, keymap: K, keys: &str) -> (Self, app::State) {
            let (ctx, _, state) = self.feed_keys_with_state(keymap, app::State::default(), keys);
            (ctx, state)
        }

        pub fn feed_keys<K: Keymap>(self, keymap: K, keys: &str) -> Self {
            let (result, _) = self.feed_keys_for_state(keymap, keys);
            result
        }

        pub fn feed_vim(self, keys: &str) -> Self {
            self.feed_keys(crate::input::maps::vim::VimKeymap::default(), keys)
        }

        pub fn feed_vim_for_keymap(
            self,
            keys: &str,
        ) -> (Self, crate::input::maps::vim::VimKeymap, app::State) {
            let (ctx, keymap, state) = self.feed_keys_with_state(
                crate::input::maps::vim::VimKeymap::default(),
                app::State::default(),
                keys,
            );
            (ctx, keymap, state)
        }

        pub fn feed_vim_with_state(self, state: app::State, keys: &str) -> (Self, app::State) {
            let (ctx, _, state) = self.feed_keys_with_state(
                crate::input::maps::vim::VimKeymap::default(),
                state,
                keys,
            );
            (ctx, state)
        }

        pub fn feed_vim_for_state(self, keys: &str) -> (Self, app::State) {
            self.feed_keys_for_state(crate::input::maps::vim::VimKeymap::default(), keys)
        }

        /// Assert that this Window visually matches the window that would
        /// be created if the given string were passed to [`window`].
        pub fn assert_visual_match(&mut self, s: &'static str) {
            let mut expected_context = window(s);
            expected_context.set_inserting(self.window.inserting);
            let expected = expected_context.render_at_own_size();
            let actual = self.render_at_own_size();
            assert_eq!(
                (actual.to_visual_string(), self.window.cursor),
                (expected.to_visual_string(), expected_context.window.cursor)
            );
        }

        pub fn render(&mut self, display: &mut Display) {
            self.window.resize(display.size);

            self.window
                .layout(&LayoutContext::with_buffer(&self.buffer));

            let state = crate::app::State::default();
            let mut context = RenderContext::new(&state, display).with_buffer(&self.buffer);
            self.window.render(&mut context);
        }

        pub fn render_at_own_size(&mut self) -> Display {
            let mut display = Display::new(self.window.size);
            self.render(&mut display);
            display
        }

        pub fn render_into_size(&mut self, width: u16, height: u16) -> Display {
            let mut display = Display::new(Size {
                w: width,
                h: height,
            });
            self.render(&mut display);
            display
        }

        pub fn scroll_lines(&mut self, virtual_lines: i32) {
            <TestWindow as MotionContext>::bufwin(self).scroll_lines(virtual_lines);
        }
    }

    impl CompletableContext for TestWindow {
        fn bufwin(&mut self) -> BufWin {
            <TestWindow as MotionContext>::bufwin(self)
        }

        fn commands(&self) -> &CommandRegistry {
            &self.commands
        }
    }

    impl MotionContext for TestWindow {
        fn buffer(&self) -> &Box<dyn Buffer> {
            &self.buffer
        }

        fn buffer_mut(&mut self) -> &mut Box<dyn Buffer> {
            &mut self.buffer
        }

        fn bufwin(&mut self) -> BufWin {
            BufWin::new(&mut self.window, &mut self.buffer)
        }

        fn cursor(&self) -> crate::editing::CursorPosition {
            self.window.cursor
        }

        fn window(&self) -> &Box<Window> {
            &self.window
        }

        fn window_mut(&mut self) -> &mut Box<Window> {
            &mut self.window
        }
    }

    /// Build a testable Window wrapper based on the visual appearance
    /// of the provided string `s`. This is the basis for many of our
    /// tests, enabling clear, visual descriptions of how content should
    /// appear before and after some action.
    ///
    /// A few characters are special within the string:
    /// `|` - The first pipe character encountered marks where the cursor
    ///       should appear. If not included, the resulting Window's
    ///       cursor will be at the "default" position (0, 0)
    /// `~` - If a line consists only of a single tilde, that line is used
    ///       only as a visual placeholder to indicate window size, and is
    ///       not considered part of the backing buffer. This is based on
    ///       how Vim renders extra space in a window when the end of the
    ///       buffer is reached.
    /// `_` - If a line consists only of a single underscore, that line is
    ///       treated as an entirely blank line. This lets us avoid relying
    ///       on hard-to-see trailing whitespace to indicate this state.
    ///
    /// In addition:
    /// `--INSERT--` - Can be the last line in the string to explicitly indicate
    /// that the window should be "inserting"
    pub fn window(s: &'static str) -> TestWindow {
        let s: String = s.into();
        let mut cursor = CursorPosition::default();
        let mut buffer = Box::new(MemoryBuffer::new(FIRST_USER_BUFFER_ID));
        let mut non_buffer_lines = 0;
        let mut inserting = false;

        for (index, line) in s.lines().enumerate() {
            if line == "--INSERT--" {
                inserting = true;
                continue;
            }

            if let Some(col) = line.find("|") {
                cursor.line = index - non_buffer_lines;
                cursor.col = col;
            }

            if line == "~" {
                non_buffer_lines += 1;
            } else if line == "_" {
                buffer.append(TextLine::from("").into());
            } else {
                // NOTE: we we just use TextLines::from, that will
                // convert an empty line into an empty TextLines vec,
                // which is incorrect
                buffer.append(TextLine::from(line.replace("|", "")).into());
            }
        }

        let mut window = Window::new(0, buffer.id());
        window.cursor = cursor;
        window.inserting = inserting;

        let height = max(1, s.chars().filter(|ch| *ch == '\n').count());
        window.resize(Size {
            w: 20,
            h: height as u16,
        });

        TestWindow {
            window: Box::new(window),
            buffer: UndoableBuffer::wrap(buffer),
            commands: CommandRegistry::default(),
        }
    }

    mod apply_cursor {
        use crate::{
            editing::motion::{
                linewise::DownLineMotion,
                word::{is_small_word_boundary, WordMotion},
            },
            tui::rendering::display::tests::TestableDisplay,
        };

        use super::*;
        use indoc::indoc;

        #[test]
        fn adjusts_scroll_up() {
            let mut ctx = window(indoc! {"
                Take my love
                |Take my land
            "});
            ctx.window.resize(Size { w: 12, h: 1 });

            ctx.motion(WordMotion::backward_until(is_small_word_boundary));

            ctx.render_at_own_size().assert_visual_match(indoc! {"
                Take my |love
            "});
        }

        #[test]
        fn adjusts_wrapped_scroll_up() {
            let mut ctx = window(indoc! {"
                Take my love |Take my land
            "});
            ctx.window.resize(Size { w: 12, h: 1 });
            ctx.render_at_own_size().assert_visual_match(indoc! {"
                |Take my land
            "});

            ctx.motion(WordMotion::backward_until(is_small_word_boundary));

            ctx.render_at_own_size().assert_visual_match(indoc! {"
                Take my |love
            "});
        }

        #[test]
        fn adjusts_scroll_down() {
            let mut ctx = window(indoc! {"
                Take my |love
                Take my land
            "});
            ctx.window.resize(Size { w: 12, h: 1 });
            ctx.scroll_lines(1);
            ctx.render_at_own_size().assert_visual_match(indoc! {"
                Take my |love
            "});

            ctx.motion(WordMotion::forward_until(is_small_word_boundary));

            ctx.render_at_own_size().assert_visual_match(indoc! {"
                |Take my land
            "});
        }

        #[test]
        fn adjusts_wrapped_scroll_down() {
            let mut ctx = window(indoc! {"
                Take my |love Take my land
            "});
            ctx.window.resize(Size { w: 12, h: 1 });
            ctx.scroll_lines(1);
            ctx.render_at_own_size().assert_visual_match(indoc! {"
                Take my |love
            "});

            ctx.motion(WordMotion::forward_until(is_small_word_boundary));

            ctx.render_at_own_size().assert_visual_match(indoc! {"
                |Take my land
            "});
        }

        #[test]
        fn handles_empty_lines() {
            let mut ctx = window(indoc! {"
                Take my |love
                
            "});
            ctx.window.resize(Size { w: 12, h: 2 });
            ctx.motion(DownLineMotion {});
            ctx.motion(DownLineMotion {});

            ctx.render_at_own_size().assert_visual_match(indoc! {"
                Take my love
                |
            "});
        }

        #[test]
        fn follows_cursor_smoothly_down() {
            let mut ctx = window(indoc! {"
                |Take my love
                Take my land
                Take me where
                I cannot
                stand
            "});
            ctx.window.resize(Size { w: 14, h: 2 });
            ctx.render_at_own_size().assert_visual_match(indoc! {"
                |Take my love
                Take my land
            "});

            ctx.motion(DownLineMotion);
            ctx.render_at_own_size().assert_visual_match(indoc! {"
                Take my love
                |Take my land
            "});

            ctx.motion(DownLineMotion);
            ctx.render_at_own_size().assert_visual_match(indoc! {"
                Take my land
                |Take me where
            "});

            ctx.motion(DownLineMotion);
            ctx.render_at_own_size().assert_visual_match(indoc! {"
                Take me where
                |I cannot
            "});

            ctx.motion(DownLineMotion);
            ctx.render_at_own_size().assert_visual_match(indoc! {"
                I cannot
                |stand
            "});
        }
    }
}
