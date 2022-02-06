use std::io;

use crate::{
    connection::ReadValue,
    editing::{source::BufferSource, window::WindowFlags},
};
use crate::{
    editing::Id,
    input::{
        maps::{KeyHandlerContext, KeyResult},
        BoxableKeymap, KeyError, KeymapContext,
    },
};

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

    let result = send_string_to_buffer(&mut ctx, conn_buffer_id, to_send);

    if result.is_ok() {
        ctx.state_mut().current_buffer_mut().clear();
    }

    result
}

pub fn send_string_to_buffer<K: KeymapContext>(
    ctx: &mut K,
    conn_buffer_id: Id,
    to_send: String,
) -> KeyResult {
    let should_echo = if let Some(conn) = ctx.state_mut().connections.by_buffer_id(conn_buffer_id) {
        conn.send(to_send.clone())?;
        conn.flags.can_echo()
    } else {
        return Err(KeyError::IO(io::ErrorKind::NotConnected.into()));
    };

    if should_echo {
        if let Some(mut output) = ctx.state_mut().winsbuf_by_id(conn_buffer_id) {
            output.append_value(ReadValue::Text(to_send.into()));
            output.append_value(ReadValue::Newline);

            // When sending anything, jump to the end in the "first"
            // PROTECTED (IE: main output) Window for this buffer
            let last_line = output.buffer.lines_count().checked_sub(1).unwrap_or(0);
            if let Some(mut first) =
                output.first_window(|win| win.flags.contains(WindowFlags::PROTECTED))
            {
                first.scrolled_lines = 0;
                first.scroll_offset = 0;
                first.cursor = (last_line, 0).into();
            }
        }
    }

    Ok(())
}
