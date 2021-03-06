use crate::input::{keys::KeysParsable, RemapMode};
use command_decl::declare_commands;

declare_commands!(declare_mapping {
    pub fn nmap(context, from: String, to: String) {
        context.keymap.remap_keys(RemapMode::VimNormal, from.into_keys(), to.into_keys());

        Ok(())
    }
});
