use crate::editing::Id;
use crate::input::maps::{vim::VimKeymap, KeyHandlerContext};
use crate::input::KeymapContext;
use crate::vim_tree;
use crate::{
    editing::source::BufferSource, editing::FocusDirection, input::maps::vim::tree::KeyTreeNode,
};

pub fn mappings() -> KeyTreeNode {
    vim_tree! {
        "<ctrl-y>" => |ctx| {
            let win_id = find_scrollable_window(&ctx);
            ctx.state_mut()
                .bufwin_by_id(win_id)
                .unwrap()
                .scroll_lines(1);
            Ok(())
        },
        "<ctrl-e>" => |ctx| {
            let win_id = find_scrollable_window(&ctx);
            ctx.state_mut()
                .bufwin_by_id(win_id)
                .unwrap()
                .scroll_lines(-1);
            Ok(())
        },

        // TODO add 'scroll' setting
        "<ctrl-u>" => |ctx| {
            let win_id = find_scrollable_window(&ctx);
            ctx.state_mut()
                .bufwin_by_id(win_id)
                .unwrap()
                .scroll_by_setting(1, 0);
            Ok(())
        },
        "<ctrl-d>" => |ctx| {
            let win_id = find_scrollable_window(&ctx);
            ctx.state_mut()
                .bufwin_by_id(win_id)
                .unwrap()
                .scroll_by_setting(-1, 0);
            Ok(())
        },

        "<ctrl-b>" => |ctx| {
            let win_id = find_scrollable_window(&ctx);
            ctx.state_mut()
                .bufwin_by_id(win_id)
                .unwrap()
                .scroll_pages(1);
            Ok(())
        },
        "<ctrl-f>" => |ctx| {
            let win_id = find_scrollable_window(&ctx);
            ctx.state_mut()
                .bufwin_by_id(win_id)
                .unwrap()
                .scroll_pages(-1);
            Ok(())
        },
    }
}

fn find_scrollable_window(ctx: &KeyHandlerContext<VimKeymap>) -> Id {
    let win_id = ctx.state().current_window().id;
    if let BufferSource::ConnectionInputForBuffer(_) = ctx.state().current_buffer().source() {
        if let Some(scrollable_win_id) = ctx
            .state()
            .current_tab()
            .next_focus_window(FocusDirection::Up)
        {
            scrollable_win_id
        } else {
            win_id
        }
    } else {
        win_id
    }
}
