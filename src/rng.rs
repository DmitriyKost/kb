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

        let seed = if seed == 0 { 1 } else { seed };

        Self { state: seed }
    }

    pub fn next(&mut self) -> usize {
        let mut x = self.state;
        let bits = usize::BITS;

        x ^= x.wrapping_shl((13 % bits) as u32);
        x ^= x.wrapping_shr((7 % bits) as u32);
        x ^= x.wrapping_shl((17 % bits) as u32);

        self.state = x;
        x
    }

    pub fn next_bound(&mut self, n: usize) -> usize {
        self.next() % n
    }
}
