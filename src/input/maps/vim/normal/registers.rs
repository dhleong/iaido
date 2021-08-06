use crate::{
    app,
    editing::{
        buffer::CopiedRange,
        motion::{linewise::FullLineMotion, Motion, MotionRange},
        CursorPosition,
    },
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
                paste_after_cursor(ctx.state_mut(), to_paste.into());
            }
            ctx.keymap.reset();
            Ok(())
        },
        "P" => |ctx| {
            if let Some(to_paste) = read_register(&mut ctx) {
                paste_before_cursor(ctx.state_mut(), to_paste.into());
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

fn paste_before_cursor(state: &mut app::State, text: CopiedRange) {
    let single_line_width = single_line_width(&text);

    if single_line_width == 0 {
        state.current_window_mut().cursor.col = 0;
    }

    state.insert_range_at_cursor(text);

    if single_line_width > 0 {
        state.current_window_mut().cursor.col += single_line_width - 1;
    }
}

fn paste_after_cursor(state: &mut app::State, text: CopiedRange) {
    let single_line_width = single_line_width(&text);

    if single_line_width > 0 {
        state.current_window_mut().cursor.col += 1;
    } else {
        let cursor = state.current_window().cursor;
        state.current_window_mut().cursor = CursorPosition {
            line: cursor.line + 1,
            col: 0,
        };
    }
    paste_before_cursor(state, text);
}

fn single_line_width(range: &CopiedRange) -> usize {
    let text = &range.text;
    if text.lines.len() == 1 && !(range.leading_newline || range.trailing_newline) {
        text.lines[0].width()
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use crate::editing::motion::tests::window;
    use indoc::indoc;

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

            // Since we have specified a specific register, 0 should not update
            let zero_contents = state.registers.by_name('0').read();

            assert!(zero_contents.is_none(), "\"0 should still be empty!");
        }

        #[test]
        fn yank_into_zero_register() {
            let ctx = window("Take my |love");
            let (_, mut state) = ctx.feed_vim_for_state("yw");
            let contents = state
                .registers
                .by_name('0')
                .read()
                .expect("Register should have contents set");
            assert_eq!(contents, "love");
        }

        #[test]
        fn yank_append_into_register() {
            let ctx = window("Take |my love");
            let (_, mut state) = ctx.feed_vim_for_state("\"ayww\"Ayw");
            let contents = state
                .registers
                .by_name('a')
                .read()
                .expect("Register should have contents set");
            assert_eq!(contents, "my love");
        }
    }

    #[cfg(test)]
    mod p {
        use super::*;

        #[test]
        fn paste_partial_line_after_cursor() {
            let ctx = window("Take my |love");
            ctx.feed_vim("ywp").assert_visual_match("Take my llov|eove");
        }

        #[test]
        fn paste_full_line_after_cursor() {
            let ctx = window(indoc! {"
                ~
                Take my |love
            "});
            ctx.feed_vim("Yp").assert_visual_match(indoc! {"
                Take my love
                |Take my love
            "});
        }
    }

    #[cfg(test)]
    mod capital_p {
        use super::*;

        #[test]
        fn paste_partial_line_before_cursor() {
            let ctx = window("Take my |love");
            ctx.feed_vim("ywP").assert_visual_match("Take my lov|elove");
        }

        #[test]
        fn paste_full_line_before_cursor() {
            let ctx = window(indoc! {"
                ~
                Take my |love
            "});
            ctx.feed_vim("YP").assert_visual_match(indoc! {"
                |Take my love
                Take my love
            "});
        }
    }
}
