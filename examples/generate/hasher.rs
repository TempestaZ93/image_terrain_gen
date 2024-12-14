use std::hash::Hasher;

pub struct SeedHasher {
    hash: u64,
    p: u64,
}

impl SeedHasher {
    pub fn new() -> Self {
        Self {
            hash: 99876516661,
            p: 779126527,
        }
    }
}

impl Hasher for SeedHasher {
    fn finish(&self) -> u64 {
        self.hash
    }

    fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.hash = (self.hash ^ *b as u64).wrapping_mul(self.p);
        }
    }
}
