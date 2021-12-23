use std::collections::HashMap;

use lazy_static::lazy_static;
use regex::Regex;

use crate::{
    editing::text::{EditableLine, TextLine},
    input::{maps::KeyResult, KeyError},
};

pub struct Matcher {
    pub description: String,
    regex: Regex,
}

impl Matcher {
    #[allow(dead_code)] // TODO Remove this when we can...
    pub fn compile(input: String) -> KeyResult<Matcher> {
        if input.starts_with("/") && input.ends_with("/") {
            let pattern = input.trim_matches('/');
            Self::from_pattern(&input, pattern)
        } else {
            let pattern = simple_matcher_to_pattern(&input);
            Self::from_pattern(&input, &pattern)
        }
    }

    fn from_pattern<D: Into<String>>(description: D, pattern: &str) -> KeyResult<Matcher> {
        match Regex::new(pattern) {
            Ok(regex) => Ok(Matcher {
                description: description.into(),
                regex,
            }),
            Err(e) => Err(KeyError::InvalidInput(e.to_string())),
        }
    }

    pub fn find(&self, input: TextLine) -> Option<Match> {
        if let Some(captures) = self.regex.captures(&input.to_string()) {
            let mut result = Match::empty();
            for (i, name) in self.regex.capture_names().enumerate() {
                let (key, captured) = if let Some(name) = name {
                    let clean_name = name.trim_start_matches("_VAR_");
                    (clean_name.to_string(), captures.name(name))
                } else {
                    (i.to_string(), captures.get(i))
                };

                if let Some(value) = captured {
                    let text = input.subs(value.start(), value.end());
                    result.groups.insert(key, text);
                }
            }
            return Some(result);
        }
        None
    }
}

/// Given a string representing a simple input spec (that is, $1/$2 and ${name}-style variable
/// placeholders and no regex except for `^`), compile a regex pattern
fn simple_matcher_to_pattern(input: &str) -> String {
    lazy_static! {
        static ref VAR_REGEX: Regex = Regex::new(r"\$(\d+|\{\w+\})").unwrap();
    }

    let mut p = String::new();
    let starts_at_beginning = input.starts_with('^');
    let mut last_end = if starts_at_beginning { 1 } else { 0 };

    if starts_at_beginning {
        // NOTE: We do this here to avoid the ^ getting escaped below
        p.push('^');
    }

    for m in VAR_REGEX.find_iter(&input) {
        p.push_str(&regex::escape(&input[last_end..m.start()]));
        last_end = m.end();

        let var_name_range = if &input[m.start() + 1..m.start() + 2] == "{" {
            // Named var
            m.start() + 2..m.end() - 1
        } else {
            // Simple number
            m.start() + 1..m.end()
        };
        p.push_str("(?P<_VAR_");
        p.push_str(&input[var_name_range]);
        p.push_str(">[^ ]+)");
    }

    p
}

pub struct Match {
    groups: HashMap<String, TextLine>,
}

impl Match {
    fn empty() -> Self {
        Self {
            groups: Default::default(),
        }
    }

    #[allow(dead_code)] // TODO Remove this when we can...
    pub fn group(&self, name: &str) -> Option<&TextLine> {
        self.groups.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn re_compile(s: &str) -> Matcher {
        Matcher::compile(s.to_string()).expect("Failed to compile")
    }

    fn group(m: Match, name: &str) -> String {
        m.group(name)
            .expect(&format!("Did not find group '{}'", name))
            .to_string()
    }

    #[cfg(test)]
    mod regex {
        use super::*;

        #[test]
        fn simple() {
            let matcher = re_compile(r"/^saute (\w+)/");
            let m = matcher
                .find("saute peppers".into())
                .expect("Failed to match");
            assert_eq!(group(m, "1"), "peppers");
        }

        #[test]
        fn simple_named() {
            let matcher = re_compile(r"/^saute (?P<food>\w+)/");
            let m = matcher
                .find("saute peppers".into())
                .expect("Failed to match");
            assert_eq!(m.group("food").unwrap().to_string(), "peppers");
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn indexed() {
            let matcher = re_compile(r"^saute $0");
            let m = matcher
                .find("saute peppers".into())
                .expect("Failed to match");
            assert_eq!(group(m, "0"), "peppers");
        }

        #[test]
        fn named() {
            let matcher = re_compile(r"^saute ${food}");
            let m = matcher
                .find("saute peppers".into())
                .expect("Failed to match");
            assert_eq!(group(m, "food"), "peppers");
        }
    }
}
