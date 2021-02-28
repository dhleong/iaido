mod window;

use crate::editing::text::TextLine;
use crate::input::{commands::CommandHandlerContext, maps::KeyResult, KeyError, KeymapContext};
use crate::{
    editing::motion::char::CharMotion,
    editing::motion::linewise::{ToLineEndMotion, ToLineStartMotion},
    editing::motion::{Motion, MotionFlags, MotionRange},
};
use crate::{key_handler, vim_tree};

use super::{
    motions::{vim_linewise_motions, vim_standard_motions},
    prompt::VimPromptConfig,
    tree::KeyTreeNode,
    VimKeymapState, VimMode,
};

fn handle_command(mut context: &mut CommandHandlerContext) -> KeyResult {
    if let Some(command) = context.command().and_then(|s| Some(s.to_string())) {
        if let Some((name, handler)) = context.state_mut().builtin_commands.take(&command) {
            let result = handler(&mut context);
            context
                .state_mut()
                .builtin_commands
                .declare(name, false, handler);
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
                handler: Box::new(handle_command),
                // TODO autocomplete
            }.into());
            Ok(())
         },
    }
}

pub fn vim_normal_mode() -> VimMode {
    let mappings = vim_tree! {
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

        "c" => operator |ctx, motion| {
            ctx.state_mut().current_buffer_mut().delete_range(motion);

            let MotionRange(start, _, flags) = motion;
            ctx.state_mut().current_window_mut().cursor = start;

            if flags.contains(MotionFlags::LINEWISE) {
                // insert a blank line at the cursor
                ctx.state_mut().current_buffer_mut().insert_lines(start.line, TextLine::from("").into());
            }

            ctx.state_mut().current_window_mut().set_inserting(true);
            Ok(())
        },
        "C" => |ctx| {
            let range = ToLineEndMotion.range(ctx.state());
            ctx.state_mut().current_buffer_mut().delete_range(range);
            ctx.state_mut().current_window_mut().set_inserting(true);
            Ok(())
        },

        "d" => operator |ctx, motion| {
            ctx.state_mut().current_buffer_mut().delete_range(motion);
            Ok(())
        },
        "D" => |ctx| {
            let range = ToLineEndMotion.range(ctx.state());
            ctx.state_mut().current_buffer_mut().delete_range(range);
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

        "u" => |ctx| {
            if ctx.state_mut().current_bufwin().undo() {
                // TODO more info?
                ctx.state_mut().echo_str("1 change; older");
            } else {
                ctx.state_mut().echo_str("Already at oldest change");
            }
            Ok(())
        },
        "<ctrl-r>" => |ctx| {
            if ctx.state_mut().current_bufwin().redo() {
                // TODO more info?
                ctx.state_mut().echo_str("1 change; newer");
            } else {
                ctx.state_mut().echo_str("Already at newest change");
            }
            Ok(())
        },

        "." => |ctx| {
            if let Some(last) = ctx.state_mut().current_buffer_mut().changes().take_last() {
                // TODO enqueue keys
                ctx.state_mut().current_buffer_mut().changes().push(last);
            }
            Ok(())
        },

    } + cmd_mode_access()
        + window::mappings()
        + vim_standard_motions()
        + vim_linewise_motions();

    VimMode::new("n", mappings).on_default(key_handler!(
        VimKeymapState | ?mut ctx | {
            ctx.keymap.reset();
            Ok(())
        }
    ))
}

#[cfg(test)]
mod tests {
    use crate::editing::motion::tests::window;
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
    mod u {
        use super::*;

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
    }
}
