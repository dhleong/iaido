use super::{Key, KeyCode};

// ======= Single-key parsing =============================

impl From<char> for Key {
    fn from(ch: char) -> Self {
        KeyCode::Char(ch).into()
    }
}

impl From<&str> for Key {
    fn from(s: &str) -> Self {
        if s.len() == 1 {
            s.chars().next().unwrap().into()
        } else {
            // TODO modifiers
            match s {
                " " | "20" | "space" => KeyCode::Char(' ').into(),
                "bs" | "backspace" => KeyCode::Backspace.into(),
                "backslash" => KeyCode::Char('\\').into(),
                "cr" | "enter" => KeyCode::Enter.into(),
                "esc" => KeyCode::Esc.into(),
                "tab" => KeyCode::Tab.into(),
                "s+tab" => KeyCode::BackTab.into(),

                "left" => KeyCode::Left.into(),
                "up" => KeyCode::Up.into(),
                "down" => KeyCode::Down.into(),
                "right" => KeyCode::Right.into(),

                "pagedown" => KeyCode::PageDown.into(),
                "pageup" => KeyCode::PageUp.into(),

                _ => todo!("parse: {}", s),
            }
        }
    }
}

impl From<String> for Key {
    fn from(s: String) -> Self {
        (&s[..]).into()
    }
}

impl From<&String> for Key {
    fn from(s: &String) -> Self {
        (&s[..]).into()
    }
}

// ======= Main string -> keys parsing ====================

fn parse_keys(s: &String) -> Vec<Key> {
    let mut v: Vec<Key> = Vec::new();
    let mut pending_key = String::default();
    let mut last_ch: char = 0.into();

    let mut in_special = false;
    for (i, ch) in s.char_indices() {
        if !in_special && start_special(s, i) {
            in_special = true
        } else if in_special && ch == '>' && last_ch != '\\' {
            // parse pending special key
            v.push((&pending_key.to_lowercase()).into());

            // reset
            in_special = false;
            pending_key.clear();
        } else if in_special && ch == '>' {
            // escaped >
            pending_key.remove(pending_key.len() - 1);
            pending_key.push(ch);
        } else if in_special {
            // any other char
            pending_key.push(ch);
        } else {
            // easy case: simple key
            v.push(ch.into());
        }
        last_ch = ch;
    }

    v
}

fn start_special(raw: &String, i: usize) -> bool {
    // special char only starts on `<`
    if raw[i..(i + 1)] != *"<" {
        return false;
    }

    // `<` must not have been escaped
    if i > 0 && raw[(i - 1)..i] == *"\\" {
        return false;
    }

    // look for a matching `>`; we start from 2 chars after
    // the `<` to handle special case `<>`, which is probably
    // not intended to be a special char sequence
    for j in (i + 2)..raw.len() {
        if raw[j..(j + 1)] == *">" && raw[(j - 1)..j] != *"\\" {
            // we found a matching, non-escaped `>`;
            // this is a legit special char sequence
            return true;
        }
    }

    return false;
}

// ======= Conveniences ===================================

pub trait KeysParsable {
    fn into_keys(&self) -> Vec<Key>;
}

impl KeysParsable for String {
    fn into_keys(&self) -> Vec<Key> {
        parse_keys(self)
    }
}

impl KeysParsable for &str {
    fn into_keys(&self) -> Vec<Key> {
        self.to_string().into_keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_keys() {
        let keys: Vec<Key> = "gTo".into_keys();
        assert_eq!(
            keys,
            vec![
                KeyCode::Char('g').into(),
                KeyCode::Char('T').into(),
                KeyCode::Char('o').into(),
            ]
        );
    }

    #[test]
    fn parse_single_special() {
        let keys: Vec<Key> = "<Cr>".into_keys();
        assert_eq!(keys, vec![KeyCode::Enter.into(),]);
    }
}
