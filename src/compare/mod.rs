use std::{
    hash::{Hash, Hasher},
    collections::hash_map::DefaultHasher
};

use derive_new::new;
use itertools::Itertools;

pub mod schedule;


pub struct Changes<T> {
    pub appeared: Vec<T>,
    pub disappeared: Vec<T>,
    pub changed: Vec<T>,
    pub unchanged: Vec<T>
}
impl<T: Hash> Changes<T> {
    pub fn compare(old: Vec<T>, new: Vec<T>) -> Changes<T> {

        let mut appeared:    Vec<T> = vec![];
        let mut disappeared: Vec<T> = vec![];
        let mut changed:     Vec<T> = vec![];
        let mut unchanged:   Vec<T> = vec![];

        for new_value in new.iter() {
            for old_value in old.iter() {
                
            }
        }

        Changes {
            appeared,
            disappeared,
            changed,
            unchanged
        }
    }
}

#[derive(new)]
pub struct Compare<T> {
    pub old: T,
    pub new: T
}
impl<T: Hash> Compare<T> {
    pub fn is_same(&self) -> bool {
        let mut old_hasher = DefaultHasher::new();
        let mut new_hasher = DefaultHasher::new();

        self.old.hash(&mut old_hasher);
        self.new.hash(&mut new_hasher);

        old_hasher.finish() == new_hasher.finish()
    }

    pub fn is_different(&self) -> bool {
        !self.is_same()
    }
}
