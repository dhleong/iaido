use crate::{
    declare_commands,
    input::{maps::KeyResult, KeyError, KeymapContext},
};

use super::{
    helpers::{buffer_connection_name, check_hide_buffer, HideBufArgs},
    CommandHandlerContext,
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
        quit_window(context, HideBufArgs { force: false })
    },
});

fn quit_window(context: &mut CommandHandlerContext, args: HideBufArgs) -> KeyResult {
    check_hide_buffer(context, args)?;

    let connection_buffer_id = context.state().current_buffer().connection_buffer_id();
    let win_id = context.state().current_window().id;
    context.state_mut().current_tab_mut().close_window(win_id);

    if let Some(id) = connection_buffer_id {
        // make sure we disconnect if there are no more windows
        if context
            .state_mut()
            .tabpages
            .containing_buffer_mut(id)
            .is_none()
        {
            let name = buffer_connection_name(context, id);
            context
                .state_mut()
                .echom(format!("{}: Disconnected.", name));

            context
                .state_mut()
                .connections
                .as_mut()
                .unwrap()
                .disconnect_buffer(id)?;
        }
    }

    if !context.state().tabpages.has_edit_windows() {
        context.state_mut().running = false;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        editing::motion::tests::TestKeymapContext, input::source::memory::MemoryKeySource,
    };

    use super::*;

    #[test]
    fn quit_single_windows_test() -> KeyResult {
        let mut state = crate::app::State::default();
        state.current_tab_mut().vsplit();

        let mut context = TestKeymapContext {
            state,
            keys: MemoryKeySource::from_keys(""),
        };
        let mut ctx = CommandHandlerContext {
            context: Box::new(&mut context),
            input: "q".to_string(),
        };

        quit_window(&mut ctx, HideBufArgs { force: false })?;
        assert_eq!(ctx.context.state_mut().running, true);
        assert_eq!(ctx.context.state_mut().current_window().focused, true);

        quit_window(&mut ctx, HideBufArgs { force: false })?;
        assert_eq!(ctx.context.state_mut().running, false);

        Ok(())
    }
}
