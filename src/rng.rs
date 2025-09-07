use std::time::{SystemTime, UNIX_EPOCH};

pub struct XorShift {
    state: usize,
}

impl XorShift {
    pub fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos() as usize;
        assert!(
            seed != 0,
            "Internal error getting system time, seed must be non-zero!"
        );
        Self { state: seed }
    }

    pub fn next(&mut self) -> usize {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    pub fn next_bound(&mut self, n: usize) -> usize {
        self.next() % n
    }
}
