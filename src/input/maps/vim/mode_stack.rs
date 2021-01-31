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
    pub fn push(&mut self, new_mode: VimMode) {
        self.stack.push(new_mode.id.clone());
        self.modes.insert(new_mode.id.clone(), new_mode);
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    pub fn return_top(&mut self, mode: VimMode) {
        if self.stack.contains(&mode.id) {
            self.modes.insert(mode.id.clone(), mode);
        }
    }

    pub fn take_top(&mut self) -> Option<VimMode> {
        if let Some(id) = self.stack.last() {
            Some(
                self.modes
                    .remove(id)
                    .expect(&format!("Top of stack mode {} not found", id)),
            )
        } else {
            None
        }
    }
}
