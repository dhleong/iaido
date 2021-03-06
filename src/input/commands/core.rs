use crate::input::KeyError;
use crate::{declare_commands, input::KeymapContext};

declare_commands!(declare_core {
    pub fn buffer(context, id: String) {
        match id.parse::<usize>() {
            Ok(id) => {
                context.state_mut().set_current_window_buffer(id);
                Ok(())
            },
            Err(e) => Err(KeyError::InvalidInput(e.to_string())),
        }
    },

    pub fn quit(context) {
        context.state_mut().running = false;
        Ok(())
    },
});
