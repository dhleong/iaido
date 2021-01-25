use crate::input::KeymapContext;
use crate::vim_branches;
use crate::vim_tree;
use crate::{
    editing::motion::char::CharMotion,
    editing::motion::linewise::{ToLineEndMotion, ToLineStartMotion},
    editing::motion::word::{is_big_word_boundary, is_small_word_boundary, WordMotion},
    editing::motion::Motion,
    key_handler,
};

use super::{VimKeymapState, VimMode};

pub fn vim_normal_mode<'a>() -> VimMode<'a> {
    let mappings = vim_tree! {
        "<cr>" => |ctx| {
            ctx.state_mut().running = false;
            Ok(())
        },

        "a" => |ctx| {
            ctx.state_mut().current_window_mut().set_inserting(true);
            CharMotion::Forward(1).apply_cursor(ctx.state_mut());
            Ok(())
        },
        "A" => |ctx| {
            ctx.state_mut().current_window_mut().set_inserting(true);
            ToLineEndMotion.apply_cursor(ctx.state_mut());
            Ok(())
        },

        "d" => operator |ctx, motion| {
            ctx.state_mut().current_buffer_mut().delete_range(motion);
            Ok(())
        },

        "i" => |ctx| {
            ctx.state_mut().current_window_mut().set_inserting(true);
            Ok(())
        },
        "I" => |ctx| {
            ctx.state_mut().current_window_mut().set_inserting(true);
            ToLineStartMotion.apply_cursor(ctx.state_mut());
            Ok(())
        },

        // NOTE: should we define motions separately and combine?
        "b" => motion { WordMotion::backward_until(is_small_word_boundary) },
        "B" => motion { WordMotion::backward_until(is_big_word_boundary) },
        "w" => motion { WordMotion::forward_until(is_small_word_boundary) },
        "W" => motion { WordMotion::forward_until(is_big_word_boundary) },

        "h" => motion { CharMotion::Backward(1) },
        "l" => motion { CharMotion::Forward(1) },

        "0" => motion { ToLineStartMotion },
        "$" => motion { ToLineEndMotion },
    };

    VimMode {
        mappings,
        default_handler: None,
    }
}
