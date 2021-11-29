mod change;
mod registers;
mod scroll;
pub mod search;
mod window;

use std::rc::Rc;
use tui::style::{Color, Style};
use tui::text::{Span, Spans};

use crate::input::{
    commands::CommandHandlerContext,
    completion::commands::CommandsCompleter,
    keys::KeysParsable,
    maps::{KeyHandlerContext, KeyResult},
    KeyError, KeymapContext, RemapMode, Remappable,
};
use crate::input::{Key, KeyCode};
use crate::{
    editing::buffer::BufHidden,
    editing::gutter::Gutter,
    editing::motion::char::CharMotion,
    editing::motion::linewise::{ToLineEndMotion, ToLineStartMotion},
    editing::motion::{Motion, MotionFlags, MotionRange},
    editing::source::BufferSource,
    editing::text::{EditableLine, TextLine},
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

fn cmdline_to_prompt(
    mut ctx: KeyHandlerContext<VimKeymap>,
    prompt_key: String,
) -> KeyResult<KeyHandlerContext<VimKeymap>> {
    let cmd = if let Some(cmd_spans) = ctx
        .state()
        .current_buffer()
        .checked_get(ctx.state().current_window().cursor.line)
    {
        cmd_spans.to_string()
    } else {
        "".to_string()
    };

    // Release the buffer
    let buffer_id = ctx.state().current_buffer().id();
    ctx.state_mut().delete_buffer(buffer_id);

    // Is this *too* hacky? Just feed each char as a key:
    // Perhaps we should match on prompt_key and invoke eg `handle_command`,
    // `handle_forward_search`, etc. directly...
    ctx = ctx.feed_keys_noremap(prompt_key.into_keys())?;

    let cmd_as_keys: Vec<Key> = cmd.chars().map(|ch| Key::from(KeyCode::Char(ch))).collect();
    ctx = ctx.feed_keys_noremap(cmd_as_keys)?;
    Ok(ctx)
}

fn cancel_cmdline(ctx: KeyHandlerContext<VimKeymap>, prompt_key: String) -> KeyResult {
    cmdline_to_prompt(ctx, prompt_key)?;
    Ok(())
}

fn submit_cmdline(ctx: KeyHandlerContext<VimKeymap>, prompt_key: String) -> KeyResult {
    let ctx = cmdline_to_prompt(ctx, prompt_key)?;
    ctx.feed_keys_noremap("<cr>".into_keys())?;
    Ok(())
}

fn open_cmdline_mode(
    mut ctx: KeyHandlerContext<VimKeymap>,
    history_key: String,
    prompt_key: String,
) -> KeyResult<()> {
    ctx.state_mut().clear_echo();
    let win_id = ctx.state_mut().current_tab_mut().split_bottom();
    let history = ctx.keymap.histories.take(&history_key);

    let buffer = ctx.state_mut().buffers.create_mut();
    let buf_id = buffer.id();
    buffer.set_source(BufferSource::Cmdline);
    buffer.config_mut().bufhidden = BufHidden::Delete;

    let mut count = 0;
    for entry in history.iter().rev() {
        buffer.append_line(entry.to_string());
        count += 1;
    }

    ctx.state_mut().set_current_window_buffer(buf_id);
    ctx.keymap
        .histories
        .replace(history_key.to_string(), history);

    // Bind <cr> to submit the input
    let normal_prompt_key = prompt_key.clone();
    let insert_prompt_key = prompt_key.clone();
    ctx.keymap.buf_remap_keys_fn(
        buf_id,
        RemapMode::VimNormal,
        "<cr>".into_keys(),
        Box::new(move |ctx| submit_cmdline(ctx, normal_prompt_key.to_string())),
    );
    ctx.keymap.buf_remap_keys_fn(
        buf_id,
        RemapMode::VimInsert,
        "<cr>".into_keys(),
        Box::new(move |ctx| submit_cmdline(ctx, insert_prompt_key.to_string())),
    );

    // Bind <ctrl-c> to cancel the mode
    let normal_prompt_key = prompt_key.clone();
    ctx.keymap.buf_remap_keys_fn(
        buf_id,
        RemapMode::VimNormal,
        "<ctrl-c>".into_keys(),
        Box::new(move |ctx| cancel_cmdline(ctx, normal_prompt_key.to_string())),
    );
    ctx.keymap.buf_remap_keys_fn(
        buf_id,
        RemapMode::VimInsert,
        "<ctrl-c>".into_keys(),
        Box::new(move |ctx| cancel_cmdline(ctx, prompt_key.to_string())),
    );

    let win = ctx.state_mut().current_tab_mut().by_id_mut(win_id).unwrap();

    // TODO Resize to cmdwinheight

    let non_line_prefix = vec![Span::styled("~", Style::default().fg(Color::DarkGray))];

    let gutter_prefix = vec![Span::styled(
        history_key,
        Style::default().fg(Color::DarkGray),
    )];

    win.gutter = Some(Gutter {
        width: 1,
        get_content: Box::new(move |line| {
            Spans(match line {
                Some(_) => gutter_prefix.clone(),
                None => non_line_prefix.clone(),
            })
        }),
    });
    win.cursor = (count, 0).into();

    Ok(())
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
            open_cmdline_mode(ctx, ":".to_string(), ":".into())
        },

        "q/" => |?mut ctx| {
            open_cmdline_mode(ctx, "/".to_string(), "/".into())
        },
        "q?" => |?mut ctx| {
            open_cmdline_mode(ctx, "/".to_string(), "?".into())
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
        + vim_standard_motions()
        + vim_linewise_motions();

    VimMode::new("n", mappings).on_default(key_handler!(
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
    }
}
