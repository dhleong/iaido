use std::collections::HashMap;

use lazy_static::lazy_static;
use regex::Regex;

use crate::{
    editing::text::{EditableLine, TextLine},
    input::{maps::KeyResult, KeyError},
};

/// A Matcher represents a compiled "match" pattern that can be used to power the various
/// text-processing utilities we might provide. Given a [TextLine], the Matcher can return
/// a [Match] if the pattern was found, from which the extracted groups may be pulled.
pub struct Matcher {
    pub description: String,
    regex: Regex,
}

impl Matcher {
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

    /// Returns an iterator of all the capture groups declared in this Matcher
    pub fn groups(&self) -> CaptureGroups {
        CaptureGroups(self.regex.capture_names().enumerate())
    }

    pub fn find(&self, input: &TextLine) -> Option<Match> {
        if let Some(captures) = self.regex.captures(&input.to_string()) {
            let mut groups = HashMap::default();
            for group in self.groups() {
                let captured = if let Some(i) = group.index {
                    captures.get(i)
                } else if let Some(name) = group.group_name {
                    captures.name(name)
                } else {
                    panic!(
                        "Invalid capture group {} has neither name nor index",
                        group.name
                    );
                };

                if let Some(value) = captured {
                    let text = input.subs(value.start(), value.end());
                    groups.insert(group.name, text);
                }
            }

            let (start, end) = if let Some(m) = captures.get(0) {
                (m.start(), m.end())
            } else {
                (0, input.width())
            };

            return Some(Match { groups, start, end });
        }
        None
    }
}

pub struct CaptureGroups<'r>(std::iter::Enumerate<regex::CaptureNames<'r>>);

impl<'r> Iterator for CaptureGroups<'r> {
    type Item = CaptureGroup<'r>;

    fn next(&mut self) -> Option<CaptureGroup<'r>> {
        match self.0.next() {
            Some((_, Some(name))) => Some(CaptureGroup {
                name: name.trim_start_matches("_VAR_").to_string(),
                group_name: Some(name),
                index: None,
            }),
            Some((index, None)) => Some(CaptureGroup {
                name: index.to_string(),
                group_name: None,
                index: Some(index),
            }),
            _ => None,
        }
    }
}

pub struct CaptureGroup<'r> {
    pub name: String,
    group_name: Option<&'r str>,
    index: Option<usize>,
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
        p.push('^');
    } else {
        p.push_str(r"\b")
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

    p.push_str(r"(?:\b|\s|$)");

    p
}

#[derive(Clone, Debug)]
pub struct Match {
    pub start: usize,
    pub end: usize,
    groups: HashMap<String, TextLine>,
}

impl Match {
    #[allow(dead_code)] // TODO Remove this when we can...
    pub fn group(&self, name: &str) -> Option<&TextLine> {
        self.groups.get(name)
    }

    pub fn expand(&self, replacement: &str) -> String {
        let mut result = replacement.to_string();
        for (group, value) in self.groups.iter() {
            let value_str = value.to_string();
            Match::replace_group(&mut result, format!("${}", group), &value_str)
        }
        result
    }

    fn replace_group(dest: &mut String, target: String, value: &str) {
        loop {
            if let Some(index) = dest.find(&target) {
                dest.replace_range(index..index + target.len(), value);
            } else {
                break;
            }
        }
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
            let input = "saute peppers".into();
            let m = matcher.find(&input).expect("Failed to match");
            assert_eq!(group(m, "1"), "peppers");
        }

        #[test]
        fn simple_named() {
            let matcher = re_compile(r"/^saute (?P<food>\w+)/");
            let input = "saute peppers".into();
            let m = matcher.find(&input).expect("Failed to match");
            assert_eq!(m.group("food").unwrap().to_string(), "peppers");
        }
    }

    #[cfg(test)]
    mod simple {
        use super::*;

        #[test]
        fn indexed() {
            let matcher = re_compile(r"^saute $0");
            let input = "saute peppers".into();
            let m = matcher.find(&input).expect("Failed to match");
            assert_eq!(group(m, "0"), "peppers");
        }

        #[test]
        fn named() {
            let matcher = re_compile(r"^saute ${food}");
            let input = "saute peppers".into();
            let m = matcher.find(&input).expect("Failed to match");
            assert_eq!(group(m, "food"), "peppers");
        }
    }
}
