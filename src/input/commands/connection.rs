use url::Url;

use crate::{
    editing::source::BufferSource,
    input::{maps::KeyResult, KeyError, KeymapContext},
};
use command_decl::declare_commands;

use super::CommandHandlerContext;

declare_commands!(declare_connection {
    pub fn connect(context, url: String) {
        connect(context, url)
    }

    pub fn disconnect(context) {
        disconnect(context)
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
            // reuse
            buffer.id()
        }

        _ => {
            // otherwise, create a new buffer for the connection
            let new = context.state_mut().buffers.create_mut();
            new.set_source(BufferSource::Connection(url.to_string()));
            new.id()
        }
    };

    let state = context.state_mut();
    let tab = state.tabpages.current_tab_mut();
    let new_window = tab.new_connection(&mut state.buffers, buffer_id);

    tab.replace_window(tab.current_window().id, Box::new(new_window));
    context
        .state_mut()
        .winsbuf_by_id(buffer_id)
        .unwrap()
        .append_line(format!("Connecting to {}...", uri));

    let mut connections = context.state_mut().connections.take().unwrap();
    let job = connections.create_async(&mut context.state_mut().jobs, buffer_id, uri);
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

        context
            .state_mut()
            .winsbuf_by_id(buffer_id)
            .expect("Could not find current buffer")
            .append_line("Disconnected.".into());

        Ok(())
    } else {
        Err(KeyError::InvalidInput(
            "No connection for current buffer".to_string(),
        ))
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
