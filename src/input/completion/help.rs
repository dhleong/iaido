use crate::declare_simple_completer;

use super::commands::CommandNamesCompleter;
use super::Completer;

declare_simple_completer!(
    HelpFilenameCompleter (app, context) {
        let names: Vec<String> = app.commands()
            .help
            .filenames()
            .map(|v| v.to_string())
            .collect();
        names.into_iter()
    }
);

pub struct HelpTopicCompleter;

impl Completer for HelpTopicCompleter {
    fn suggest(
        &self,
        app: Box<&dyn super::CompletableContext>,
        context: super::CompletionContext,
    ) -> super::BoxedSuggestions {
        let filenames = HelpFilenameCompleter.suggest(app.clone(), context.clone());
        let names = CommandNamesCompleter.suggest(app, context);
        Box::new(filenames.chain(names))
    }
}
