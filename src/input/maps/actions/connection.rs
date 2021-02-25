use std::io;

use crate::editing::source::BufferSource;
use crate::input::{
    maps::{KeyHandlerContext, KeyResult},
    KeyError, KeymapContext,
};

/// Send the contents of the current Connection input buffer to
/// its associated connection (if any)
///
/// Returns `io::ErrorKind::NotConnected` if there is no Connection
/// associated with the current buffer.
pub fn send_current_input_buffer<T>(mut ctx: KeyHandlerContext<T>) -> KeyResult {
    let buffer = ctx.state().current_buffer();
    let to_send = buffer.get_contents();
    let conn_buffer_id =
        if let BufferSource::ConnectionInputForBuffer(conn_buffer_id) = buffer.source() {
            conn_buffer_id.clone()
        } else {
            return Err(KeyError::IO(io::ErrorKind::NotConnected.into()));
        };

    ctx.state_mut().current_buffer_mut().clear();
    if let Some(ref mut conns) = ctx.state_mut().connections {
        if let Some(conn) = conns.by_buffer_id(conn_buffer_id) {
            conn.send(to_send.clone())?;
            return Ok(());
        }
    }

    Err(KeyError::IO(io::ErrorKind::NotConnected.into()))
}
