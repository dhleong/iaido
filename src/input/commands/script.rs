/*!
 * Script-related commands
 */

use std::path::PathBuf;

use command_decl::declare_commands;

use crate::{input::maps::KeyResult, script::ScriptingManager};

use super::CommandHandlerContext;

declare_commands!(declare_script {
    /// Load a script file
    pub fn source(context, file_path: PathBuf) {
        source_path(context, file_path)
    },
});

fn source_path(context: &mut CommandHandlerContext, file_path: PathBuf) -> KeyResult {
    // TODO Stash file_path somewhere so we can :reload

    ScriptingManager::load_scripts(
        &mut context.context,
        &mut context.keymap,
        vec![file_path.to_string_lossy().to_string()],
    );

    Ok(())
}
