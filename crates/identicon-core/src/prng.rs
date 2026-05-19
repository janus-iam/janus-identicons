use rand_xoshiro::Xoshiro256PlusPlus;
use rand_xoshiro::rand_core::{RngCore, SeedableRng};

pub struct Prng {
    inner: Xoshiro256PlusPlus,
}

impl Prng {
    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self {
            inner: Xoshiro256PlusPlus::from_seed(seed),
        }
    }

    pub fn next_u32(&mut self) -> u32 {
        self.inner.next_u32()
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() as f32) / (u32::MAX as f32)
    }

    pub fn range_u32(&mut self, min: u32, max: u32) -> u32 {
        debug_assert!(min <= max);
        if min == max {
            return min;
        }
        min + (self.next_u32() % (max - min + 1))
    }

    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}
