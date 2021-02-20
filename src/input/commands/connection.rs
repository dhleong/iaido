use url::Url;

use crate::{
    declare_commands,
    editing::source::BufferSource,
    input::{maps::KeyResult, KeymapContext},
};

use super::CommandHandlerContext;

declare_commands!(declare_connection {
    pub fn connect(context, url: String) {
        connect(context, url)
    },

    pub fn disconnect(context) {
        disconnect(context)
    },
});

fn parse_url(url: &str) -> Result<Url, url::ParseError> {
    if url.find("://").is_none() {
        Url::parse(format!("telnet://{}", url).as_str())
    } else {
        Url::parse(url)
    }
}

fn connect(context: &mut CommandHandlerContext, url: String) -> KeyResult {
    let uri = parse_url(url.as_str())?;
    let buffer = context.state().current_buffer();
    let buffer_id = match &buffer.source() {
        &BufferSource::Connection(existing_url) if existing_url == &url => {
            // reuse
            buffer.id()
        }

        _ => {
            // otherwise, create a new buffer for the connection
            let new_id = context.state_mut().buffers.create().id();

            context
                .state_mut()
                .buffers
                .by_id_mut(new_id)
                .expect("Couldn't find newly-created buffer")
                .set_source(BufferSource::Connection(url.to_string()));

            new_id
        }
    };

    context.state_mut().set_current_window_buffer(buffer_id);
    context
        .state_mut()
        .current_winsbuf()
        .append_line(format!("Connecting to {}...", uri));

    let connections = context
            .state_mut()
            .connections
            .as_mut()
            .unwrap().factories.clone();
    context.state_mut().jobs.spawn(move |ctx| async move {
        ctx.echo("Connecting...".into())?;
        connections.create(buffer_id, uri)?;
        ctx.echo("Connected".into())?;
        Ok(())
    });

    Ok(())
}

fn disconnect(context: &mut CommandHandlerContext) -> KeyResult {
    let buffer_id = context.state().current_buffer().id();
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
