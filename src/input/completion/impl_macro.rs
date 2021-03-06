#[macro_export]
macro_rules! impl_simple_completer {
    ($completer_name:ident (&$self:ident, $app:ident, $context:ident) $body:expr) => {
        impl crate::input::completion::Completer for $completer_name {
            fn suggest(
                &$self,
                $app: Box<&dyn crate::input::completion::CompletableContext>,
                $context: crate::input::completion::CompletionContext,
            ) -> crate::input::completion::BoxedSuggestions {
                let _input = $context.word().to_string();
                Box::new(
                    $body
                        .into_iter()
                        .map(move |s| $context.create_completion(s))
                        .filter(move |candidate| candidate.replacement.starts_with(&_input)),
                )
            }
        }
    };

    ($completer_name:ident ($app:ident, $context:ident) $body:expr) => {
        crate::impl_simple_completer!($completer_name (&self, $app, $context) $body);
    };

    ($completer_name:ident $body:expr) => {
        crate::impl_simple_completer!($completer_name (&self, _app, _context) $body);
    };
}

#[macro_export]
macro_rules! declare_simple_completer {
    ($completer_name:ident $($body:tt)+) => {
        pub struct $completer_name;

        crate::impl_simple_completer!($completer_name $($body)*);
    }
}
