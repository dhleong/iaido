mod change;
pub mod count;
mod registers;
mod scroll;
pub mod search;
mod window;

use std::rc::Rc;

use crate::{
    editing::motion::char::CharMotion,
    editing::motion::linewise::{ToLineEndMotion, ToLineStartMotion},
    editing::motion::{Motion, MotionFlags, MotionRange},
    editing::text::TextLine,
};
use crate::{
    editing::source::BufferSource,
    input::{
        commands::CommandHandlerContext,
        completion::commands::CommandsCompleter,
        maps::vim::cmdline::{self, CmdlineSink},
        maps::{KeyHandlerContext, KeyResult},
        KeyError, KeymapContext,
    },
};
use crate::{key_handler, vim_tree};

use super::{
    motions::{vim_linewise_motions, vim_standard_motions},
    prompt::VimPromptConfig,
    tree::KeyTreeNode,
    VimKeymap, VimMode,
};

fn handle_command(mut context: &mut CommandHandlerContext) -> KeyResult {
    if let Some(command) = context.command().and_then(|s| Some(s.to_string())) {
        if let Some((name, spec)) = context.state_mut().builtin_commands.take(&command) {
            let result = (spec.handler)(&mut context);
            context.state_mut().builtin_commands.insert(name, spec);
            result
        } else {
            Err(KeyError::NoSuchCommand(command))
        }
    } else {
        // no command; nop is okay
        Ok(())
    }
}

fn cmd_mode_access() -> KeyTreeNode {
    vim_tree! {
        ":" => |ctx| {
            ctx.state_mut().clear_echo();
            ctx.state_mut().prompt.activate(":".into());

            ctx.keymap.push_mode(VimPromptConfig{
                prompt: ":".into(),
                history_key: ":".into(),
                handler: Box::new(handle_command),
                completer: Some(Rc::new(CommandsCompleter)),
            }.into());
            Ok(())
        },

        "q:" => |?mut ctx| {
            cmdline::open(ctx, ":".to_string(), CmdlineSink::SubmitPrompt(":"))
        },

        "q/" => |?mut ctx| {
            cmdline::open(ctx, "/".to_string(), CmdlineSink::SubmitPrompt("/"))
        },
        "q?" => |?mut ctx| {
            cmdline::open(ctx, "/".to_string(), CmdlineSink::SubmitPrompt("?"))
        },

        "qi" => |ctx| {
            let buffer_id = ctx.state().current_buffer().id();
            let conn_buffer_id = match ctx.state().current_buffer().source() {
                &BufferSource::ConnectionInputForBuffer(conn_buff_id) => conn_buff_id,
                _ => buffer_id,
            };

            let mut conns = ctx.state_mut().connections.take().expect("Connections obj missing");
            let result = if let Some(conn) = conns.by_buffer_id(buffer_id) {
                conn.with_engine(|engine| {
                    let history = &engine.history;

                    cmdline::open_from_history(&mut ctx, history, "!".to_string(), CmdlineSink::ConnectionBuffer(conn_buffer_id))
                })

            } else {
                Err(KeyError::IO(std::io::ErrorKind::NotConnected.into()))
            };

            ctx.state_mut().connections = Some(conns);

            result
        },
    }
}

pub fn vim_normal_mode() -> VimMode {
    let mappings = vim_tree! {
        "a" => change |ctx| {
            ctx.state_mut().current_window_mut().set_inserting(true);
            CharMotion::Forward(1).apply_cursor(ctx.state_mut());
            Ok(())
        },
        "A" => change |ctx| {
            ctx.state_mut().current_window_mut().set_inserting(true);
            ToLineEndMotion.apply_cursor(ctx.state_mut());
            Ok(())
        },

        "c" => operator |ctx, motion| {
            delete_range(&mut ctx, motion);

            let MotionRange(start, _, flags) = motion;
            ctx.state_mut().current_window_mut().cursor = start;

            if flags.contains(MotionFlags::LINEWISE) {
                // insert a blank line at the cursor
                ctx.state_mut().current_buffer_mut().insert_lines(start.line, TextLine::from("").into());
            }

            // NOTE: after leaving, we would normally finish the change
            // BUT we want any text edited as part of insert to be
            // included, so we "start" a new change that will take over
            // ownership of the change for all keys in this insert mode
            ctx.state_mut().current_bufwin().begin_insert_change("");
            Ok(())
        },
        "C" => change |ctx| {
            ctx.state_mut().current_window_mut().set_inserting(true);
            let range = ToLineEndMotion.range(ctx.state());
            delete_range(&mut ctx, range);
            Ok(())
        },

        "d" => operator |ctx, motion| {
            delete_range(&mut ctx, motion);
            ctx.state_mut().current_window_mut().cursor =
                ctx.state().current_window().clamp_cursor(ctx.state().current_buffer(), motion.0);
            Ok(())
        },
        "D" => change |ctx| {
            let range = ToLineEndMotion.range(ctx.state());
            delete_range(&mut ctx, range);
            ctx.state_mut().current_buffer_mut().end_change();
            Ok(())
        },

        "i" => change |ctx| {
            ctx.state_mut().clear_echo();
            ctx.state_mut().current_window_mut().set_inserting(true);
            Ok(())
        },
        "I" => change |ctx| {
            ctx.state_mut().clear_echo();
            ctx.state_mut().current_window_mut().set_inserting(true);
            ToLineStartMotion.apply_cursor(ctx.state_mut());
            Ok(())
        },

        "x" => change |ctx| {
            delete_with_motion(ctx, CharMotion::Forward(1))
        },
        "X" => change |ctx| {
            delete_with_motion(ctx, CharMotion::Backward(1))
        },
    } + cmd_mode_access()
        + change::mappings()
        + registers::mappings()
        + scroll::mappings()
        + search::mappings()
        + window::mappings()
        + count::mappings()
        + vim_standard_motions()
        + vim_linewise_motions();

    VimMode::new("n", mappings)
        .with_shows_keys(true)
        .on_default(key_handler!(
            VimKeymap | ?mut ctx | {
                ctx.keymap.reset();
                Ok(())
            }
        ))
}

fn delete_with_motion<M: Motion>(mut ctx: KeyHandlerContext<VimKeymap>, motion: M) -> KeyResult {
    let range = motion.range(ctx.state());
    delete_range(&mut ctx, range);
    ctx.state_mut().current_window_mut().cursor = ctx
        .state()
        .current_window()
        .clamp_cursor(ctx.state().current_buffer(), range.0);
    ctx.state_mut().current_buffer_mut().end_change();
    Ok(())
}

fn delete_range(ctx: &mut KeyHandlerContext<VimKeymap>, range: MotionRange) {
    let register = ctx.keymap.selected_register;
    let yanked = ctx.state_mut().current_buffer_mut().delete_range(range);
    ctx.state_mut().registers.handle_deleted(register, yanked);
}

#[cfg(test)]
mod tests {
    use crate::editing::motion::tests::window;
    use crate::input::keys::KeysParsable;
    use indoc::indoc;

    #[cfg(test)]
    mod c {
        use super::*;

        #[test]
        fn cc_retains_line() {
            let ctx = window(indoc! {"
                Take my love
                |Take my land
                Take me where
            "});
            ctx.feed_vim("cc").assert_visual_match(indoc! {"
                Take my love
                |
                Take me where
            "});
        }

        #[test]
        fn ck_retains_line() {
            let ctx = window(indoc! {"
                Take my love
                |Take my land
                Take me where
            "});
            ctx.feed_vim("ck").assert_visual_match(indoc! {"
                ~
                |
                Take me where
            "});
        }

        #[test]
        fn text_object() {
            let ctx = window(indoc! {"
                Take my love
                Take m|y land
                Take me where
            "});
            ctx.feed_vim("caw").assert_visual_match(indoc! {"
                Take my love
                Take |land
                Take me where
            "});
        }

        #[test]
        fn with_motion_adds_keys_to_change() {
            let mut ctx = window(indoc! {"
                Take my love
                |Take my land
            "});
            ctx = ctx.feed_vim("cwFarm <esc>");
            ctx.assert_visual_match(indoc! {"
                Take my love
                Farm| my land
            "});
            let change = ctx.buffer.changes().take_last().unwrap();
            assert_eq!(change.keys, "cwFarm <esc>".into_keys());
        }
    }

    #[cfg(test)]
    mod d {
        use super::*;

        #[test]
        fn d_in_empty() {
            // Sanity check:
            let ctx = window("");
            ctx.feed_vim("dw").assert_visual_match("");
        }

        #[test]
        fn dd() {
            let ctx = window(indoc! {"
                Take my love
                |Take my land
                Take me where
            "});
            ctx.feed_vim("dd").assert_visual_match(indoc! {"
                ~
                Take my love
                |Take me where
            "});
        }

        #[test]
        fn dh() {
            let ctx = window(indoc! {"
                Take my l|and
            "});
            ctx.feed_vim("dh").assert_visual_match(indoc! {"
                Take my |and
            "});
        }

        #[test]
        fn dl() {
            let ctx = window(indoc! {"
                Take my |land
            "});
            ctx.feed_vim("dl").assert_visual_match(indoc! {"
                Take my |and
            "});
        }

        #[test]
        fn follows_exclusive_line_cross_exception() {
            // see :help exclusive in vim
            let ctx = window(indoc! {"
                Take my |love
                Take my land
            "});
            ctx.feed_vim("dw").assert_visual_match(indoc! {"
                Take my| 
                Take my land
            "});
        }

        #[test]
        fn with_motion_adds_keys_to_change() {
            let mut ctx = window(indoc! {"
                Take my love
                |Take my land
            "});
            ctx = ctx.feed_vim("dw");
            ctx.assert_visual_match(indoc! {"
                Take my love
                |my land
            "});
            let change = ctx.buffer.changes().take_last().unwrap();
            assert_eq!(change.keys, "dw".into_keys());
        }

        #[test]
        fn dw_deletes_empty_line() {
            let ctx = window(indoc! {"
                Take my love
                |
                Take my land
            "});
            ctx.feed_vim("dw").assert_visual_match(indoc! {"
                ~
                Take my love
                |Take my land
            "});
        }
    }

    #[cfg(test)]
    mod capital_d {
        use super::*;

        #[test]
        fn deletes_through_end_of_line() {
            let ctx = window(indoc! {"
                Take my love
                Take |my land
                Take me where
            "});
            ctx.feed_vim("D").assert_visual_match(indoc! {"
                Take my love
                Take |
                Take me where
            "});
        }

        #[test]
        fn retains_empty_line() {
            let ctx = window(indoc! {"
                Take my love
                |Take my land
                Take me where
            "});
            ctx.feed_vim("D").assert_visual_match(indoc! {"
                Take my love
                |
                Take me where
            "});
        }
    }

    #[cfg(test)]
    mod f {
        use super::*;

        #[test]
        fn find_char() {
            let ctx = window(indoc! {"
                |Take my land
            "});
            ctx.feed_vim("fl").assert_visual_match(indoc! {"
                Take my |land
            "});
        }

        #[test]
        fn find_non_matching_does_not_move() {
            let ctx = window(indoc! {"
                |Take my land
            "});
            ctx.feed_vim("fz").assert_visual_match(indoc! {"
                |Take my land
            "});
        }

        #[test]
        fn delete_with_find() {
            let ctx = window(indoc! {"
                |Take my land
            "});
            ctx.feed_vim("dfl").assert_visual_match(indoc! {"
                |and
            "});
        }
    }

    #[cfg(test)]
    mod capital_f {
        use super::*;

        #[test]
        fn find_char() {
            let ctx = window(indoc! {"
                Take my |land
            "});
            ctx.feed_vim("Fe").assert_visual_match(indoc! {"
                Tak|e my land
            "});
        }

        #[test]
        fn find_non_matching_does_not_move() {
            let ctx = window(indoc! {"
                Take my |land
            "});
            ctx.feed_vim("Fz").assert_visual_match(indoc! {"
                Take my |land
            "});
        }

        #[test]
        fn delete_with_find() {
            let ctx = window(indoc! {"
                Take my |land
            "});
            ctx.feed_vim("dFm").assert_visual_match(indoc! {"
                Take |land
            "});
        }

        #[test]
        fn delete_with_non_matching() {
            let ctx = window(indoc! {"
                Take my |land
            "});
            ctx.feed_vim("dFz").assert_visual_match(indoc! {"
                Take my |land
            "});
        }

        #[test]
        fn delete_with_count() {
            let ctx = window(indoc! {"
                |Take my land
            "});
            ctx.feed_vim("4dl").assert_visual_match(indoc! {"
                | my land
            "});
        }

        #[test]
        fn delete_with_motion_count() {
            let ctx = window(indoc! {"
                |Take my land
            "});
            ctx.feed_vim("d4l").assert_visual_match(indoc! {"
                | my land
            "});
        }

        #[test]
        fn delete_with_both_count_types() {
            let ctx = window(indoc! {"
                Take my |land
            "});
            ctx.feed_vim("2d2h").assert_visual_match(indoc! {"
                Take|land
            "});
        }
    }

    #[cfg(test)]
    mod motions {
        use super::*;

        #[test]
        fn zero_to_line_start() {
            let ctx = window(indoc! {"
                Take my |land
            "});
            ctx.feed_vim("0").assert_visual_match(indoc! {"
                |Take my land
            "});
        }

        #[test]
        fn count_with_zero() {
            let ctx = window(indoc! {"
                Take my love, Take my |land
            "});
            ctx.feed_vim("10h").assert_visual_match(indoc! {"
                Take my love|, Take my land
            "});
        }
    }
}
