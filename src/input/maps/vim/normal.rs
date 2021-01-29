use crate::input::KeymapContext;
use crate::vim_branches;
use crate::vim_tree;
use crate::{
    editing::motion::char::CharMotion,
    editing::motion::linewise::{ToLineEndMotion, ToLineStartMotion},
    editing::motion::Motion,
    key_handler,
};

use super::{
    motions::{vim_linewise_motions, vim_standard_motions},
    tree::KeyTreeNode,
    VimKeymapState, VimMode,
};

fn cmd_mode_access<'a>() -> KeyTreeNode<'a> {
    vim_tree! {
        ":" => |ctx| {
            let prompt = &mut ctx.state_mut().prompt;
            prompt.buffer.append(":".into());
            prompt.handle_content_change();

            ctx.keymap.push_mode(); // TODO
            Ok(())
         },
    }
}

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

    } + cmd_mode_access()
        + vim_standard_motions()
        + vim_linewise_motions();

    VimMode {
        mappings,
        default_handler: None,
    }
}
