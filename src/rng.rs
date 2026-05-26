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

        x ^= x.wrapping_shl(13 % bits);
        x ^= x.wrapping_shr(7 % bits);
        x ^= x.wrapping_shl(17 % bits);

        self.state = x;
        x
    }

    pub fn next_bound(&mut self, n: usize) -> usize {
        assert!(n > 0, "next_bound requires n > 0");
        // Rejection sampling: discard values in the biased remainder region so
        // every bucket [0, n) is equally likely.
        let threshold = usize::MAX - (usize::MAX % n);
        loop {
            let x = self.next();
            if x <= threshold {
                return x % n;
            }
        }
    }
}
