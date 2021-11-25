use std::rc::Rc;

use crate::{
    editing::text::EditableLine,
    input::{
        commands::{CommandHandler, CommandHandlerContext},
        completion::Completer,
        KeyCode, KeymapContext,
    },
    key_handler, vim_tree,
};

use super::{insert::vim_insert_mappings, tree::KeyTreeNode, VimKeymap, VimMode};

pub struct VimPromptConfig {
    pub prompt: String,
    pub history_key: String,
    pub handler: Box<CommandHandler>,
    pub completer: Option<Rc<dyn Completer>>,
}

impl Into<VimMode> for VimPromptConfig {
    fn into(mut self) -> VimMode {
        let prompt = self.prompt.clone();
        let prompt_len = prompt.len();
        let mode_id = format!("prompt:{}", prompt);
        let completer = self.completer.take();

        VimMode::new(mode_id.clone(), vim_insert_mappings() + mappings(self))
            .with_completer(completer)
            .on_default(key_handler!(
                VimKeymap | ctx | {
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
                VimKeymap move | ctx | {
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
    let prompt = config.prompt.to_string();
    let prompt_len = config.prompt.len();
    let history_key = config.history_key.to_string();
    vim_tree! {
        "<esc>" => |ctx| {
            ctx.keymap.mode_stack.pop();
            ctx.state_mut().prompt.clear();
            Ok(())
         },

         "<up>" => move |ctx| {
             // TODO Back in time
             if let Some(previous) = ctx.keymap.histories.get_most_recent(&history_key) {
                 let mut new_content = String::new();
                 new_content.push_str(&prompt);
                 new_content.push_str(previous);

                 ctx.state_mut().prompt.activate(new_content.into());
             }
             Ok(())
          },

         "<cr>" => move |ctx| {
             let input = ctx.state().prompt.buffer.get_contents()[prompt_len..].to_string();
             ctx.keymap.mode_stack.pop();
             ctx.state_mut().prompt.clear();

             ctx.keymap.histories.maybe_insert(config.history_key.to_string(), input.to_string());

             // submit to handler
             (config.handler)(&mut CommandHandlerContext::new(&mut ctx.context, ctx.keymap, input))
         },
    }
}
