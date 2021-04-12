use std::io;

use crate::input::{
    maps::{KeyHandlerContext, KeyResult},
    BoxableKeymap, KeyError, KeymapContext,
};
use crate::{connection::ReadValue, editing::source::BufferSource};

/// Send the contents of the current Connection input buffer to
/// its associated connection (if any)
///
/// Returns `io::ErrorKind::NotConnected` if there is no Connection
/// associated with the current buffer.
pub fn send_current_input_buffer<T: BoxableKeymap>(mut ctx: KeyHandlerContext<T>) -> KeyResult {
    let buffer = ctx.state().current_buffer();
    let to_send = buffer.get_contents();
    let conn_buffer_id =
        if let BufferSource::ConnectionInputForBuffer(conn_buffer_id) = buffer.source() {
            conn_buffer_id.clone()
        } else {
            return Err(KeyError::IO(io::ErrorKind::NotConnected.into()));
        };

    let mut sent = false;

    if let Some(ref mut conns) = ctx.state_mut().connections {
        if let Some(conn) = conns.by_buffer_id(conn_buffer_id) {
            conn.send(to_send.clone())?;
            sent = true;
        }
    }

    if sent {
        ctx.state_mut().current_buffer_mut().clear();
        if let Some(mut output) = ctx.state_mut().winsbuf_by_id(conn_buffer_id) {
            output.append_value(ReadValue::Text(to_send.into()));
            output.append_value(ReadValue::Newline);
        }
        return Ok(());
    }

    Err(KeyError::IO(io::ErrorKind::NotConnected.into()))
}
