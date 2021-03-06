/*!
 * File IO commands
 */

use crate::{
    editing::{
        source::BufferSource,
        text::{TextLine, TextLines},
    },
    input::{maps::KeyResult, KeyError},
};
use std::fs;

use crate::{declare_commands, input::KeymapContext};

use super::helpers::{check_hide_buffer, HideBufArgs};
use super::CommandHandlerContext;

declare_commands!(declare_file {
    pub fn edit(context, file_path: String) {
        check_hide_buffer(context, HideBufArgs { force: false })?;

        // TODO if the file doesn't exist, we should still be able to edit it
        let full_path = fs::canonicalize(&file_path)?;
        let contents = fs::read_to_string(&full_path)?;
        let bytes = contents.as_bytes().len();
        let lines: Vec<TextLine> = contents.split("\n").map(|line| line.to_owned().into()).collect();
        let lines_count = lines.len();

        let full_path_string = full_path.to_string_lossy();
        context.state_mut().echom(format!("\"{}\": {}L, {}B", full_path_string, lines_count, bytes));

        let buffer_id = {
            let buf = context.state_mut().buffers.create_mut();
            buf.append(TextLines::from(lines));
            buf.set_source(BufferSource::LocalFile(full_path_string.to_string()));
            buf.changes().clear();
            buf.id()
        };

        context.state_mut().set_current_window_buffer(buffer_id);

        Ok(())
    },

    pub fn write(context, given_path: Optional<String>) {
        let current_path = match context.state().current_buffer().source() {
            &BufferSource::LocalFile(ref path) => Some(path.clone()),
            _ => None,
        };

        let path = if let Some(path) = given_path {
            path
        } else if let Some(path) = current_path {
            path.clone()
        } else {
            return Err(KeyError::InvalidInput("No file name".to_string()));
        };

        write(context, path)
    },
});

fn write(context: &mut CommandHandlerContext, path: String) -> KeyResult {
    let content = context.state().current_buffer().get_contents();
    let lines_count = context.state().current_buffer().lines_count();
    let bytes = content.as_bytes().len();

    fs::write(&path, content)?;

    context.state_mut().echom(format!(
        "\"{}\": {}L, {}B written",
        path, lines_count, bytes,
    ));

    // if we don't already have a source, set it
    if context.state().current_buffer().source().is_none() {
        let canonical = fs::canonicalize(path)?;
        context
            .state_mut()
            .current_buffer_mut()
            .set_source(BufferSource::LocalFile(
                canonical.to_string_lossy().to_string(),
            ));
    }

    Ok(())
}
