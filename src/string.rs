use rand::distributions::{Alphanumeric, DistString};

pub fn random(len: usize) -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), len)
}