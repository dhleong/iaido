pub trait CompletionTokenizable {
    fn to_completion_tokens(&self) -> Vec<&str>;
    fn to_all_completion_tokens(&self) -> Vec<&str>;
}

const MIN_TOKEN_LENGTH: usize = 3;

fn is_non_token(ch: char) -> bool {
    return !(ch.is_alphabetic() || ch == '\'');
}

impl CompletionTokenizable for &str {
    fn to_completion_tokens(&self) -> Vec<&str> {
        self.split_terminator(is_non_token)
            .filter(|token| token.len() >= MIN_TOKEN_LENGTH)
            .collect()
    }

    fn to_all_completion_tokens(&self) -> Vec<&str> {
        self.split_terminator(is_non_token)
            .filter(|token| !token.is_empty())
            .collect()
    }
}

impl CompletionTokenizable for String {
    fn to_completion_tokens(&self) -> Vec<&str> {
        self.split_terminator(is_non_token)
            .filter(|token| token.len() >= MIN_TOKEN_LENGTH)
            .collect()
    }

    fn to_all_completion_tokens(&self) -> Vec<&str> {
        self.split_terminator(is_non_token)
            .filter(|token| !token.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn empty_strings() {
        let s = "".to_string();
        let tokens = s.to_completion_tokens();
        assert!(tokens.is_empty())
    }

    #[test]
    pub fn just_symbols() {
        let s = "( *$ ][".to_string();
        let tokens = s.to_completion_tokens();
        assert!(tokens.is_empty())
    }

    #[test]
    pub fn words() {
        let s = "You can't (take)".to_string();
        let tokens = s.to_completion_tokens();
        assert_eq!(tokens, vec!["You", "can't", "take"]);
    }
}
