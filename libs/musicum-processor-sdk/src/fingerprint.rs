//! Shared hasher facade used by processors (slot keys) and core
//! (prefix fingerprints). One place to swap the hasher if needed.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct Fingerprint(DefaultHasher);

impl Fingerprint {
    pub fn new() -> Self { Self(DefaultHasher::new()) }
    pub fn add(&mut self, v: impl Hash)  { v.hash(&mut self.0); }
    pub fn add_f32(&mut self, v: f32)    { v.to_bits().hash(&mut self.0); }
    pub fn add_f64(&mut self, v: f64)    { v.to_bits().hash(&mut self.0); }
    pub fn finish(self) -> u64           { self.0.finish() }

    pub fn of_f32(v: f32) -> u64 { let mut f = Self::new(); f.add_f32(v); f.finish() }
    pub fn of_f64(v: f64) -> u64 { let mut f = Self::new(); f.add_f64(v); f.finish() }
}

impl Default for Fingerprint {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_input_same_hash() {
        assert_eq!(Fingerprint::of_f32(0.5), Fingerprint::of_f32(0.5));
    }

    #[test]
    fn different_input_different_hash() {
        assert_ne!(Fingerprint::of_f32(0.5), Fingerprint::of_f32(0.6));
    }

    #[test]
    fn add_order_matters() {
        let mut a = Fingerprint::new(); a.add(1u32); a.add(2u32);
        let mut b = Fingerprint::new(); b.add(2u32); b.add(1u32);
        assert_ne!(a.finish(), b.finish());
    }

    #[test]
    fn f64_distinct_from_f32_for_same_value() {
        // We hash bit patterns, so this is expected — pin it so a future
        // hasher swap doesn't silently change semantics.
        assert_ne!(Fingerprint::of_f32(0.5), Fingerprint::of_f64(0.5));
    }
}
