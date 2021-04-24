/*!
 * Shared helper logic
 */

use crate::{
    editing::window::WindowFlags,
    editing::{source::BufferSource, Id},
    input::{maps::KeyResult, KeyError, KeymapContext},
};

use super::CommandHandlerContext;

pub struct HideBufArgs {
    pub force: bool,
}

pub fn buffer_connection_name(context: &CommandHandlerContext, id: Id) -> String {
    if let Some(buf) = context.state().buffers.by_id(id) {
        if let BufferSource::Connection(uri) = buf.source() {
            // it should
            uri.clone()
        } else {
            format!("{:?}", buf.source())
        }
    } else {
        let conn_id = context
            .state()
            .connections
            .as_ref()
            .unwrap()
            .buffer_to_id(id)
            .unwrap_or(id);
        format!("Connection#{}", conn_id)
    }
}

pub fn check_hide_buffer(context: &mut CommandHandlerContext, args: HideBufArgs) -> KeyResult {
    if args.force {
        return Ok(());
    }

    let connection_buffer_id = context.state().current_buffer().connection_buffer_id();
    let buf_id_to_protect = if let Some(id) = connection_buffer_id {
        id
    } else {
        context.state().current_buffer().id()
    };
    let has_other_window = context
        .state_mut()
        .tabpages
        .windows_for_buffer(buf_id_to_protect)
        .count()
        > 1;

    if context.state().current_buffer().is_modified() && !has_other_window {
        return Err(KeyError::NotPermitted(
            "No write since last change".to_string(),
        ));
    }

    if let Some(id) = connection_buffer_id {
        let is_connected = context
            .state_mut()
            .connections
            .as_mut()
            .unwrap()
            .by_buffer_id(id)
            .is_some();

        // disallow closing the main window
        let name = buffer_connection_name(context, id);
        if is_connected
            && has_other_window
            && context
                .state()
                .current_window()
                .flags
                .contains(WindowFlags::PROTECTED)
        {
            return Err(KeyError::NotPermitted(format!(
                "{}: You may not close the primary connection windows",
                name
            )));
        } else if is_connected && !has_other_window {
            return Err(KeyError::NotPermitted(format!("{}: Still connected", name)));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod check_hide_buffer {
        use crate::editing::FocusDirection;
        use crate::{connection::Connection, editing::motion::tests::TestKeyHandlerContext};

        use super::*;

        struct TestConnection;

        impl Connection for TestConnection {
            fn id(&self) -> Id {
                0
            }

            fn read(&mut self) -> std::io::Result<Option<crate::connection::ReadValue>> {
                Ok(None)
            }

            fn write(&mut self, _bytes: &[u8]) -> std::io::Result<()> {
                Ok(())
            }
        }

        #[test]
        fn disallow_hiding_connlayout() -> KeyResult {
            let mut context = TestKeyHandlerContext::empty();
            let mut ctx = context.command_context("q");

            let state = ctx.state_mut();
            let buffer = state.buffers.create_mut();
            buffer.set_source(BufferSource::Connection("serenity.co".to_string()));
            let buf_id = buffer.id();
            let tab = state.tabpages.current_tab_mut();
            let conn_layout = tab.new_connection(&mut state.buffers, buf_id);
            tab.replace_window(tab.current_window().id, Box::new(conn_layout));

            state
                .connections
                .as_mut()
                .unwrap()
                .add_for_test(buf_id, Box::new(TestConnection));

            // we should not be able to hide a single, connected window
            let connected = check_hide_buffer(&mut ctx, HideBufArgs { force: false });
            match connected {
                Ok(_) => panic!("Should not allow connected window hide!"),
                Err(e) => {
                    assert!(format!("{:?}", e).contains("Still connected"));
                }
            }

            // a split window should be hideable
            let split_id = ctx.state_mut().current_tab_mut().vsplit();
            ctx.state_mut().current_tab_mut().set_focus(split_id);
            match check_hide_buffer(&mut ctx, HideBufArgs { force: false }) {
                Ok(_) => {}
                Err(e) => std::panic::panic_any(e),
            }

            // the original connlayout should NOT be hideable
            let tab = ctx.state_mut().current_tab_mut();
            tab.move_focus(FocusDirection::Left);
            tab.move_focus(FocusDirection::Up);

            assert_eq!(ctx.state().current_buffer().id(), buf_id);

            let layout_result = check_hide_buffer(&mut ctx, HideBufArgs { force: false });
            match layout_result {
                Ok(_) => panic!("Should not allow main window hide!"),
                Err(e) => {
                    assert!(format!("{:?}", e).contains("primary"));
                }
            }

            Ok(())
        }
    }
}
