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
        let is_connected = context
            .state_mut()
            .connections
            .as_mut()
            .unwrap()
            .by_buffer_id(id)
            .is_some();
        if is_connected
            && context
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
    use crate::editing::motion::tests::TestKeyHandlerContext;

    use super::*;

    #[test]
    fn quit_single_windows_test() -> KeyResult {
        let mut context = TestKeyHandlerContext::empty();
        let mut ctx = context.command_context("q");
        ctx.state_mut().current_tab_mut().vsplit();

        quit_window(&mut ctx, HideBufArgs { force: false })?;
        assert_eq!(ctx.context.state_mut().running, true);
        assert_eq!(ctx.context.state_mut().current_window().focused, true);

        quit_window(&mut ctx, HideBufArgs { force: false })?;
        assert_eq!(ctx.context.state_mut().running, false);

        Ok(())
    }

    #[test]
    fn quit_connection_window_test() -> KeyResult {
        let mut context = TestKeyHandlerContext::empty();
        let mut ctx = context.command_context("q");

        let state = ctx.state_mut();
        let buf_id = state.buffers.create().id();
        let tab = state.tabpages.current_tab_mut();
        tab.new_connection(&mut state.buffers, buf_id);

        // NOTE: it should close both the input window and the output
        quit_window(&mut ctx, HideBufArgs { force: false })?;
        assert_eq!(ctx.context.state_mut().running, false);

        Ok(())
    }
}
