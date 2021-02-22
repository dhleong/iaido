use crate::{declare_commands, input::KeymapContext};

declare_commands!(declare_window {
    pub fn vsplit(context) {
        context.state_mut().current_tab_mut().vsplit();
        Ok(())
    },
});
