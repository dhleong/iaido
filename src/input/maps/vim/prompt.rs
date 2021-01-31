use crate::{
    input::{KeyCode, KeymapContext},
    key_handler, vim_tree,
};

use super::{insert::vim_insert_mappings, tree::KeyTreeNode, VimKeymapState, VimMode};

fn mappings(prompt: String) -> KeyTreeNode {
    let prompt_len = prompt.len();
    vim_tree! {
        "<esc>" => |ctx| {
            ctx.keymap.mode_stack.pop();
            ctx.state_mut().prompt.clear();
            Ok(())
         },

         "<cr>" => move |ctx| {
             let input = ctx.state().prompt.buffer.to_string()[prompt_len..].to_string();
             ctx.keymap.mode_stack.pop();
             ctx.state_mut().prompt.clear();

             // TODO submit to handler
             ctx.state_mut().current_buffer_mut().append(input.into());
             Ok(())
         },
    }
}

pub fn vim_prompt_mode(prompt: String) -> VimMode {
    // TODO an "after" handler to ensure we don't delete or move onto the prompt
    VimMode {
        id: format!("prompt:{}", prompt),
        mappings: vim_insert_mappings() + mappings(prompt),
        default_handler: Some(key_handler!(
            VimKeymapState | ctx | {
                match ctx.key.code {
                    KeyCode::Char(c) => {
                        ctx.state_mut().type_at_cursor(c);
                    }
                    _ => {} // ignore
                };

                Ok(())
            }
        )),
    }
}
