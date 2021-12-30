use url::Url;

use crate::{
    editing::{source::BufferSource, Id},
    input::{maps::KeyResult, KeyError, KeymapContext},
};
use command_decl::declare_commands;

use super::CommandHandlerContext;

declare_commands!(declare_connection {
    //! Connection-management commands

    /// Connect to a telnet server at the given `url`. This will replace your current [Window] with
    /// a new Connection [Window] that has an input [Buffer] and an output [Buffer].
    /// The url can be in one of the following formats:
    ///
    ///   - server.com:port
    ///   - telnet://server.com:port
    ///   - ssl://server.com:port
    pub fn connect(context, url: String) {
        connect(context, url)
    }

    pub fn disconnect(context) {
        disconnect(context)
    }

    /// Reconnect to the most-recently connected server associated with
    /// the current buffer.
    pub fn reconnect(context) {
        let current_buffer_id = context.state().current_buffer().id();
        if context.state_mut().connections.as_mut().and_then(|conns| conns.by_buffer_id(current_buffer_id)).is_some() {
            return Err(KeyError::InvalidInput("Current buffer is still connected!".to_string()));
        }

        if let Some((id, url)) = get_associated_connection(context) {
            context.state_mut().current_tab_mut().set_focus_to_buffer(id);
            connect(context, url)
        } else {
            Err(KeyError::InvalidInput("No associated connection for current buffer".to_string()))
        }
    }
});

fn parse_url(url: &str) -> Result<Url, url::ParseError> {
    if url.find("://").is_none() {
        Url::parse(format!("telnet://{}", url).as_str())
    } else {
        Url::parse(url)
    }
}

pub fn connect(context: &mut CommandHandlerContext, url: String) -> KeyResult {
    let uri = parse_url(url.as_str())?;
    let buffer = context.state().current_buffer();
    let buffer_id = match &buffer.source() {
        &BufferSource::Connection(existing_url) if existing_url == &url => {
            // Reuse
            buffer.id()
        }

        &BufferSource::None if buffer.is_empty() => {
            // Reuse this, too
            buffer.id()
        }

        _ => {
            // Otherwise, create a new buffer for the connection
            let new = context.state_mut().buffers.create_mut();
            new.id()
        }
    };

    let state = context.state_mut();
    if let Some(buf) = state.buffers.by_id_mut(buffer_id) {
        buf.set_source(BufferSource::Connection(url.to_string()));
    }

    let tab = state.tabpages.current_tab_mut();
    let new_window = tab.new_connection(&mut state.buffers, buffer_id);
    let input_buffer_id = new_window.input.buffer;

    tab.replace_window(tab.current_window().id, Box::new(new_window));
    context
        .state_mut()
        .winsbuf_by_id(buffer_id)
        .unwrap()
        .append_line(format!("Connecting to {}...", uri));

    let mut connections = context.state_mut().connections.take().unwrap();
    let job = connections.create_async(
        &mut context.state_mut().jobs,
        buffer_id,
        input_buffer_id,
        uri,
    );
    context.state_mut().connections = Some(connections);

    match job.join_interruptably(context) {
        Ok(_) => Ok(()),

        // write the error to the buffer, if possible
        Err(e) => {
            if let Some(mut win) = context.state_mut().winsbuf_by_id(buffer_id) {
                match e {
                    KeyError::Interrupted => {
                        win.append_line("Canceled.".into());
                    }
                    e => {
                        win.append_line(format!("Error: {:?}.", e).into());
                    }
                }
                Ok(())
            } else {
                Err(e)
            }
        }
    }
}

fn disconnect(context: &mut CommandHandlerContext) -> KeyResult {
    if let Some(buffer_id) = context.state().current_buffer().connection_buffer_id() {
        context
            .state_mut()
            .connections
            .as_mut()
            .unwrap()
            .disconnect_buffer(buffer_id)?;

        on_disconnect(context, buffer_id);

        Ok(())
    } else {
        Err(KeyError::InvalidInput(
            "No connection for current buffer".to_string(),
        ))
    }
}

pub fn on_disconnect(context: &mut CommandHandlerContext, buffer_id: Id) {
    context
        .state_mut()
        .winsbuf_by_id(buffer_id)
        .expect("Could not find current buffer")
        .append_line("Disconnected.".into());
}

/// If possible, returns the buffer ID and URL of a connection (possibly defunct) associated with
/// the current buffer
pub fn get_associated_connection(context: &CommandHandlerContext) -> Option<(Id, String)> {
    let buffer = context.state().current_buffer();
    match &buffer.source() {
        &BufferSource::Connection(url) => Some((buffer.id(), url.to_string())),
        &BufferSource::ConnectionInputForBuffer(buffer_id) => {
            match context
                .state()
                .buffers
                .by_id(buffer_id.to_owned())
                .map(|buf| buf.source())
            {
                Some(BufferSource::Connection(url)) => Some((*buffer_id, url.to_string())),
                _ => None,
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod parse_url {
        use super::*;

        #[test]
        fn defaults_to_telnet() {
            assert_eq!(parse_url("serenity.co"), Url::parse("telnet://serenity.co"));
        }
    }
}
