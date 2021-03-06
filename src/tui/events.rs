use std::{io, time::Duration};

use crossterm::{event::Event, event::KeyCode, event::KeyModifiers, ErrorKind};

use crate::{
    input::Key,
    ui::{UiEvent, UiEvents},
};

// ======= Conversions ====================================

impl From<crossterm::event::KeyEvent> for Key {
    fn from(ev: crossterm::event::KeyEvent) -> Self {
        match ev.code {
            KeyCode::Char(ch) => {
                // NOTE: capital ascii letters from crossterm include the SHIFT modifier, but
                // symbols like ! do not. For consistency, let's remove SHIFT from letters, too:
                if ch.is_alphabetic() && ch == ch.to_ascii_uppercase() {
                    return Key::new(ev.code, ev.modifiers - KeyModifiers::SHIFT);
                }

                // some special cases by experimentation:
                match ch {
                    '\u{7f}' if !ev.modifiers.is_empty() => {
                        return Key::new(KeyCode::Backspace, ev.modifiers);
                    }
                    '\r' if !ev.modifiers.is_empty() => {
                        return Key::new(KeyCode::Enter, ev.modifiers);
                    }

                    _ => {} // fall through...
                };
            }

            KeyCode::BackTab => {
                return Key::new(KeyCode::Tab, ev.modifiers | KeyModifiers::SHIFT);
            }

            _ => {} // fall through for default:
        }

        Key::new(ev.code, ev.modifiers)
    }
}

// ======= TuiEvents ======================================

pub struct TuiEvents {
    pending_event: Option<UiEvent>,
}

impl Default for TuiEvents {
    fn default() -> Self {
        Self {
            pending_event: None,
        }
    }
}

fn wrap_as_io(e: ErrorKind) -> io::Error {
    match e {
        ErrorKind::IoError(source) => source,
        _ => io::Error::new(io::ErrorKind::Other, e),
    }
}

impl UiEvents for TuiEvents {
    fn poll_event(&mut self, timeout: Duration) -> io::Result<Option<UiEvent>> {
        match crossterm::event::poll(timeout) {
            Ok(found) if found => {
                if let Some(pending) = self.pending_event {
                    // unconsumed pending event; return unchanged:
                    Ok(Some(pending))
                } else {
                    let next = self.next_event()?;
                    self.pending_event = Some(next);
                    Ok(Some(next))
                }
            }
            Ok(_) => Ok(None),
            Err(e) => Err(wrap_as_io(e)),
        }
    }

    fn next_event(&mut self) -> Result<UiEvent, io::Error> {
        if let Some(pending) = self.pending_event {
            self.pending_event = None;
            return Ok(pending);
        }

        match crossterm::event::read() {
            Ok(Event::Resize(_, _)) => Ok(UiEvent::Redraw),
            Ok(Event::Key(key)) => Ok(UiEvent::Key(key.into())),
            Ok(Event::Mouse(_)) => Ok(UiEvent::Redraw),
            Err(e) => Err(wrap_as_io(e)),
        }
    }
}
