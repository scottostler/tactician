use rand::{SeedableRng, Rng, thread_rng, XorShiftRng};
use std;

pub fn subtract_vector<T: std::fmt::Display + Eq>(vs: &mut Vec<T>, s: &Vec<T>) {
    for x in s.iter() {
        let idx = match vs.iter().position(|v| *v == *x) {
            Some(idx) => idx,
            None => panic!("Unable to find index for {}", x)
        };
        vs.remove(idx);
    }
}

pub fn randomly_seeded_weak_rng() -> XorShiftRng {
    let mut base_rng = thread_rng();
    let seed = &[
        base_rng.gen::<u32>(),
        base_rng.gen::<u32>(),
        base_rng.gen::<u32>(),
        base_rng.gen::<u32>()];
    XorShiftRng::from_seed(*seed)
}
