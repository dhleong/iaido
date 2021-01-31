use crate::{
    editing::text::EditableLine,
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
             let input = ctx.state().prompt.buffer.get_contents()[prompt_len..].to_string();
             ctx.keymap.mode_stack.pop();
             ctx.state_mut().prompt.clear();

             // TODO submit to handler
             ctx.state_mut().current_buffer_mut().append(input.into());
             Ok(())
         },
    }
}

pub fn vim_prompt_mode(prompt: String) -> VimMode {
    VimMode::new(
        format!("prompt:{}", prompt),
        vim_insert_mappings() + mappings(prompt.clone()),
    )
    .on_default(key_handler!(
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
    .on_after(key_handler!(
        VimKeymapState move | ctx | {
            let b = &ctx.state().prompt.buffer;
            if b.is_empty() || !b.get(0).starts_with(&prompt) {
                ctx.state_mut().prompt.buffer.insert((0, 0).into(), prompt.clone().into());
            }

            Ok(())
        }
    ))
}
