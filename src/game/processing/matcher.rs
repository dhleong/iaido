use std::collections::HashMap;

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
            match Regex::new(pattern) {
                Ok(regex) => Ok(Matcher {
                    description: input,
                    regex,
                }),
                Err(e) => Err(KeyError::InvalidInput(e.to_string())),
            }
        } else {
            todo!("translate simple pattern");
        }
    }

    pub fn find(&self, input: TextLine) -> Option<Match> {
        if let Some(captures) = self.regex.captures(&input.to_string()) {
            let mut result = Match::empty();
            for (i, name) in self.regex.capture_names().enumerate() {
                let (key, captured) = if let Some(name) = name {
                    (name.to_string(), captures.name(name))
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
}
