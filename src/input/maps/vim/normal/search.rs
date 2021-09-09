use crate::editing::motion::search::SearchMotion;
use crate::input::commands::CommandHandler;
use crate::input::maps::vim::motion::apply_motion;
use crate::input::maps::vim::prompt::VimPromptConfig;
use crate::input::maps::vim::tree::KeyTreeNode;
use crate::input::maps::vim::VimKeymap;
use crate::input::maps::CommandHandlerContext;
use crate::input::maps::KeyError;
use crate::input::maps::KeyHandlerContext;
use crate::input::maps::KeyResult;
use crate::input::KeymapContext;
use crate::vim_tree;

fn handle_search(context: &mut CommandHandlerContext, ui: char, motion: SearchMotion) -> KeyResult {
    let query = context.input.to_string();
    context.state_mut().echo(format!("{}{}", ui, query).into());

    if let Some(keymap) = context.keymap.as_any_mut().downcast_mut::<VimKeymap>() {
        let ctx = KeyHandlerContext {
            context: Box::new(&mut context.context),
            keymap,
            key: "<cr>".into(),
        };
        apply_motion(ctx, motion)
    } else {
        // This shouldn't be possible:
        panic!("Performing vim search without VimKeymap")
    }
}

fn handle_forward_search(context: &mut CommandHandlerContext) -> KeyResult {
    handle_search(
        context,
        '/',
        SearchMotion::forward_until(context.input.to_string()),
    )
}

fn handle_backward_search(context: &mut CommandHandlerContext) -> KeyResult {
    handle_search(
        context,
        '?',
        SearchMotion::backward_until(context.input.to_string()),
    )
}

fn activate_search(
    mut ctx: KeyHandlerContext<VimKeymap>,
    ui: char,
    handler: Box<CommandHandler>,
) -> KeyResult {
    ctx.state_mut().clear_echo();
    ctx.state_mut().prompt.activate(ui.to_string().into());

    ctx.keymap.push_mode(
        VimPromptConfig {
            prompt: ui.to_string(),
            handler,
            completer: None,
        }
        .into(),
    );

    Ok(())
}

pub fn mappings() -> KeyTreeNode {
    vim_tree! {
        "/" => |?mut ctx| {
            activate_search(ctx, '/', Box::new(handle_forward_search))
        },

        "?" => |?mut ctx| {
            activate_search(ctx, '?', Box::new(handle_backward_search))
        },

        "n" => |?mut _ctx| {
            Err(KeyError::InvalidInput("Result browsing not yet supported".to_string()))
        },

        "N" => |?mut _ctx| {
            Err(KeyError::InvalidInput("Result browsing not yet supported".to_string()))
        },
    }
}
