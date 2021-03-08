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
use std::{fs, path::PathBuf};

use crate::{declare_commands, input::KeymapContext};

use super::helpers::{check_hide_buffer, HideBufArgs};
use super::CommandHandlerContext;

declare_commands!(declare_file {
    pub fn edit(context, file_path: PathBuf) {
        check_hide_buffer(context, HideBufArgs { force: false })?;

        let (full_path_string, lines) = if file_path.exists() {
            let contents = fs::read_to_string(&file_path)?;
            let bytes = contents.as_bytes().len();
            let lines: Vec<TextLine> = contents.split("\n").map(|line| line.to_owned().into()).collect();
            let lines_count = lines.len();

            let canonical = file_path.canonicalize().unwrap();
            let full_path_string = canonical.to_string_lossy().to_string();
            context.state_mut().echom(format!("\"{}\": {}L, {}B", full_path_string, lines_count, bytes));

            (full_path_string, lines)
        } else {
            let full_path_string = file_path.to_string_lossy().to_string();
            context.state_mut().echom(format!("\"{}\": [New]", full_path_string));

            (full_path_string, vec![])
        };

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

    pub fn write(context, given_path: Optional<PathBuf>) {
        let current_path = match context.state().current_buffer().source() {
            &BufferSource::LocalFile(ref path) => Some(path.clone()),
            _ => None,
        };

        let path = if let Some(path) = given_path {
            path
        } else if let Some(path) = current_path {
            PathBuf::from(path.clone())
        } else {
            return Err(KeyError::InvalidInput("No file name".to_string()));
        };

        write(context, path)
    },
});

fn write(context: &mut CommandHandlerContext, path: PathBuf) -> KeyResult {
    let content = context.state().current_buffer().get_contents();
    let lines_count = context.state().current_buffer().lines_count();
    let bytes = content.as_bytes().len();

    fs::write(&path, content)?;

    context.state_mut().echom(format!(
        "\"{}\": {}L, {}B written",
        path.to_string_lossy(),
        lines_count,
        bytes,
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
