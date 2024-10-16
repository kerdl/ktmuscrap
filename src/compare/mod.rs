//! # I DON'T CARE ABOUT CLONING, NIGGERS
//! # "UOOOGH BRO!! YOU COULD'VE USED &'a EVERYWHERE IN YOUR CODE!! UOOOGH!!!!!"
//! # SHUT UP BITCH, I'VE HAD ENOUGH, GO FUCK YOUR &'aSS


pub mod schedule;

use derive_new::new;
use serde::Serialize;
use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}, ops::ControlFlow};


pub trait DetailedCmp<ToCompare, Compared> {
    fn compare(old: Option<ToCompare>, new: Option<ToCompare>) -> impl std::future::Future<Output = Compared> + Send;
}

/// # Partially compares the struct
/// Used to find a corresponding one from another list
pub trait FindingCmp {
    fn is_partially_same_with(&self, other: &Self) -> bool;
}


fn option_check<Primary>(
    old: &mut Option<Vec<Primary>>,
    new: &mut Option<Vec<Primary>>,
    appeared: &mut Vec<Primary>,
    disappeared: &mut Vec<Primary>,
) -> ControlFlow<()> {
    match (old, new) {
        (None, None) => {
            return ControlFlow::Break(())
        },
        (None, Some(new)) => {
            appeared.append(new);
            return ControlFlow::Break(())
        },
        (Some(old), None) => {
            disappeared.append(old);
            return ControlFlow::Break(())
        },
        _ => ControlFlow::Continue(())
    }    
}

fn find<'a, Primary>(
    list: &'a mut Vec<Primary>,
    value: &Primary,
    ignored_idx: &'a mut Vec<usize>
) -> Option<&'a Primary>
where Primary: FindingCmp + std::fmt::Debug
{
    list.iter().enumerate().find(
        |(index, list_value)| {
            if list_value.is_partially_same_with(value) && !ignored_idx.contains(index) {
                ignored_idx.push(*index);
                true
            } else {
                false
            }
        }
    ).map(
        |(_index, list_value)| list_value
    )
}

fn disappeared_lookup<Primary>(
    old: &mut Vec<Primary>,
    new: &mut Vec<Primary>,
    disappeared: &mut Vec<Primary>
)
where Primary: FindingCmp + Clone + std::fmt::Debug
{
    let mut new_ignored_idx: Vec<usize> = vec![];

    for old_value in old.iter() {
        let new_value = find(new, old_value, &mut new_ignored_idx);

        if new_value.is_none() {
            disappeared.push(old_value.clone());
            continue;
        }
    }
}


#[derive(Debug, Clone, Serialize)]
pub struct DetailedChanges<Primary, Detailed> {
    pub appeared: Vec<Primary>,
    pub disappeared: Vec<Primary>,
    pub changed: Vec<Detailed>,
}
impl<Primary, Detailed> DetailedChanges<Primary, Detailed> 
where 
    Primary: Hash + FindingCmp + Clone + std::fmt::Debug,
    Detailed: DetailedCmp<Primary, Detailed>
{
    pub async fn compare(
        mut old: Option<Vec<Primary>>,
        mut new: Option<Vec<Primary>>
    ) -> DetailedChanges<Primary, Detailed> {
        let mut appeared: Vec<Primary> = vec![];
        let mut disappeared: Vec<Primary> = vec![];
        let mut changed: Vec<Detailed> = vec![];

        match option_check(&mut old, &mut new, &mut appeared, &mut disappeared) {
            ControlFlow::Break(_) => {
                return DetailedChanges {
                    appeared,
                    disappeared,
                    changed,
                }
            }
            ControlFlow::Continue(_) => ()
        }

        disappeared_lookup(old.as_mut().unwrap(), new.as_mut().unwrap(), &mut disappeared);

        let mut old_ignored_idx: Vec<usize> = vec![];

        for new_value in new.as_ref().unwrap().iter() {
            let old_value = find(old.as_mut().unwrap(), new_value, &mut old_ignored_idx);
            
            if old_value.is_none() {
                appeared.push(new_value.clone());
                continue;
            }

            let primitive = Primitive::new(
                old_value.cloned(),
                Some(new_value.clone())
            );

            if primitive.is_different_hash() {
                let detailed = Detailed::compare(
                    old_value.cloned(),
                    Some(new_value.clone())
                ).await;

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
    pub appeared: Vec<Primary>,
    pub disappeared: Vec<Primary>,
    pub changed: Vec<Primary>,
}
impl<Primary> Changes<Primary> 
where 
    Primary: Hash + FindingCmp + Clone + std::fmt::Debug
{
    pub async fn compare(
        mut old: Option<Vec<Primary>>,
        mut new: Option<Vec<Primary>>
    ) -> Changes<Primary> {
        let mut appeared: Vec<Primary> = vec![];
        let mut disappeared: Vec<Primary> = vec![];
        let mut changed: Vec<Primary> = vec![];

        match option_check(&mut old, &mut new, &mut appeared, &mut disappeared) {
            ControlFlow::Break(_) => {
                return Changes {
                    appeared,
                    disappeared,
                    changed,
                }
            }
            ControlFlow::Continue(_) => ()
        }

        disappeared_lookup(old.as_mut().unwrap(), new.as_mut().unwrap(), &mut disappeared);

        let mut old_ignored_idx: Vec<usize> = vec![];

        for new_value in new.as_ref().unwrap().iter() {
            let old_value = find(old.as_mut().unwrap(), new_value, &mut old_ignored_idx);
            
            if old_value.is_none() {
                appeared.push(new_value.clone());
                continue;
            }

            let primitive = Primitive::new(
                old_value.cloned(),
                Some(new_value.clone())
            );

            if primitive.is_different_hash() {
                changed.push(new_value.clone());
                continue;
            }
        }

        Changes {
            appeared,
            disappeared,
            changed
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
