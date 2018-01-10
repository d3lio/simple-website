use std::sync::atomic::{AtomicUsize, Ordering};

pub type Id = usize;

pub struct IdPool {
    id: AtomicUsize
}

impl IdPool {
    pub fn new() -> Self {
        Self {
            id: AtomicUsize::new(0)
        }
    }

    pub fn next(&self) -> Id {
        self.id.fetch_add(1, Ordering::SeqCst);
        self.id.load(Ordering::SeqCst)
    }
}
