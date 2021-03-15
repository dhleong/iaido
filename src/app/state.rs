use std::sync::{Arc, Mutex};

use crate::{
    connection::connections::Connections,
    editing::{
        buffer::MemoryBuffer,
        buffers::Buffers,
        motion::char::CharMotion,
        motion::{Motion, MotionContext},
        tabpage::Tabpage,
        tabpages::Tabpages,
        text::{EditableLine, TextLine, TextLines},
        window::Window,
        Buffer, Id, Resizable, Size,
    },
    input::{
        commands::{create_builtin_commands, registry::CommandRegistry},
        completion::CompletableContext,
    },
    script::{ApiManager, ScriptingManager},
};

use super::{bufwin::BufWin, jobs::Jobs, prompt::Prompt, widgets::Widget, winsbuf::WinsBuf};

pub struct AppState {
    pub running: bool,
    pub requested_redraw: bool,

    pub buffers: Buffers,
    pub tabpages: Tabpages,
    pub echo_buffer: Box<dyn Buffer>,
    pub prompt: Prompt,
    pub builtin_commands: CommandRegistry,

    pub keymap_widget: Option<Widget>,

    // Connections should generally be available, but is an
    // Option so callers may temporarily take ownership of it
    pub connections: Option<Connections>,

    pub scripting: Arc<Mutex<ScriptingManager>>,
    pub api: Option<ApiManager>,

    pub jobs: Jobs,
}

impl AppState {
    pub fn current_buffer<'a>(&'a self) -> &'a Box<dyn Buffer> {
        if self.prompt.window.focused {
            return &self.prompt.buffer;
        }

        self.current_window().current_buffer(&self.buffers)
    }

    pub fn current_buffer_mut<'a>(&'a mut self) -> &'a mut Box<dyn Buffer> {
        if self.prompt.window.focused {
            return &mut self.prompt.buffer;
        }

        // NOTE: if we just use self.current_window(), rust complains that we've already immutably
        // borrowed self.buffers, so we go the long way:
        self.tabpages
            .current_tab()
            .current_window()
            .current_buffer_mut(&mut self.buffers)
    }

    pub fn current_window<'a>(&'a self) -> &'a Box<Window> {
        if self.prompt.window.focused {
            return &self.prompt.window;
        }
        self.current_tab().current_window()
    }

    pub fn current_window_mut<'a>(&'a mut self) -> &'a mut Box<Window> {
        if self.prompt.window.focused {
            return &mut self.prompt.window;
        }
        self.current_tab_mut().current_window_mut()
    }

    pub fn current_tab<'a>(&'a self) -> &'a Box<Tabpage> {
        self.tabpages.current_tab()
    }

    pub fn current_tab_mut<'a>(&'a mut self) -> &'a mut Box<Tabpage> {
        self.tabpages.current_tab_mut()
    }

    pub fn current_bufwin<'a>(&'a mut self) -> BufWin<'a> {
        if self.prompt.window.focused {
            BufWin::new(&mut self.prompt.window, &mut self.prompt.buffer)
        } else {
            let window_id = self.tabpages.current_tab_mut().current_window().id;
            if let Some(bufwin) = self.bufwin_by_id(window_id) {
                return bufwin;
            }

            panic!("Unable to locate current window/buffer");
        }
    }

    pub fn bufwin_by_id<'a>(&'a mut self, window_id: usize) -> Option<BufWin<'a>> {
        if let Some(tabpage) = self.tabpages.containing_window_mut(window_id) {
            if let Some(window) = tabpage.by_id_mut(window_id) {
                if let Some(buffer) = self.buffers.by_id_mut(window.buffer) {
                    return Some(BufWin::new(window, buffer));
                }
            }
        }
        None
    }

    pub fn winsbuf_by_id<'a>(&'a mut self, buffer_id: Id) -> Option<WinsBuf<'a>> {
        if let Some(buffer) = self.buffers.by_id_mut(buffer_id) {
            let windows = self.tabpages.windows_for_buffer(buffer_id);
            Some(WinsBuf::new(windows.collect(), buffer))
        } else {
            None
        }
    }

    // ======= redraw =========================================

    pub fn request_redraw(&mut self) {
        self.requested_redraw = true;
    }

    // ======= echo ===========================================

    pub fn clear_echo(&mut self) {
        self.echo_buffer.clear();
    }

    pub fn echo(&mut self, text: TextLines) {
        if self.requested_redraw {
            self.requested_redraw = false;
            self.clear_echo();
        }

        self.echo_buffer.append(text);
    }

    pub fn echo_str(&mut self, text: &'static str) {
        self.echo(text.into());
    }

    pub fn echom<T: Into<TextLines>>(&mut self, text: T) {
        let lines = text.into();
        for line in &lines.lines {
            crate::info!("{}", line.to_string());
        }
        self.echo(lines);
    }

    // ======= keymap conveniences ============================

    pub fn backspace(&mut self) {
        CharMotion::Backward(1).delete_range(self);
    }

    pub fn insert_at_cursor(&mut self, text: TextLine) {
        let cursor = self.current_window().cursor;
        let buffer = self.current_buffer_mut();
        buffer.insert(cursor, text);
    }

    pub fn type_at_cursor(&mut self, ch: char) {
        self.insert_at_cursor(String::from(ch).into());
        self.current_window_mut().cursor.col += 1;
        self.current_window_mut().completion_state = None; // reset on type
    }

    // ======= buf/win cross-modification =====================

    pub fn set_current_window_buffer(&mut self, new_id: Id) {
        self.current_window_mut().buffer = new_id;
        let buffer = self
            .buffers
            .by_id(new_id)
            .expect("Could not find new buffer");
        let cursor = self.current_window().cursor;

        let clamped_cursor = self.current_window().clamp_cursor(buffer, cursor);
        let mut window = self.current_window_mut();
        window.cursor = clamped_cursor;
        window.scrolled_lines = 0;
        window.scroll_offset = 0;
    }
}

impl Default for AppState {
    fn default() -> Self {
        let buffers = Buffers::new();
        let tabpages = Tabpages::new(Size { w: 0, h: 0 });
        let mut app = Self {
            running: true,
            requested_redraw: true,
            buffers,
            tabpages,
            echo_buffer: Box::new(MemoryBuffer::new(0)),
            prompt: Prompt::default(),
            builtin_commands: create_builtin_commands(),
            keymap_widget: None,
            connections: Some(Connections::default()),
            scripting: Arc::new(Mutex::new(ScriptingManager::default())),
            api: Some(ApiManager::default()),
            jobs: Jobs::new(),
        };

        // create the default tabpage
        let default_id = app.tabpages.create(&mut app.buffers);
        app.tabpages.current = default_id;

        app
    }
}

impl Resizable for AppState {
    fn resize(&mut self, new_size: Size) {
        self.tabpages.resize(new_size);
        self.prompt.resize(new_size);
    }
}

impl CompletableContext for AppState {
    fn bufwin(&mut self) -> BufWin {
        self.current_bufwin()
    }

    fn commands(&self) -> &CommandRegistry {
        &self.builtin_commands
    }
}

impl MotionContext for AppState {
    fn buffer(&self) -> &Box<dyn Buffer> {
        self.current_buffer()
    }

    fn buffer_mut(&mut self) -> &mut Box<dyn Buffer> {
        self.current_buffer_mut()
    }

    fn bufwin(&mut self) -> BufWin {
        self.current_bufwin()
    }

    fn cursor(&self) -> crate::editing::CursorPosition {
        self.window().cursor
    }

    fn window(&self) -> &Box<Window> {
        self.current_window()
    }

    fn window_mut(&mut self) -> &mut Box<Window> {
        self.current_window_mut()
    }
}
