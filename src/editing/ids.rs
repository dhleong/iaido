use super::Id;

pub const BUFFER_ID_LOG: Id = 0;
pub const FIRST_USER_BUFFER_ID: Id = 1;

pub struct Ids {
    next: usize,
    initial: usize,
}

impl Ids {
    pub fn new() -> Self {
        Self::with_first(0)
    }

    pub fn with_first(next: Id) -> Self {
        Self {
            next,
            initial: next,
        }
    }

    pub fn most_recent(&self) -> Option<Id> {
        if self.next <= self.initial {
            None
        } else {
            Some(self.next - 1)
        }
    }

    pub fn next(&mut self) -> Id {
        let n = self.next;
        self.next += 1;
        return n;
    }
}
