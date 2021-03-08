use genawaiter::sync::gen;

use crate::declare_simple_completer;

declare_simple_completer!(
    EmptyCompleter (_app, _context) {
        gen!({})
    }
);
