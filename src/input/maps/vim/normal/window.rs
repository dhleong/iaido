use crate::input::maps::vim::VimKeymap;
use crate::input::maps::{vim::tree::KeyTreeNode, KeyHandlerContext};
use crate::input::{KeyError, KeymapContext};
use crate::vim_tree;
use crate::{editing::source::BufferSource, editing::FocusDirection, input::maps::KeyResult};

pub fn mappings() -> KeyTreeNode {
    vim_tree! {
        "<c-w>h" => |?mut ctx| focus(ctx, FocusDirection::Left),
        "<c-w>j" => |?mut ctx| focus(ctx, FocusDirection::Down),
        "<c-w>k" => |?mut ctx| focus(ctx, FocusDirection::Up),
        "<c-w>l" => |?mut ctx| focus(ctx, FocusDirection::Right),

        "<c-w><c-h>" => |?mut ctx| focus(ctx, FocusDirection::Left),
        "<c-w><c-j>" => |?mut ctx| focus(ctx, FocusDirection::Down),
        "<c-w><c-k>" => |?mut ctx| focus(ctx, FocusDirection::Up),
        "<c-w><c-l>" => |?mut ctx| focus(ctx, FocusDirection::Right),

        "<c-w><left>" => |?mut ctx| focus(ctx, FocusDirection::Left),
        "<c-w><down>" => |?mut ctx| focus(ctx, FocusDirection::Down),
        "<c-w><up>" => |?mut ctx| focus(ctx, FocusDirection::Up),
        "<c-w><right>" => |?mut ctx| focus(ctx, FocusDirection::Right),
    }
}

fn focus(mut ctx: KeyHandlerContext<VimKeymap>, direction: FocusDirection) -> KeyResult {
    match ctx.state().current_buffer().source() {
        &BufferSource::Cmdline => Err(KeyError::NotPermitted(
            "Invalid in command-line window; <CR> executes, CTRL-C quits".to_string(),
        )),
        _ => {
            ctx.state_mut().current_tab_mut().move_focus(direction);
            Ok(())
        }
    }
}
