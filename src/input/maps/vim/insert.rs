use super::{VimKeymapState, VimMode};
use crate::editing::motion::{
    char::CharMotion,
    word::{is_small_word_boundary, WordMotion},
    Motion,
};
use crate::input::{KeyCode, KeymapContext};
use crate::{key_handler, vim_tree};

pub fn vim_insert_mode<'a>() -> VimMode<'a> {
    let mappings = vim_tree! {
        "<esc>" => |ctx| {
            ctx.state_mut().clear_echo();
            ctx.state_mut().current_window_mut().set_inserting(false);
            CharMotion::Backward(1).apply_cursor(ctx.state_mut());
            Ok(())
         },

        "<a-bs>" => |ctx| {
            let state = ctx.state_mut();
            let motion = WordMotion::backward_until(is_small_word_boundary);
            let end_cursor = motion.destination(state);
            motion.delete_range(state);
            state.current_window_mut().cursor = end_cursor;
            Ok(())
        },
        "<bs>" => |ctx| {
            ctx.state_mut().backspace();
            Ok(())
        },
    };

    VimMode {
        mappings,
        default_handler: Some(key_handler!(
            VimKeymapState | ctx | {
                match ctx.key.code {
                    KeyCode::Char(c) => {
                        ctx.state_mut().type_at_cursor(c);
                    }
                    _ => {} // ignore
                };
                Ok(())
            }
        )),
    }
}
