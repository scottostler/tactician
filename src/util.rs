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
