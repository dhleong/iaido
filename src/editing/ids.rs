use super::Id;

pub const BUFFER_ID_LOG: Id = 0;

pub struct Ids {
    next: usize,
}

impl Ids {
    pub fn new() -> Self {
        Self { next: 0 }
    }

    pub fn with_first(next: Id) -> Self {
        Self { next }
    }

    pub fn next(&mut self) -> Id {
        let n = self.next;
        self.next += 1;
        return n;
    }
}
