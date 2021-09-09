use crate::editing::FocusDirection;
use crate::input::maps::vim::prompt::VimPromptConfig;
use crate::input::maps::vim::tree::KeyTreeNode;
use crate::input::maps::vim::VimKeymap;
use crate::input::maps::CommandHandlerContext;
use crate::input::maps::KeyError;
use crate::input::maps::KeyResult;
use crate::input::KeymapContext;
use crate::vim_tree;

fn handle_search(context: &mut CommandHandlerContext, _direction: FocusDirection) -> KeyResult {
    // TODO
    context.state_mut().echo_str("TODO: search");
    Ok(())
}

fn handle_forward_search(context: &mut CommandHandlerContext) -> KeyResult {
    handle_search(context, FocusDirection::Down)
}

fn handle_reverse_search(context: &mut CommandHandlerContext) -> KeyResult {
    handle_search(context, FocusDirection::Up)
}

pub fn mappings() -> KeyTreeNode {
    vim_tree! {
        "/" => |ctx| {
            ctx.state_mut().clear_echo();
            ctx.state_mut().prompt.activate("/".into());

            ctx.keymap.push_mode(VimPromptConfig{
                prompt: "/".into(),
                handler: Box::new(handle_forward_search),
                completer: None,
            }.into());

            Ok(())
        },

        "?" => |ctx| {
            ctx.state_mut().clear_echo();
            ctx.state_mut().prompt.activate("?".into());

            ctx.keymap.push_mode(VimPromptConfig{
                prompt: "?".into(),
                handler: Box::new(handle_reverse_search),
                completer: None,
            }.into());

            Ok(())
        },

        "n" => |?mut _ctx| {
            Err(KeyError::InvalidInput("Result browsing not yet supported".to_string()))
        },

        "N" => |?mut _ctx| {
            Err(KeyError::InvalidInput("Result browsing not yet supported".to_string()))
        },
    }
}
