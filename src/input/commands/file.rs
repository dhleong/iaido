/*!
 * File IO commands
 */

use crate::{declare_commands, input::KeymapContext};

declare_commands!(declare_file {
    pub fn edit(context, file_path: String) {
        // TODO find an existing buffer, or read into a new one and
        // update our window
        context.state_mut().echo(format!("{}: TODO: read", file_path).into());
        Ok(())
    },
});
