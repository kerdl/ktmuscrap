//! # I DON'T CARE ABOUT CLONING, NIGGERS
//! # "UOOOGH BRO!! YOU COULD USE &'a EVERYWHERE IN YOUR CODE!! UOOOGH!!!!!"
//! # SHUT UP BITCH, I'VE HAD ENOUGH, GO FUCK YOUR &'aSS


use derive_new::new;
use async_trait::async_trait;
use serde::Serialize;
use std::{collections::{hash_map::DefaultHasher}, hash::{Hash, Hasher}};

pub mod schedule;


#[async_trait]
pub trait DetailedCmp<ToCompare, Compared> {
    async fn compare(old: Option<ToCompare>, new: Option<ToCompare>) -> Compared;
}

pub trait Name {
    fn name(&self) -> String;
}


#[derive(Debug, Clone, Serialize)]
pub struct DetailedChanges<Primary, Detailed> {
    pub appeared:    Vec<Primary>,
    pub disappeared: Vec<Primary>,
    pub changed:     Vec<Detailed>,
}
impl<Primary, Detailed> DetailedChanges<Primary, Detailed> 
where 
    Primary: Hash + PartialEq + Clone,
    Detailed: DetailedCmp<Primary, Detailed>
{
    pub async fn compare(
        mut old: Option<Vec<Primary>>,
        mut new: Option<Vec<Primary>>
    ) -> DetailedChanges<Primary, Detailed> {
        let mut appeared:    Vec<Primary> = vec![];
        let mut disappeared: Vec<Primary> = vec![];
        let mut changed:     Vec<Detailed> = vec![];

        match (&mut old, &mut new) {
            (None, None) => {
                return DetailedChanges {
                    appeared,
                    disappeared,
                    changed,
                }
            },
            (None, Some(new)) => {
                appeared.append(new);

                return DetailedChanges {
                    appeared,
                    disappeared,
                    changed,
                }
            },
            (Some(old), None) => {
                disappeared.append(old);

                return DetailedChanges {
                    appeared,
                    disappeared,
                    changed,
                }
            },
            _ => ()
        }

        for old_value in old.as_ref().unwrap().iter() {
            let new_value = new.as_ref().unwrap().iter().find(
                |new_value| new_value == &old_value
            );

            if new_value.is_none() {
                disappeared.push(old_value.clone());
                continue;
            }
        }

        for new_value in new.as_ref().unwrap().iter() {
            let old_value = old.as_ref().unwrap().iter().find(
                |old_value| old_value == &new_value
            );
            
            if old_value.is_none() {
                appeared.push(new_value.clone());
                continue;
            }

            let detailed = Detailed::compare(
                old_value.cloned(),
                Some(new_value.clone())
            ).await;

            let primitive = Primitive::new(
                old_value.cloned(),
                Some(new_value.clone())
            );

            if !primitive.is_same_hash() {
                changed.push(detailed);
                continue;
            }
        }

        DetailedChanges {
            appeared,
            disappeared,
            changed,
        }
    }

    pub fn has_changes(&self) -> bool {
        !self.appeared.is_empty()
        || !self.changed.is_empty()
        || !self.disappeared.is_empty()
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
    pub async fn compare(
        mut old: Option<Vec<Primary>>,
        mut new: Option<Vec<Primary>>
    ) -> Changes<Primary> {
        let mut appeared:    Vec<Primary> = vec![];
        let mut disappeared: Vec<Primary> = vec![];
        let mut changed:     Vec<Primary> = vec![];
        let mut unchanged:   Vec<Primary> = vec![];

        match (&mut old, &mut new) {
            (None, None) => {
                return Changes {
                    appeared,
                    disappeared,
                    changed,
                    unchanged
                }
            },
            (None, Some(new)) => {
                appeared.append(new);

                return Changes {
                    appeared,
                    disappeared,
                    changed,
                    unchanged
                }
            },
            (Some(old), None) => {
                disappeared.append(old);

                return Changes {
                    appeared,
                    disappeared,
                    changed,
                    unchanged
                }
            },
            _ => ()
        }

        for old_value in old.as_ref().unwrap().iter() {
            let new_value = new.as_ref().unwrap().iter().find(
                |new_value| new_value == &old_value
            );

            if new_value.is_none() {
                disappeared.push(old_value.clone());
                continue;
            }
        }

        for new_value in new.as_ref().unwrap().iter() {
            let old_value = old.as_ref().unwrap().iter().find(
                |old_value| old_value == &new_value
            );
            
            if old_value.is_none() {
                appeared.push(new_value.clone());
                continue;
            }

            let primitive = Primitive::new(
                old_value.cloned(),
                Some(new_value.clone())
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

    pub fn any_changes(&self) -> bool {
        [&self.appeared, &self.disappeared, &self.changed].iter().any(
            |change| !change.is_empty()
        )
    }
}


#[derive(new, Debug, Clone, Serialize)]
pub struct Primitive<T> {
    pub old: Option<T>,
    pub new: Option<T>
}
impl<T: PartialEq> Primitive<T> {
    pub fn is_same_eq(&self) -> bool {
        match (&self.old, &self.new) {
            (Some(old), Some(new)) => old == new,
            (None, None) => true,
            _ => false,
        }
    }

    pub fn is_different_eq(&self) -> bool {
        !self.is_same_eq()
    }
}
impl<T: Hash> Primitive<T> {
    pub fn is_same_hash(&self) -> bool {
        match (&self.old, &self.new) {
            (Some(old), Some(new)) => {
                let mut old_hasher = DefaultHasher::new();
                let mut new_hasher = DefaultHasher::new();

                old.hash(&mut old_hasher);
                new.hash(&mut new_hasher);

                old_hasher.finish() == new_hasher.finish()
            },
            (None, None) => true,
            _ => false,
        }
    }

    pub fn is_different_hash(&self) -> bool {
        !self.is_same_hash()
    }
}
