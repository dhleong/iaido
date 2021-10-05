use super::commands::CommandNamesCompleter;
use super::Completer;

pub struct HelpTopicCompleter;

impl Completer for HelpTopicCompleter {
    fn suggest(
        &self,
        app: Box<&dyn super::CompletableContext>,
        context: super::CompletionContext,
    ) -> super::BoxedSuggestions {
        // TODO Other help topics
        return CommandNamesCompleter.suggest(app, context);
    }
}
