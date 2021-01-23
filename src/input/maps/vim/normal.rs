use crate::key_handler;
use crate::vim_branches;
use crate::vim_tree;
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

        "d" => |ctx| {
            ctx.state.pending_motion_action_key = Some('d'.into());
            Ok(())
        }
    }
}
