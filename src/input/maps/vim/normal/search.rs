use crate::editing::motion::search::SearchMotion;
use crate::editing::CursorPosition;
use crate::input::commands::CommandHandler;
use crate::input::maps::vim::motion::{apply_motion, apply_motion_returning};
use crate::input::maps::vim::prompt::VimPromptConfig;
use crate::input::maps::vim::tree::KeyTreeNode;
use crate::input::maps::vim::VimKeymap;
use crate::input::maps::CommandHandlerContext;
use crate::input::maps::KeyError;
use crate::input::maps::KeyHandlerContext;
use crate::input::maps::KeyResult;
use crate::input::KeymapContext;
use crate::vim_tree;

const SEARCH_HISTORY_KEY: &str = "/";

#[derive(Default)]
pub struct VimSearchState {
    pub last_search_forward: bool,
}

fn perform_motion(
    context: &mut CommandHandlerContext,
    ui: char,
    query: &String,
    motion: SearchMotion,
) -> KeyResult {
    if let Some(keymap) = context.keymap.as_any_mut().downcast_mut::<VimKeymap>() {
        keymap.search.last_search_forward = ui == '/';
        keymap
            .histories
            .maybe_insert(SEARCH_HISTORY_KEY.to_string(), query.to_string());

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

fn handle_search_result<C: KeymapContext>(
    context: &mut C,
    ui: char,
    query: String,
    initial_cursor: CursorPosition,
    result: KeyResult,
) -> KeyResult {
    match result {
        Ok(()) => {
            let end_cursor = context.state().current_window().cursor;
            if end_cursor == initial_cursor {
                context.state_mut().clear_echo();
                Err(KeyError::PatternNotFound(query))
            } else {
                if end_cursor > initial_cursor && ui == '?' {
                    context.state_mut().clear_echo();
                    context
                        .state_mut()
                        .echom("Search hit TOP; continuing at BOTTOM");
                } else if end_cursor < initial_cursor && ui == '/' {
                    context.state_mut().clear_echo();
                    context
                        .state_mut()
                        .echom("Search hit BOTTOM; continuing at TOP");
                }

                Ok(())
            }
        }

        _ => result,
    }
}

fn handle_search(context: &mut CommandHandlerContext, ui: char, motion: SearchMotion) -> KeyResult {
    let query = context.input.to_string();
    if query.len() == 0 {
        // TODO Repeat the last search
        return Ok(());
    }

    context.state_mut().echo(format!("{}{}", ui, query).into());

    let initial_cursor = context.state().current_window().cursor;
    let result = perform_motion(context, ui, &query, motion);
    handle_search_result(context, ui, query, initial_cursor, result)
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
            history_key: "/".to_string(),
            handler,
            completer: None,
        }
        .into(),
    );

    Ok(())
}

fn next_search(ctx: KeyHandlerContext<VimKeymap>, match_direction: bool) -> KeyResult {
    let ui = if ctx.keymap.search.last_search_forward == match_direction {
        // Either we were going forward and we want to match that, or we were not
        // and *don't* want to match that
        '/'
    } else {
        '?'
    };

    if let Some(query_ref) = ctx.keymap.histories.get_most_recent(SEARCH_HISTORY_KEY) {
        let query = query_ref.to_string();
        let initial_cursor = ctx.state().current_window().cursor;
        let motion = if ui == '/' {
            SearchMotion::forward_until(query.to_string())
        } else {
            SearchMotion::backward_until(query.to_string())
        };
        let (mut ctx, result) = apply_motion_returning(ctx, motion);
        handle_search_result(&mut ctx, ui, query.to_string(), initial_cursor, result)
    } else {
        Err(KeyError::InvalidInput("No previous search".into()))
    }
}

pub fn mappings() -> KeyTreeNode {
    vim_tree! {
        "/" => |?mut ctx| activate_search(ctx, '/', Box::new(handle_forward_search)),
        "?" => |?mut ctx| activate_search(ctx, '?', Box::new(handle_backward_search)),

        "n" => |?mut ctx| next_search(ctx, true),
        "N" => |?mut ctx| next_search(ctx, false),
    }
}
