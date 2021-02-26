use super::Change;

pub struct ChangeManager {
    change_depth: u16,
    current_change: Option<Change>,
    undo_stack: Vec<Change>,
}

impl Default for ChangeManager {
    fn default() -> Self {
        Self {
            change_depth: 0,
            current_change: None,
            undo_stack: Vec::default(),
        }
    }
}
