use super::{tree::KeyTreeNode, VimKeymapState, VimMode};
use crate::{
    editing::motion::{
        char::CharMotion,
        word::{is_small_word_boundary, WordMotion},
        Motion,
    },
    input::completion::commands::CommandsCompleter,
    input::maps::actions::connection::send_current_input_buffer,
};
use crate::{
    editing::source::BufferSource,
    input::{completion::state::CompletionState, KeyCode, KeymapContext},
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
            let mut state = if let Some(current_state) = ctx.state_mut().current_window_mut().completion_state.take() {
                current_state
            } else {
                // TODO get the completer to use from context/window/buffer, probably
                let c = CommandsCompleter;
                CompletionState::new(c, &mut ctx)
            };

            state.apply_next(ctx.state_mut());

            ctx.state_mut().current_window_mut().completion_state = Some(state);

            Ok(())
        },
        "<s-tab>" => |ctx| {
            if let Some(mut state) = ctx.state_mut().current_window_mut().completion_state.take() {
                state.apply_prev(ctx.state_mut());

                ctx.state_mut().current_window_mut().completion_state = Some(state);
            }

            Ok(())
         },
    }
}

fn common_insert_mode(extra_mappings: Option<KeyTreeNode>) -> VimMode {
    let mut mappings = vim_tree! {
        "<esc>" => |ctx| {
            ctx.state_mut().clear_echo();
            ctx.state_mut().current_window_mut().set_inserting(false);
            ctx.state_mut().current_buffer_mut().end_change();
            CharMotion::Backward(1).apply_cursor(ctx.state_mut());
            Ok(())
         },
    } + vim_insert_mappings();

    if let Some(extra) = extra_mappings {
        mappings = mappings + extra;
    }

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

pub fn vim_standard_insert_mode() -> VimMode {
    common_insert_mode(None)
}

pub fn vim_conn_insert_mode() -> VimMode {
    let mappings = vim_tree! {
        "<cr>" => |?mut ctx| {
            send_current_input_buffer(ctx)
         },
    };

    common_insert_mode(Some(mappings))
}

pub fn vim_insert_mode(source: &BufferSource) -> VimMode {
    match source {
        BufferSource::ConnectionInputForBuffer(_) => vim_conn_insert_mode(),

        _ => vim_standard_insert_mode(),
    }
}
