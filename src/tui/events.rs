use std::{
    collections::VecDeque,
    io,
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    time::Duration,
};

use crossterm::event::{Event, KeyCode, KeyModifiers};

use crate::{
    app::dispatcher::DispatchSender,
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
    running: Arc<Mutex<bool>>,
    events: Receiver<UiEvent>,
    pending_events: VecDeque<UiEvent>,
}

impl TuiEvents {
    pub fn start_with_dispatcher(dispatcher: DispatchSender) -> Self {
        let running = Arc::new(Mutex::new(true));
        let (tx, events) = mpsc::channel();

        let running_ref = running.clone();
        tokio::runtime::Handle::current().spawn_blocking(move || {
            while *running_ref.lock().unwrap() {
                let event = match crossterm::event::read() {
                    Ok(Event::Resize(_, _)) => UiEvent::Redraw,
                    Ok(Event::Key(key)) => UiEvent::Key(key.into()),
                    Ok(Event::Mouse(_)) => UiEvent::Redraw,
                    // TODO dispatch error?
                    _ => break,
                };

                tx.send(event).ok();

                // Dispatch a nop to the main thread to ensure we stop
                // waiting and check for events
                dispatcher.spawn(|_| ()).background();
            }
        });

        Self {
            running,
            events,
            pending_events: Default::default(),
        }
    }
}

impl Drop for TuiEvents {
    fn drop(&mut self) {
        if let Ok(mut lock) = self.running.lock() {
            *lock = false;
        }
    }
}

impl UiEvents for TuiEvents {
    fn poll_event(&mut self, timeout: Duration) -> io::Result<Option<UiEvent>> {
        if let Some(pending) = self.pending_events.front() {
            return Ok(Some(pending.clone()));
        }

        match self.events.recv_timeout(timeout) {
            Ok(received) => {
                self.pending_events.push_front(received);
                Ok(Some(received))
            }
            _ => Ok(None),
        }
    }

    fn next_event(&mut self) -> io::Result<UiEvent> {
        if let Some(pending) = self.pending_events.pop_front() {
            return Ok(pending);
        }

        Ok(self.events.recv().unwrap())
    }
}
