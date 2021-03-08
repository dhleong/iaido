use genawaiter::{sync::gen, yield_};
use std::path::PathBuf;

use crate::declare_simple_completer;

fn is_path_word(c: char) -> bool {
    !char::is_whitespace(c)
}

declare_simple_completer!(
    FileCompleter (_app, context) {
        // TODO we need to allow spaces if escaped
        let given_word = context.word_where(is_path_word).to_string();
        let given_path = PathBuf::from(given_word.clone());
        gen!({
            let mut path = if let Ok(path) = std::env::current_dir() {
                path
            } else {
                return;
            };

            path.push(given_path);

            let dir_source = if given_word.is_empty() || path.exists() {
                // eg: `:e` or `:e src/`
                Some(path)
            } else {
                // eg: `:e Ca`  (a partial file name)
                path.parent().and_then(|b| Some(b.to_path_buf()))
            };

            if let Some(dir_source) = dir_source {
                if let Ok(dir) = dir_source.read_dir() {
                    for sibling in dir {
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

    #[test]
    fn complete_file_paths() {
        let mut app = window(":e src/|");

        let suggestions: Vec<String> = complete(&FileCompleter, &mut app)
            .into_iter()
            .filter(|name| name.ends_with(".rs"))
            .collect();
        assert_eq!(
            suggestions,
            vec!["log.rs".to_string(), "main.rs".to_string()]
        );
    }
}
