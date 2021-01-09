use super::Id;

pub struct Ids {
    next: usize,
}

impl Ids {
    pub fn new() -> Self {
        Self { next: 0 }
    }

    pub fn next(&mut self) -> Id {
        let n = self.next;
        self.next += 1;
        return n;
    }
}
