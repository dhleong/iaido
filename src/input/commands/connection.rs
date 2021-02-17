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
});

fn connect(context: &mut CommandHandlerContext, url: String) -> KeyResult {
    let uri = Url::parse(url.as_str())?;
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

    context.state_mut().current_window_mut().buffer = buffer_id;
    context
        .state_mut()
        .current_buffer_mut()
        .append(format!("Connecting to {}...", uri).into());

    // TODO can we redraw first? and/or can this be async?
    context.state_mut().connections.create(buffer_id, uri)?;

    Ok(())
}
