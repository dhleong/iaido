use crate::declare_simple_completer;
use genawaiter::{sync::gen, yield_};

declare_simple_completer!(
    CommandsCompleter (app, context) {
        let names: Vec<String> = app.commands()
            .names()
            .map(|v| v.to_string())
            .collect();

        gen!({
            // NOTE: this generator is obviously not doing much work
            // here, but more complicated completers might benefit...
            for name in names {
                yield_!(name);
            }
        })
    }
);
