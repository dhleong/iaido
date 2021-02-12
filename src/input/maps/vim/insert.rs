use super::{tree::KeyTreeNode, VimKeymapState, VimMode};
use crate::input::{
    completion::{state::CompletionState, Completer, CompletionContext},
    KeyCode, KeymapContext,
};
use crate::{
    app,
    editing::motion::{
        char::CharMotion,
        word::{is_small_word_boundary, WordMotion},
        Motion,
    },
    input::completion::commands::CommandsCompleter,
};
use crate::{key_handler, vim_tree};

pub fn vim_insert_mappings() -> KeyTreeNode {
    vim_tree! {
        "<a-bs>" => |ctx| {
            let state = ctx.state_mut();
            let motion = WordMotion::backward_until(is_small_word_boundary);
            let end_cursor = motion.destination(state);
            motion.delete_range(state);
            state.current_window_mut().cursor = end_cursor;
            Ok(())
        },
        "<bs>" => |ctx| {
            ctx.state_mut().backspace();
            Ok(())
        },
        "<tab>" => |ctx| {
            if let Some(ref mut current_state) = ctx.state_mut().current_window_mut().completion_state {
                // apply next completion
                let prev = current_state.take_current();
                let next = current_state.advance();
                ctx.state_mut().current_buffer_mut().apply_completion(prev.as_ref(), next.as_ref());
                ctx.state_mut().current_window_mut().completion_state.as_mut().unwrap().push_history(prev, next);
            } else {
                // TODO get the completer to use from context/window/buffer, probably
                let c = CommandsCompleter;
                let context: CompletionContext<app::State> = ctx.state_mut().into();
                let mut state = CompletionState::new(Box::new(c.suggest(&context)));

                // apply initial suggestion
                let next = state.take_current();
                ctx.state_mut().current_buffer_mut().apply_completion(None, next.as_ref());
                state.push_history(None, next);

                ctx.state_mut().current_window_mut().completion_state = Some(state);
            }
            Ok(())
         },
    }
}

pub fn vim_insert_mode() -> VimMode {
    let mappings = vim_tree! {
        "<esc>" => |ctx| {
            ctx.state_mut().clear_echo();
            ctx.state_mut().current_window_mut().set_inserting(false);
            CharMotion::Backward(1).apply_cursor(ctx.state_mut());
            Ok(())
         },
    } + vim_insert_mappings();

    VimMode::new("i", mappings).on_default(key_handler!(
        VimKeymapState | ctx | {
            match ctx.key.code {
                KeyCode::Char(c) => {
                    ctx.state_mut().type_at_cursor(c);
                }
                _ => {} // ignore
            };
            Ok(())
        }
    ))
}
