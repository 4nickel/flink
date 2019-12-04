use rand::distributions::{Alphanumeric, Standard};
use rand::{self, Rng};

pub fn random_utf8(len: usize) -> String {
    rand::thread_rng()
        .sample_iter::<char, _>(&Standard)
        .take(len)
        .collect()
}

pub fn random_ascii(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .collect()
}
