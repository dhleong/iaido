use crate::input::{commands::CommandHandlerContext, maps::KeyResult, KeyError, KeymapContext};
use crate::vim_tree;
use crate::{
    editing::motion::char::CharMotion,
    editing::motion::linewise::{ToLineEndMotion, ToLineStartMotion},
    editing::motion::Motion,
};

use super::{
    motions::{vim_linewise_motions, vim_standard_motions},
    prompt::VimPromptConfig,
    tree::KeyTreeNode,
    VimKeymapState, VimMode,
};

fn handle_command(mut context: &mut CommandHandlerContext) -> KeyResult {
    let input = context.input.clone();
    if let Some((name, handler)) = context.state_mut().builtin_commands.take(&input) {
        let result = handler(&mut context);
        context
            .state_mut()
            .builtin_commands
            .declare(name, false, handler);
        result
    } else {
        Err(KeyError::NoSuchCommand(input))
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

        "d" => operator |ctx, motion| {
            ctx.state_mut().current_buffer_mut().delete_range(motion);
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

    } + cmd_mode_access()
        + vim_standard_motions()
        + vim_linewise_motions();

    VimMode::new("n", mappings)
}
