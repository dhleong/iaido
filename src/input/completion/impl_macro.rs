#[macro_export]
macro_rules! impl_simple_completer {
    ($completer_name:ident ($app:ident, $context:ident) $body:expr) => {
        impl crate::input::completion::Completer for $completer_name {
            type Iter = Box<dyn Iterator<Item = crate::input::completion::Completion>>;

            fn suggest<T: crate::input::completion::CompletableContext>(
                &self,
                $app: &T,
                $context: crate::input::completion::CompletionContext,
            ) -> Self::Iter {
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

    ($completer_name:ident $body:expr) => {
        crate::impl_simple_completer!($completer_name (_app, _context) $body);
    }
}

#[macro_export]
macro_rules! declare_simple_completer {
    ($completer_name:ident $($body:tt)+) => {
        pub struct $completer_name;

        crate::impl_simple_completer!($completer_name $($body)*);
    }
}
