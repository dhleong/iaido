use genawaiter::{sync::gen, yield_};
use std::path::PathBuf;

use crate::declare_simple_completer;

declare_simple_completer!(
    FileCompleter (_app, context) {
        let given_path = PathBuf::from(context.word());
        gen!({
            let mut path = if let Ok(path) = std::env::current_dir() {
                path
            } else {
                return;
            };

            path.push(given_path);

            if let Ok(dir) = path.read_dir() {
                for child in dir {
                    if let Ok(entry) = child {
                        yield_!(entry.file_name().to_string_lossy().to_string());
                    }
                }
            }

            if let Some(parent) = path.parent() {
                if let Ok(parent_dir) = parent.read_dir() {
                    for sibling in parent_dir {
                        if let Ok(entry) = sibling {
                            yield_!(entry.file_name().to_string_lossy().to_string());
                        }
                    }
                }
            }
        })
    }
);

#[cfg(test)]
mod tests {
    use crate::{editing::motion::tests::window, input::completion::tests::complete};

    use super::*;

    #[test]
    fn complete_files() {
        let mut app = window(":e C|");

        let suggestions = complete(&FileCompleter, &mut app);
        assert_eq!(
            suggestions,
            vec!["Cargo.toml".to_string(), "Cargo.lock".to_string()]
        );
    }
}
