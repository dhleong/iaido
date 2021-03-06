use crate::{
    declare_commands,
    input::{KeyError, KeymapContext},
};

declare_commands!(declare_core {
    pub fn buffer(context, id: usize) {
        if context.state().buffers.by_id(id).is_some() {
            context.state_mut().set_current_window_buffer(id);
            Ok(())
        } else {
            Err(KeyError::InvalidInput(format!("buffer: {}: Buffer does not exist", id)))
        }
    },

    pub fn quit(context) {
        context.state_mut().running = false;
        Ok(())
    },
});
