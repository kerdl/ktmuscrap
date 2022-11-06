use derive_new::new;
use std::{collections::{hash_map::DefaultHasher}, hash::{Hash, Hasher}};

pub mod schedule;


pub trait DetailedCmp<ToCompare, Compared> {
    fn compare(old: ToCompare, new: ToCompare) -> Compared;
}


#[derive(Debug)]
pub struct DetailedChanges<Primary, Detailed> {
    pub appeared:    Vec<Primary>,
    pub disappeared: Vec<Primary>,
    pub changed:     Vec<Detailed>,
    pub unchanged:   Vec<Detailed>,
}
impl<'a, Primary, Detailed> DetailedChanges<&'a Primary, &'a Detailed> 
where 
    Primary: Hash + PartialEq,
    Detailed: DetailedCmp<&'a Primary, Detailed>
{
    pub fn compare(
        old: &'a Vec<Primary>,
        new: &'a Vec<Primary>
    ) -> DetailedChanges<&'a Primary, Detailed> {

        let mut appeared:    Vec<&Primary> = vec![];
        let mut disappeared: Vec<&Primary> = vec![];
        let mut changed:     Vec<Detailed> = vec![];
        let mut unchanged:   Vec<Detailed> = vec![];

        for old_value in old.iter() {
            let new_value = new.iter().find(
                |new_value| new_value == &old_value
            );

            if new_value.is_none() {
                disappeared.push(old_value);
                continue;
            }
        }

        for new_value in new.iter() {
            let old_value = old.iter().find(
                |old_value| old_value == &new_value
            );
            
            if old_value.is_none() {
                appeared.push(new_value);
                continue;
            }

            let detailed = Detailed::compare(
                old_value.unwrap(),
                new_value
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

#[derive(Debug)]
pub struct Changes<Primary> {
    pub appeared:    Vec<Primary>,
    pub disappeared: Vec<Primary>,
    pub changed:     Vec<Primary>,
    pub unchanged:   Vec<Primary>,
}
impl<'a, Primary> Changes<&'a Primary> 
where 
    Primary: Hash + PartialEq
{
    pub fn compare(
        old: &'a Vec<Primary>,
        new: &'a Vec<Primary>
    ) -> Changes<&'a Primary> {

        let mut appeared:    Vec<&Primary> = vec![];
        let mut disappeared: Vec<&Primary> = vec![];
        let mut changed:     Vec<&Primary> = vec![];
        let mut unchanged:   Vec<&Primary> = vec![];

        for old_value in old.iter() {
            let new_value = new.iter().find(
                |new_value| new_value == &old_value
            );

            if new_value.is_none() {
                disappeared.push(old_value);
                continue;
            }
        }

        for new_value in new.iter() {
            let old_value = old.iter().find(
                |old_value| old_value == &new_value
            );
            
            if old_value.is_none() {
                appeared.push(new_value);
                continue;
            }

            let primitive = Primitive::new(
                old_value.unwrap(),
                new_value
            );

            if primitive.is_same_hash() {
                unchanged.push(new_value);
                continue;
            } else {
                changed.push(new_value);
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


#[derive(new, Debug)]
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
