use crate::input::commands::CommandHandlerContext;
use crate::input::maps::KeyResult;
use crate::input::KeymapContext;
use command_decl::declare_commands;

pub struct HelpTopic {
    pub topic: String,
}

impl From<String> for HelpTopic {
    fn from(topic: String) -> HelpTopic {
        HelpTopic { topic }
    }
}

impl From<&&str> for HelpTopic {
    fn from(topic: &&str) -> HelpTopic {
        HelpTopic {
            topic: topic.to_string(),
        }
    }
}

declare_commands!(declare_help {
    /// This command. View help on using iaido on general, or a specific topic.
    pub fn help(context, subject: Option<HelpTopic>) {
        help(context, subject)
    },
});

fn help(context: &mut CommandHandlerContext, subject: Option<HelpTopic>) -> KeyResult {
    match subject {
        Some(topic) => {
            context.state_mut().echom(format!("help: {}", topic.topic));
        }
        _ => {
            context.state_mut().echom("TODO: help overview");
        }
    };
    Ok(())
}
