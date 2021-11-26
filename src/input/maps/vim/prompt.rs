use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::{
    editing::text::EditableLine,
    input::{
        commands::{CommandHandler, CommandHandlerContext},
        completion::Completer,
        history::HistoryCursor,
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
    let up_prompt = config.prompt.to_string();
    let down_prompt = config.prompt.to_string();
    let prompt_len = config.prompt.len();
    let up_history_key = config.history_key.to_string();
    let down_history_key = config.history_key.to_string();

    let cursor = Arc::new(Mutex::new(HistoryCursor::default()));
    let up_cursor = cursor.clone();
    let down_cursor = cursor.clone();

    vim_tree! {
        "<esc>" => |ctx| {
            ctx.keymap.mode_stack.pop();
            ctx.state_mut().prompt.clear();
            Ok(())
        },

        "<up>" => move |ctx| {
            let mut cursor = up_cursor.lock().unwrap();
            let history = ctx.keymap.histories.take(&up_history_key);
            if let Some(previous) = cursor.back(&mut ctx.state_mut().prompt, prompt_len, &history) {
                let mut new_content = String::new();
                new_content.push_str(&up_prompt);
                new_content.push_str(previous);

                ctx.state_mut().prompt.activate(new_content.into());
            }
            ctx.keymap.histories.replace(up_history_key.to_string(), history);
            Ok(())
        },

        "<down>" => move |ctx| {
            let mut cursor = down_cursor.lock().unwrap();
            let history = ctx.keymap.histories.take(&down_history_key);
            if let Some(previous) = cursor.forward(&history) {
                let mut new_content = String::new();
                new_content.push_str(&down_prompt);
                new_content.push_str(previous);

                ctx.state_mut().prompt.activate(new_content.into());
            } else if let Some(original) = cursor.take_stashed_input() {
                let mut new_content = String::new();
                new_content.push_str(&down_prompt);
                new_content.push_str(&original);

                ctx.state_mut().prompt.activate(new_content.into());
            }
            ctx.keymap.histories.replace(down_history_key.to_string(), history);
            Ok(())
        },

         "<cr>" => move |ctx| {
             let input = ctx.state().prompt.buffer.get_contents()[prompt_len..].to_string();

             ctx.keymap.histories.maybe_insert(config.history_key.to_string(), input.to_string());
             ctx.keymap.mode_stack.pop();
             ctx.state_mut().prompt.clear();

             // submit to handler
             (config.handler)(&mut CommandHandlerContext::new(&mut ctx.context, ctx.keymap, input))
         },
    }
}
