use crate::declare_commands;

declare_commands!(declare_core {
    pub fn quit(context) {
        context.state_mut().running = false;
        Ok(())
    },
});
