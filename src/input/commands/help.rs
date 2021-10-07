use indoc::indoc;

use crate::app::help::{self, HelpTopic};
use crate::editing::layout::Layout;
use crate::editing::source::BufferSource;
use crate::editing::Id;
use crate::input::commands::CommandHandlerContext;
use crate::input::maps::KeyResult;
use crate::input::KeymapContext;
use clap::crate_version;
use command_decl::declare_commands;

declare_commands!(declare_help {
    /// This command. View help on using iaido on general, or a specific topic.
    pub fn help(context, subject: Option<HelpTopic>) {
        help(context, subject)
    },
});

fn find_help_window(context: &mut CommandHandlerContext) -> Option<Id> {
    context.state().current_tab().layout.iter().find_map(|win| {
        if let Some(buf) = context.state().buffers.by_id(win.buffer) {
            match buf.source() {
                &BufferSource::Help => Some(win.id),
                _ => None,
            }
        } else {
            None
        }
    })
}

fn ensure_help_window(context: &mut CommandHandlerContext) -> Id {
    // If we already have a help window in the current tab, use it:
    if let Some(existing_window) = find_help_window(context) {
        return existing_window;
    }

    // TODO If the current window uses the full width of the screen or is at least
    // 80 characters wide, split upward

    // Otherwise, just create the help window at the very top
    return context.state_mut().current_tab_mut().split_top();
}

fn show_help_window(context: &mut CommandHandlerContext, help: String) {
    let help_win_id = ensure_help_window(context);
    context.state_mut().current_tab_mut().set_focus(help_win_id);

    // TODO We don't always need to create a new buffer
    let buf_id = context.state_mut().buffers.create().id();
    context.state_mut().set_current_window_buffer(buf_id);

    let buffer = context.state_mut().current_buffer_mut();
    buffer.set_source(BufferSource::Help);
    buffer.clear();
    buffer.append(help::format(help));
}

fn help(context: &mut CommandHandlerContext, subject: Option<HelpTopic>) -> KeyResult {
    match subject {
        Some(HelpTopic { topic }) => {
            // TODO Get a whole page on which topic appears
            if let Some(help) = context.state().builtin_commands.get_doc(&topic) {
                let help_str = help.to_string();
                let command_name = context
                    .state()
                    .builtin_commands
                    .expand_name(&topic)
                    .unwrap()
                    .to_string();
                show_help_window(
                    context,
                    format!("## [{}]({})\n\n{}", command_name, command_name, help_str),
                );
            }
        }

        _ => {
            let mut s = String::new();
            s.push_str(&format!(
                indoc! {"
                    # iaido {}

                    ## About
                    More help TK

                    - Try :help connect<Enter>

                    ## Commands:\n\n
                "},
                crate_version!()
            ));

            for name in context.state().builtin_commands.names() {
                s.push_str(" - [");
                s.push_str(name);
                s.push_str("](");
                s.push_str(name);
                s.push_str(")\n");
            }

            show_help_window(context, s);
        }
    };
    Ok(())
}
