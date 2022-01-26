use crate::editing::ids::BUFFER_ID_LOG;
use crate::input::KeymapContext;
use crate::log;
use command_decl::declare_commands;

declare_commands!(declare_log {
    pub fn messages(context) {
        context.state_mut().current_tab_mut().split_bottom();
        context.state_mut().set_current_window_buffer(BUFFER_ID_LOG)?;

        let buf = context.state_mut().current_buffer_mut();
        buf.clear();
        log::write_to_buffer(buf);

        Ok(())
    }
});
