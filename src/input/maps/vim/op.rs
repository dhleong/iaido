use super::{
    motions::{vim_linewise_motions, vim_standard_motions},
    VimKeymap, VimMode,
};

use crate::key_handler;

pub fn vim_operator_pending_mode(linewise: bool) -> VimMode {
    let mut mappings = vim_standard_motions();
    if linewise {
        mappings = mappings + vim_linewise_motions();
    }

    VimMode::new("o", mappings)
        .on_default(key_handler!(
            VimKeymap | ?mut ctx | {
                // If a key is unhandled, leave operator-pending mode
                ctx.keymap.pop_mode("o");
                Ok(())
            }
        ))
        .on_exit(key_handler!(
            VimKeymap | ctx | {
                // Always ensure operator_fn is cleared
                ctx.keymap.operator_fn = None;
                Ok(())
            }
        ))
}
