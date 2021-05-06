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
            if let Some(to_paste) = read_register(&mut ctx) {
                ctx.state_mut().insert_at_cursor(to_paste.into());
            }
            ctx.keymap.reset();
            Ok(())
        },
        "P" => |ctx| {
            if let Some(to_paste) = read_register(&mut ctx) {
                ctx.state_mut().insert_at_cursor(to_paste.into());
            }
            ctx.keymap.reset();
            Ok(())
        },

        "y" => operator |ctx, motion| {
            yank(&mut ctx, motion)
        },
        "Y" => |ctx| {
            let range = FullLineMotion.range(ctx.state());
            yank(&mut ctx, range)
        },
    }
}

fn read_register(ctx: &mut KeyHandlerContext<VimKeymap>) -> Option<String> {
    let register = ctx.keymap.selected_register;
    ctx.state_mut()
        .registers
        .by_optional_name(register)
        .read()
        .and_then(|s| Some(s.to_string()))
}

fn yank(ctx: &mut KeyHandlerContext<VimKeymap>, range: MotionRange) -> KeyResult {
    let register = ctx.keymap.selected_register;
    let yanked = ctx.state_mut().current_buffer_mut().get_range(range);
    ctx.state_mut().registers.handle_yanked(register, yanked);
    ctx.keymap.reset();
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::editing::motion::tests::window;

    #[cfg(test)]
    mod y {
        use super::*;

        #[test]
        fn yank_into_register() {
            let ctx = window("Take my |love");
            let (_, mut state) = ctx.feed_vim_for_state("\"ayw");
            let contents = state
                .registers
                .by_name('a')
                .read()
                .expect("Register should have contents set");
            assert_eq!(contents, "love");
        }
    }

    #[cfg(test)]
    mod p {
        use super::*;

        #[test]
        fn paste_single_line_after_cursor() {
            let ctx = window("Take my |love");
            ctx.feed_vim("ywp").assert_visual_match("Take my llov|eove");
        }
    }

    #[cfg(test)]
    mod capital_p {
        use super::*;

        #[test]
        fn paste_single_line_before_cursor() {
            let ctx = window("Take my |love");
            ctx.feed_vim("ywP").assert_visual_match("Take my lov|elove");
        }
    }
}
