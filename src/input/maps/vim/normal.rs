use crate::vim_branches;
use crate::vim_tree;
use crate::{
    editing::motion::char::CharMotion,
    editing::motion::linewise::{ToLineEndMotion, ToLineStartMotion},
    editing::motion::word::{is_big_word_boundary, is_small_word_boundary, WordMotion},
    key_handler,
};
use crate::{
    editing::text::TextLines,
    input::{keys::KeysParsable, KeymapContext},
};

use super::{KeyTreeNode, VimKeymapState};

pub fn vim_normal_mode<'a>() -> KeyTreeNode<'a> {
    vim_tree! {
        "<cr>" => |ctx| {
            ctx.state_mut().running = false;
            Ok(())
        },

        "a" => |ctx| {
            ctx.state_mut().current_buffer_mut().append(TextLines::raw("append"));
            Ok(())
         },

        "d" => operator |ctx, motion| {
            ctx.state_mut().current_buffer_mut().delete_range(motion);
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
    }
}
