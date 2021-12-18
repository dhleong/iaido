use super::{
    motions::{vim_linewise_motions, vim_standard_motions},
    normal::count,
    object::vim_standard_objects,
    VimKeymap, VimMode,
};

use crate::{
    editing::motion::{linewise::FullLineMotion, Motion},
    input::{Key, KeymapContext},
    key_handler,
};

pub fn vim_operator_pending_mode(linewise: bool, op_repeat_key: Key) -> VimMode {
    let mut mappings = count::mappings() + vim_standard_motions() + vim_standard_objects();
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
            VimKeymap | ctx | {
                // If a key is unhandled, leave operator-pending mode and cancel
                // any pending change
                ctx.keymap.pop_mode("o");
                ctx.keymap.keys_buffer.clear();
                if ctx.state().current_buffer().can_handle_change() {
                    ctx.state_mut().current_buffer_mut().changes().cancel();
                }
                Ok(())
            }
        ))
        .on_after(key_handler!(
            VimKeymap | ?mut ctx | {
                // Also ensure we leave op mode after executing
                if ctx.key.to_digit().is_some() {
                    // ... but let counts accumulate
                    if ctx.keymap.count > 0 {
                        return Ok(());
                    }
                }

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
