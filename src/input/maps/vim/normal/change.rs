use crate::input::maps::vim::tree::KeyTreeNode;
use crate::input::maps::vim::VimKeymap;
use crate::input::maps::KeyError;
use crate::input::KeymapContext;
use crate::vim_tree;

pub fn mappings() -> KeyTreeNode {
    vim_tree! {
        "u" => |ctx| {
            if ctx.state().current_buffer().is_read_only() {
                return Err(KeyError::ReadOnlyBuffer);
            }

            ctx.state_mut().request_redraw();
            if ctx.state_mut().current_bufwin().undo() {
                // TODO more info?
                ctx.state_mut().echo_str("1 change; older");
            } else {
                ctx.state_mut().echo_str("Already at oldest change");
            }
            Ok(())
        },
        "<ctrl-r>" => |ctx| {
            if ctx.state().current_buffer().is_read_only() {
                return Err(KeyError::ReadOnlyBuffer);
            }

            ctx.state_mut().request_redraw();
            if ctx.state_mut().current_bufwin().redo() {
                // TODO more info?
                ctx.state_mut().echo_str("1 change; newer");
            } else {
                ctx.state_mut().echo_str("Already at newest change");
            }
            Ok(())
        },

        "." => |ctx| {
            if ctx.state().current_buffer().is_read_only() {
                return Err(KeyError::ReadOnlyBuffer);
            }

            ctx.state_mut().request_redraw();
            if let Some(last) = ctx.state_mut().current_buffer_mut().changes().take_last() {
                let keys = last.keys.clone();
                ctx.state_mut().current_buffer_mut().changes().push(last);
                ctx.feed_keys(keys)?;
            }
            Ok(())
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::editing::motion::tests::window;
    use indoc::indoc;

    #[cfg(test)]
    mod u {
        use super::*;

        #[test]
        fn undo_empty() {
            let mut ctx = window(indoc! {"
                Take my love
                |Take my land
                Take me where
            "});
            ctx.buffer.changes().clear();
            ctx.feed_vim("u").assert_visual_match(indoc! {"
                Take my love
                |Take my land
                Take me where
            "});
        }

        #[test]
        fn undo_line_appends() {
            let mut ctx = window("");
            ctx.window.size = (20, 2).into();
            ctx.buffer.append("Take my love".into());
            ctx.buffer.append("Take my land".into());
            ctx.buffer.append("Take me where".into());
            ctx.window.cursor = (2, 12).into();

            ctx = ctx.feed_vim("u");
            ctx.assert_visual_match(indoc! {"
                Take my love
                |Take my land
            "});

            ctx = ctx.feed_vim("u");
            ctx.assert_visual_match(indoc! {"
                ~
                |Take my love
            "});

            ctx = ctx.feed_vim("u");
            ctx.render_at_own_size();

            ctx.feed_vim("u").render_at_own_size();
        }

        #[test]
        fn undo_restores_cursor() {
            let ctx = window(indoc! {"
                Take my love
                |Take my land
                Take me where
            "});
            ctx.feed_vim("Dku").assert_visual_match(indoc! {"
                Take my love
                |Take my land
                Take me where
            "});
        }

        #[test]
        fn undo_redone() {
            let mut ctx = window(indoc! {"
                |Take my love
            "});
            ctx.buffer.append("Take my land".into());
            ctx.window.size = (20, 2).into();
            ctx.assert_visual_match(indoc! {"
                |Take my love
                Take my land
            "});

            ctx.feed_vim("u<ctrl-r>u").assert_visual_match(indoc! {"
                ~
                |Take my love
            "});
        }
    }

    #[cfg(test)]
    mod repeat {
        use super::*;

        #[test]
        fn repeat_delete_with_motion() {
            let mut ctx = window(indoc! {"
                |Take my love
            "});
            ctx.assert_visual_match(indoc! {"
                |Take my love
            "});

            ctx.feed_vim("dw..").assert_visual_match(indoc! {"
                |
            "});
        }

        #[test]
        fn repeat_change_can_seem_idempotent() {
            let mut ctx = window(indoc! {"
                Take my |love
            "});
            ctx.assert_visual_match(indoc! {"
                Take my |love
            "});

            ctx.feed_vim("Cland<esc>b.").assert_visual_match(indoc! {"
                ake my lan|d
            "});
        }
    }
}
