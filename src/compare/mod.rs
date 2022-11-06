//! # I DON'T CARE ABOUT CLONING, NIGGERS
//! # "UOOOGH BRO!! YOU COULD USE &'a EVERYWHERE IN YOUR CODE!! UOOOGH!!!!!"
//! # SHUT UP BITCH, I'VE HAD ENOUGH, GO FUCK YOUR &'aSS


use derive_new::new;
use serde::Serialize;
use std::{collections::{hash_map::DefaultHasher}, hash::{Hash, Hasher}};

pub mod schedule;


pub trait DetailedCmp<ToCompare, Compared> {
    fn compare(old: ToCompare, new: ToCompare) -> Compared;
}


#[derive(Debug, Clone, Serialize)]
pub struct DetailedChanges<Primary, Detailed> {
    pub appeared:    Vec<Primary>,
    pub disappeared: Vec<Primary>,
    pub changed:     Vec<Detailed>,
    pub unchanged:   Vec<Detailed>,
}
impl<Primary, Detailed> DetailedChanges<Primary, Detailed> 
where 
    Primary: Hash + PartialEq + Clone,
    Detailed: DetailedCmp<Primary, Detailed>
{
    pub fn compare(
        old: Vec<Primary>,
        new: Vec<Primary>
    ) -> DetailedChanges<Primary, Detailed> {

        let mut appeared:    Vec<Primary> = vec![];
        let mut disappeared: Vec<Primary> = vec![];
        let mut changed:     Vec<Detailed> = vec![];
        let mut unchanged:   Vec<Detailed> = vec![];

        for old_value in old.iter() {
            let new_value = new.iter().find(
                |new_value| new_value == &old_value
            );

            if new_value.is_none() {
                disappeared.push(old_value.clone());
                continue;
            }
        }

        for new_value in new.iter() {
            let old_value = old.iter().find(
                |old_value| old_value == &new_value
            );
            
            if old_value.is_none() {
                appeared.push(new_value.clone());
                continue;
            }

            let detailed = Detailed::compare(
                old_value.unwrap().clone(),
                new_value.clone()
            );

            let primitive = Primitive::new(
                old_value.unwrap(),
                new_value
            );

            if primitive.is_same_hash() {
                unchanged.push(detailed);
                continue;
            } else {
                changed.push(detailed);
                continue;
            }
        }

        DetailedChanges {
            appeared,
            disappeared,
            changed,
            unchanged
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Changes<Primary> {
    pub appeared:    Vec<Primary>,
    pub disappeared: Vec<Primary>,
    pub changed:     Vec<Primary>,
    pub unchanged:   Vec<Primary>,
}
impl<Primary> Changes<Primary> 
where 
    Primary: Hash + PartialEq + Clone
{
    pub fn compare(
        old: Vec<Primary>,
        new: Vec<Primary>
    ) -> Changes<Primary> {

        let mut appeared:    Vec<Primary> = vec![];
        let mut disappeared: Vec<Primary> = vec![];
        let mut changed:     Vec<Primary> = vec![];
        let mut unchanged:   Vec<Primary> = vec![];

        for old_value in old.iter() {
            let new_value = new.iter().find(
                |new_value| new_value == &old_value
            );

            if new_value.is_none() {
                disappeared.push(old_value.clone());
                continue;
            }
        }

        for new_value in new.iter() {
            let old_value = old.iter().find(
                |old_value| old_value == &new_value
            );
            
            if old_value.is_none() {
                appeared.push(new_value.clone());
                continue;
            }

            let primitive = Primitive::new(
                old_value.unwrap(),
                new_value
            );

            if primitive.is_same_hash() {
                unchanged.push(new_value.clone());
                continue;
            } else {
                changed.push(new_value.clone());
                continue;
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


#[derive(new, Debug, Clone, Serialize)]
pub struct Primitive<T> {
    pub old: T,
    pub new: T
}
impl<T: PartialEq> Primitive<T> {
    pub fn is_same_eq(&self) -> bool {
        self.old == self.new
    }

    pub fn is_different_eq(&self) -> bool {
        !self.is_same_eq()
    }
}
impl<T: Hash> Primitive<T> {
    pub fn is_same_hash(&self) -> bool {
        let mut old_hasher = DefaultHasher::new();
        let mut new_hasher = DefaultHasher::new();

        self.old.hash(&mut old_hasher);
        self.new.hash(&mut new_hasher);

        old_hasher.finish() == new_hasher.finish()
    }

    pub fn is_different_hash(&self) -> bool {
        !self.is_same_hash()
    }
}
