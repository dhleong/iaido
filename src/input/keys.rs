use super::{Key, KeyCode, KeyModifiers};

// ======= Single-key parsing =============================

impl From<char> for Key {
    fn from(ch: char) -> Self {
        KeyCode::Char(ch).into()
    }
}

impl From<&str> for Key {
    fn from(s: &str) -> Self {
        parse_key(s).ok().unwrap()
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

enum KeyParseError {
    InvalidModifier(String),
    InvalidKey(String),
}

fn parse_key(s: &str) -> Result<Key, KeyParseError> {
    if s.len() == 1 {
        // easy case:
        return Ok(s.chars().next().unwrap().into());
    }

    let s = if s.starts_with("<") && s.ends_with(">") {
        &s[1..s.len() - 1]
    } else {
        s
    };

    let mut modifiers = KeyModifiers::empty();
    let mut code = KeyCode::Char('\0');

    let parts = s.split("-").count();
    for (i, part) in s.split("-").enumerate() {
        if i < parts - 1 {
            modifiers |= match part {
                "a" | "alt" => KeyModifiers::ALT,
                "c" | "ctrl" => KeyModifiers::CONTROL,
                "s" | "shift" => KeyModifiers::SHIFT,
                _ => return Err(KeyParseError::InvalidModifier(part.to_string())),
            };
        } else if part.len() == 1 {
            code = KeyCode::Char(part.chars().next().unwrap());
        } else {
            code = match part {
                " " | "20" | "space" => KeyCode::Char(' '),
                "bs" | "backspace" => KeyCode::Backspace,
                "backslash" => KeyCode::Char('\\'),
                "cr" | "enter" => KeyCode::Enter,
                "esc" => KeyCode::Esc,
                "tab" => KeyCode::Tab,

                "left" => KeyCode::Left,
                "up" => KeyCode::Up,
                "down" => KeyCode::Down,
                "right" => KeyCode::Right,

                "pagedown" => KeyCode::PageDown,
                "pageup" => KeyCode::PageUp,

                _ => return Err(KeyParseError::InvalidKey(part.to_string())),
            };
        }
    }

    Ok(Key { code, modifiers })
}

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

    #[test]
    fn parse_modifiers() {
        let keys: Vec<Key> = "<aLt-Cr>".into_keys();
        assert_eq!(keys, vec![Key::new(KeyCode::Enter, KeyModifiers::ALT),]);
    }

    #[test]
    fn parse_abbreviated_modifiers() {
        assert_eq!(
            "<c-c>".into_keys(),
            vec![Key::new(KeyCode::Char('c'), KeyModifiers::CONTROL),]
        );
        assert_eq!(
            Key::from("<c-c>"),
            Key::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        );
    }
}
