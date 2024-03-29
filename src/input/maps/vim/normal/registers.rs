use crate::{
    app,
    editing::{
        buffer::CopiedRange,
        motion::{linewise::FullLineMotion, Motion, MotionRange},
        CursorPosition,
    },
    input::{
        maps::{
            vim::{tree::KeyTreeNode, util::verify_can_edit, VimKeymap},
            KeyHandlerContext, KeyResult,
        },
        Key, KeyCode, KeySource, KeymapContext,
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
            verify_can_edit(&ctx)?;
            if let Some(to_paste) = read_register(&mut ctx) {
                paste_after_cursor(ctx.state_mut(), to_paste.into());
            }
            ctx.keymap.reset();
            Ok(())
        },
        "P" => |ctx| {
            verify_can_edit(&ctx)?;
            if let Some(to_paste) = read_register(&mut ctx) {
                paste_before_cursor(ctx.state_mut(), to_paste.into());
            }
            ctx.keymap.reset();
            Ok(())
        },

        "y" => operator ?change |ctx, motion| {
            let result = yank(&mut ctx, motion);
            ctx.state_mut().current_window_mut().cursor =
                ctx.state().current_window().clamp_cursor(ctx.state().current_buffer(), motion.0);
            result
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

fn paste_before_cursor(state: &mut app::State, mut text: CopiedRange) {
    let single_line_width = single_line_width(&text);

    if single_line_width == 0 {
        state.current_window_mut().cursor.col = 0;
        text.leading_newline = true;
    }

    state.insert_range_at_cursor(text);

    if single_line_width > 0 {
        state.current_window_mut().cursor.col += single_line_width - 1;
    }
}

fn paste_after_cursor(state: &mut app::State, mut text: CopiedRange) {
    let single_line_width = single_line_width(&text);

    if single_line_width > 0 {
        let win = state.current_window();
        let buf_id = win.buffer;
        let mut cursor = win.cursor;
        cursor.col += 1;

        let buf = state.buffers.by_id(buf_id).expect("Expected a buffer");
        cursor = win.clamp_cursor(buf, cursor);

        state.current_window_mut().cursor = cursor;
    } else {
        let cursor = state.current_window().cursor;
        state.current_window_mut().cursor = CursorPosition {
            line: cursor.line + 1,
            col: 0,
        };
        text.leading_newline = true;
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
        fn yank_in_empty() {
            // Sanity check:
            let ctx = window("");
            ctx.feed_vim("yw").assert_visual_match("");
        }

        #[test]
        fn yank_in_read_only() {
            let mut ctx = window("Take my |love");
            ctx.buffer
                .set_source(crate::editing::source::BufferSource::Log);
            let (_, mut state) = ctx.feed_vim_for_state("\"ayw");
            let contents = state
                .registers
                .by_name('a')
                .read()
                .expect("Register should have contents set");
            assert_eq!(contents, "love");
        }

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
        fn yank_forwards_does_not_move_cursor() {
            let ctx = window("Take |my love");
            ctx.feed_vim("yw").assert_visual_match(indoc! {"
                Take |my love
            "});
        }

        #[test]
        fn yank_backwards_moves_cursor() {
            let ctx = window("Take my |love");
            ctx.feed_vim("yb").assert_visual_match(indoc! {"
                Take |my love
            "});
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

        #[test]
        fn paste_line_from_clipboard_after_cursor() {
            let ctx = window(indoc! {"
                ~
                Take my |love
            "});

            let mut state = crate::app::State::default();
            state
                .registers
                .by_name('a')
                .write("Take my land\n".to_string());

            let (mut ctx, _) = ctx.feed_vim_with_state(state, "\"ap");
            ctx.assert_visual_match(indoc! {"
                Take my love
                |Take my land
            "});
        }

        #[test]
        fn paste_single_line_after_cursor_when_empty() {
            let ctx = window(indoc! {"
                ~
            "});

            let mut state = crate::app::State::default();
            state
                .registers
                .by_name('a')
                .write("Take my love".to_string());

            let (mut ctx, _) = ctx.feed_vim_with_state(state, "\"ap");
            ctx.assert_visual_match(indoc! {"
                Take my lov|e
            "});
        }

        #[test]
        fn paste_line_from_clipboard_after_cursor_when_empty() {
            let ctx = window(indoc! {"
                ~
                ~
            "});

            let mut state = crate::app::State::default();
            state
                .registers
                .by_name('a')
                .write("Take my love\n".to_string());

            let (mut ctx, _) = ctx.feed_vim_with_state(state, "\"ap");
            ctx.assert_visual_match(indoc! {"
                _
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

        #[test]
        fn paste_line_from_clipboard_before_cursor() {
            let ctx = window(indoc! {"
                ~
                Take my |love
            "});

            let mut state = crate::app::State::default();
            state
                .registers
                .by_name('a')
                .write("Take my land\n".to_string());

            let (mut ctx, _) = ctx.feed_vim_with_state(state, "\"aP");
            ctx.assert_visual_match(indoc! {"
                |Take my land
                Take my love
            "});
        }
    }
}
