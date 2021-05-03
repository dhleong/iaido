use crate::{
    editing::motion::{linewise::FullLineMotion, Motion, MotionRange},
    input::{
        maps::{
            vim::{tree::KeyTreeNode, VimKeymap},
            KeyHandlerContext, KeyResult,
        },
        Key, KeyCode, KeyError, KeySource, KeymapContext,
    },
    vim_tree,
};

pub fn mappings() -> KeyTreeNode {
    vim_tree! {
        "\"" => |ctx| {
            if let Some(key) = ctx.next_key()? {
                if let Key { code: KeyCode::Char(ch), .. } = key {
                    ctx.keymap.keys_buffer.push(key);
                    ctx.keymap.selected_register = Some(ch);
                    return Ok(());
                }
            }
            ctx.keymap.reset();
            Ok(())
         },

        "p" => |ctx| {
            let register = ctx.keymap.selected_register;
            let reg_contents = ctx.state_mut()
                .registers
                .by_optional_name(register)
                .read()
                .and_then(|s| Some(s.to_string()));
            if let Some(to_paste) = reg_contents {
                ctx.state_mut().insert_at_cursor(to_paste.into());
            }
            ctx.keymap.reset();
            Ok(())
         },

        "y" => operator |ctx, motion| {
            yank(ctx, motion)
        },
        "Y" => |ctx| {
            let range = FullLineMotion.range(ctx.state());
            yank(ctx, range)
        },
    }
}

fn yank(mut ctx: KeyHandlerContext<VimKeymap>, range: MotionRange) -> KeyResult {
    let register = ctx.keymap.selected_register;
    let yanked = ctx.state_mut().current_buffer_mut().get_range(range);
    ctx.state_mut().registers.handle_yanked(register, yanked);
    ctx.keymap.reset();
    Ok(())
}
