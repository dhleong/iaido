use super::{
    motions::{vim_linewise_motions, vim_standard_motions},
    VimKeymap, VimMode,
};

use crate::{
    editing::motion::{linewise::FullLineMotion, Motion},
    input::{Key, KeymapContext},
    key_handler,
};

pub fn vim_operator_pending_mode(linewise: bool, op_repeat_key: Key) -> VimMode {
    let mut mappings = vim_standard_motions();
    if linewise {
        mappings = mappings + vim_linewise_motions();
    }
    mappings.insert(
        &[op_repeat_key],
        Box::new(key_handler!(
            VimKeymap | ctx | {
                if let Some(op) = ctx.keymap.operator_fn.take() {
                    let motion_impl = FullLineMotion;
                    let motion = motion_impl.range(ctx.state());
                    op(&mut ctx, motion)
                } else {
                    Ok(())
                }
            }
        )),
    );

    VimMode::new("o", mappings)
        .with_shows_keys(true)
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
