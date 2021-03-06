/*!
 * Shared helper logic
 */

use crate::{
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
    if !args.force {
        return Ok(());
    }

    if context.state().current_buffer().is_modified() {
        // TODO actually, if the buffer is saved or open in another window,
        // we should be allowed to hide it.
        return Err(KeyError::NotPermitted(
            "No write since last change".to_string(),
        ));
    }

    if let Some(id) = context.state().current_buffer().connection_buffer_id() {
        if context
            .state_mut()
            .connections
            .as_mut()
            .unwrap()
            .by_buffer_id(id)
            .is_some()
        {
            let name = buffer_connection_name(context, id);
            return Err(KeyError::NotPermitted(format!("{}: Still connected", name)));
        }
    }

    Ok(())
}
