/*!
 * Script-related commands
 */

use std::path::PathBuf;

use command_decl::declare_commands;

use crate::{
    input::{maps::KeyResult, KeymapContext},
    script::ScriptingManager,
};

use super::CommandHandlerContext;

declare_commands!(declare_script {
    /// Re-load the most-recently :sourced a script file for the current buffer
    pub fn reload(context) {
        reload_buffer_source(context)
    },

    /// Load a script file
    pub fn source(context, file_path: PathBuf) {
        source_path(context, file_path)
    },
});

fn reload_buffer_source(context: &mut CommandHandlerContext) -> KeyResult {
    if let Some(path) = context
        .state_mut()
        .current_buffer_mut()
        .config_mut()
        .loaded_script
        .take()
    {
        source_path(context, path)?;
    } else {
        // TODO Probably, check for an associated buffer via Connections
        // to handle running this while eg focusing the input buffer
        context.state_mut().echom("No script loaded in this bufer");
    }
    Ok(())
}

fn source_path(context: &mut CommandHandlerContext, file_path: PathBuf) -> KeyResult {
    let buffer_id = context.state().current_buffer().id();
    if let Some(conn) = context
        .state_mut()
        .connections
        .as_mut()
        .and_then(|conns| conns.by_buffer_id(buffer_id))
    {
        // Clear config on any associated connection
        conn.game.reset();
    }

    let path_str = file_path.to_string_lossy().to_string();
    ScriptingManager::load_script(&mut context.context, &mut context.keymap, file_path);
    context.state_mut().echom(format!("Sourced {}", path_str));
    Ok(())
}
