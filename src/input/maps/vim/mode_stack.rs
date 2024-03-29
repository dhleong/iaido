use std::collections::HashMap;

use super::VimMode;

/// A possibly-overengineered abstraction for managing a stack
/// of possibly-custom Modes
pub struct VimModeStack {
    modes: HashMap<String, VimMode>,
    stack: Vec<String>,
}

impl Default for VimModeStack {
    fn default() -> Self {
        Self {
            modes: HashMap::default(),
            stack: Vec::default(),
        }
    }
}

impl VimModeStack {
    pub fn contains(&self, mode_id: &String) -> bool {
        self.stack.contains(mode_id)
    }

    pub fn push(&mut self, new_mode: VimMode) {
        self.stack.push(new_mode.id.clone());
        self.modes.insert(new_mode.id.clone(), new_mode);
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    pub fn pop_if(&mut self, mode_id: &str) {
        if let Some(last) = self.stack.last() {
            if last == mode_id {
                self.pop();
            }
        }
    }

    pub fn peek(&self) -> Option<&VimMode> {
        if let Some(id) = self.stack.last() {
            self.modes.get(id)
        } else {
            None
        }
    }
}
