use rand::{SeedableRng, Rng, thread_rng, XorShiftRng};

pub fn subtract_vector<T: Eq>(vs: &mut Vec<T>, s: &Vec<T>) {
    for x in s.iter() {
        let idx = vs.iter().position(|v| *v == *x).expect("Unable to find index");
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
