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
    events: Receiver<io::Result<UiEvent>>,
    pending_events: VecDeque<io::Result<UiEvent>>,
}

impl TuiEvents {
    pub fn start_with_dispatcher(dispatcher: DispatchSender) -> Self {
        let running = Arc::new(Mutex::new(true));
        let (tx, events) = mpsc::channel();

        let running_ref = running.clone();
        tokio::runtime::Handle::current().spawn_blocking(move || {
            while *running_ref.lock().unwrap() {
                let event = match crossterm::event::read() {
                    Ok(Event::Resize(_, _)) => Ok(UiEvent::Redraw),
                    Ok(Event::Key(key)) => Ok(UiEvent::Key(key.into())),
                    Ok(Event::Mouse(_)) => Ok(UiEvent::Redraw),
                    Err(err) => Err(err),
                };

                let is_err = event.is_err();
                tx.send(event).ok();

                // Dispatch a nop to the main thread to ensure we stop
                // waiting and check for events
                dispatcher.spawn(|_| ()).background();

                if is_err {
                    break;
                }
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
            return clone_io_error(pending);
        }

        match self.events.recv_timeout(timeout) {
            Ok(received) => {
                let clone = clone_io_error(&received);
                self.pending_events.push_front(received);
                clone
            }
            _ => Ok(None),
        }
    }

    fn next_event(&mut self) -> io::Result<UiEvent> {
        if let Some(pending) = self.pending_events.pop_front() {
            return pending;
        }

        self.events.recv().unwrap()
    }
}

fn clone_io_error(result: &io::Result<UiEvent>) -> io::Result<Option<UiEvent>> {
    match result {
        Ok(ev) => Ok(Some(ev.clone())),
        Err(e) => {
            if let Some(code) = e.raw_os_error() {
                Err(io::Error::from_raw_os_error(code))
            } else if e.get_ref().is_some() {
                Err(io::Error::new(e.kind(), e.to_string()))
            } else {
                Err(e.kind().into())
            }
        }
    }
}
