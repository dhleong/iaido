use crate::{
    editing::text::EditableLine,
    input::{
        commands::{CommandHandler, CommandHandlerContext},
        KeyCode, KeymapContext,
    },
    key_handler, vim_tree,
};

use super::{insert::vim_insert_mappings, tree::KeyTreeNode, VimKeymapState, VimMode};

pub struct VimPromptConfig {
    pub prompt: String,
    pub handler: Box<CommandHandler>,
}

impl Into<VimMode> for VimPromptConfig {
    fn into(self) -> VimMode {
        let prompt = self.prompt.clone();
        let prompt_len = prompt.len();
        let mode_id = format!("prompt:{}", prompt);

        VimMode::new(mode_id.clone(), vim_insert_mappings() + mappings(self))
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

                    let cursor = ctx.state().current_window().cursor;
                    if cursor.line == 0 && cursor.col < prompt_len {
                        ctx.state_mut().current_window_mut().cursor.col = prompt_len;
                    }

                    Ok(())
                }
            ))
    }
}

fn mappings(config: VimPromptConfig) -> KeyTreeNode {
    let prompt_len = config.prompt.len();
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

             // submit to handler
             (config.handler)(&mut CommandHandlerContext::new(&mut ctx, input))
         },
    }
}
