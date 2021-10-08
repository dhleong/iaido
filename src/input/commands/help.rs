use indoc::indoc;
use std::fmt::Write;

use crate::app::help::{self, HelpQuery, HelpTopic};
use crate::editing::layout::Layout;
use crate::editing::source::BufferSource;
use crate::editing::Id;
use crate::input::commands::CommandHandlerContext;
use crate::input::maps::KeyResult;
use crate::input::KeyError;
use crate::input::KeymapContext;
use clap::crate_version;
use command_decl::declare_commands;

declare_commands!(declare_help {
    /// This command. View help on using iaido on general, or a specific topic.
    pub fn help(context, subject: Option<HelpQuery>) {
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
    if context.state().current_window().size.w >= 80 {
        return context.state_mut().current_tab_mut().hsplit();
    }

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

fn generate_help_entry(help: &HelpTopic) -> String {
    let mut help_str = help.doc;
    let command_name = help.topic;

    if help_str.is_empty() {
        help_str = "TK";
    }

    return format!("## [{}]({})\n\n{}", command_name, command_name, help_str);
}

fn generate_help_file(context: &mut CommandHandlerContext, filename: &str) -> String {
    let mut file = String::new();
    file.push_str("# Help Topic: ***");
    file.push_str(filename);
    file.push_str("***");

    // Help files may have an intro section using inner doc comments
    if let Some(intro) = context.state().builtin_commands.help.doc_for_file(filename) {
        file.push_str("\n\n");
        file.push_str(intro);
        file.push_str("\n\n");
    }

    let entries = context
        .state()
        .builtin_commands
        .help
        .entries_for_file(filename);
    for entry in entries {
        file.push_str("\n\n");
        file.push_str(&generate_help_entry(entry));
    }

    return file;
}

fn generate_help_index(context: &mut CommandHandlerContext) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        indoc! {"
            # iaido {}

            ## About
            More help TK

            - Try :help connect<Enter>

            ## Help files

            Try :help [filename]()<Enter>
        "},
        crate_version!()
    ));

    s.push_str("\n\n");

    for name in context.state().builtin_commands.help.filenames() {
        write!(&mut s, " - [{}]({})\n", name, name).unwrap();
    }

    s.push_str("\n\n## Commands:\n\n");

    for name in context.state().builtin_commands.names() {
        write!(&mut s, " - [{}]({})\n", name, name).unwrap();
    }

    return s;
}

fn help(context: &mut CommandHandlerContext, subject: Option<HelpQuery>) -> KeyResult {
    match subject {
        Some(HelpQuery { query }) => {
            if let Some(help) = context.state().builtin_commands.get_doc(&query) {
                // TODO Get a whole page on which topic appears, and jump to
                // where the topic is
                let help = generate_help_entry(help);
                show_help_window(context, help);
            } else if context.state().builtin_commands.help.has_filename(&query) {
                // Generate full help file
                let help = generate_help_file(context, &query);
                show_help_window(context, help);
            } else {
                context
                    .state_mut()
                    .echom_error(KeyError::PatternNotFound(format!(
                        "No help matching: '{}'",
                        query
                    )));
            }
        }

        _ => {
            let help = generate_help_index(context);
            show_help_window(context, help);
        }
    };
    Ok(())
}
