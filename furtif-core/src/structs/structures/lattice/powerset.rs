use std::{ 
    collections::{ HashMap, BTreeMap, hash_map, }, vec,
};
use core::fmt::Debug;

use hashed_type_def::{ HashedTypeDef, add_hash_fnv1a, };
// #[cfg(feature = "silx-types")] use silx_types::{ u128slx, IntoSlx, SlxInto, f64slx, };
// #[cfg(not(feature = "silx-types"))] use crate::fake_slx::{ f64slx, u128slx, FakeSlx, };

use crate::types::{ u128slx, f64slx, SlxInto, IntoSlx, };


#[cfg(feature = "serde")] use serde::{Serialize as SerdeSerialize, Deserialize as SerdeDeserialize};
#[cfg(feature = "rkyv")] use rkyv::{Archive, Serialize as RkyvSerialize, Deserialize as RkyvDeserialize};
// This program is free software: you can redistribute it and/or modify
// it under the terms of the Lesser GNU General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// Lesser GNU General Public License for more details.

// You should have received a copy of the Lesser GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// Copyright 2024 Frederic Dambreville, Jean Dezert Developers.


use rand::prelude::*;

use crate::{
    traits::{ Lattice, ComplementedLattice, IterableLattice, LatticeWithLeaves }, 
    structs::SafeElement,
};

const DEFAULT_MAX_ITER_LEN : usize = 1024;

#[derive(Clone, Debug, HashedTypeDef)]
#[cfg_attr(feature = "rkyv", derive(Archive,RkyvSerialize,RkyvDeserialize))]
/// Powerset lattice
pub struct Powerset {
    max_iter_len: u128slx,
    top: SafeElement<u128slx>,
    bottom: SafeElement<u128slx>,
    tags: BTreeMap<u128slx,String,>,
    untags: HashMap<String,u128slx,>,
    leaves: Vec<u128slx>,
    weighted_leaves: HashMap<u128slx,f64slx>,
    bottom_to_top: Option<Vec<u128slx>>,
}

// implementation of Serde serialization
#[cfg(feature = "serde")] mod serding {
    // #[cfg(feature = "silx-types")] use silx_types::SlxInto;
    // #[cfg(not(feature = "silx-types"))] use crate::fake_slx::FakeSlx;

    use super::{ 
        Powerset as SerdingPowerset, SerdeSerialize, SerdeDeserialize, SlxInto,
    };
    #[derive(SerdeSerialize,SerdeDeserialize)]
    pub struct Powerset {
        nb_leaves: usize, max_iter_len: usize,
    }
    impl<'de> SerdeDeserialize<'de> for SerdingPowerset {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de> {
            let Powerset { nb_leaves, max_iter_len } = Powerset::deserialize(deserializer)?;
            match SerdingPowerset::new(nb_leaves, max_iter_len) {
                Ok(p) => Ok(p),
                Err(_) => Ok(Self::empty()),
            }
        }
    }
    impl SerdeSerialize for SerdingPowerset {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer {
            let SerdingPowerset { max_iter_len, leaves, .. } = self;
            let nb_leaves = leaves.len();
            let max_iter_len = (*max_iter_len).unslx() as usize;
            let powerset = Powerset { nb_leaves, max_iter_len };
            powerset.serialize(serializer)
        }
    }
}

impl Powerset {
    /// Powerset constructor
    /// * Leaves labels are generated automatically
    /// * `nb_leaves: usize` : number of leaves
    /// * `max_iter_len: usize` : maximal size for an iterator on the powerset
    /// * Output: the powerset or an error, when the number of leaves exceeds 128
    pub fn new(nb_leaves: usize, max_iter_len: usize,) -> Result<Powerset,String> {
        let leaves_names = (0..nb_leaves).map(|u| format!("U{u}")).collect::<Vec<_>>();
        Self::new_with_label(&leaves_names, max_iter_len)
    }
    /// Powerset constructor with predefined leaves labels
    /// * Leaves labels are provided
    ///   * The labels should be different: if this condition is not met, this should not affect the computations, but it would make the results less readable
    /// * `leaves_names: &[String]` : list of leaves described by their names
    /// * `max_iter_len: usize` : maximal size for an iterator on the powerset
    /// * Output: the powerset or an error, when the number of leaves exceeds 128
    pub fn new_with_label(leaves_names: &[String], max_iter_len: usize,) -> Result<Powerset,String> {
        let nb_leaves = leaves_names.len();
        let s = std::mem::size_of::<u128>() << 3; // number of bits of usize
        let top = match nb_leaves { //
            m if m > s     => Err(format!("Number of leaves cannot excess {s}")), // nb of bits of u128 is exceeded
            m if m == s    => Ok(u128::MAX.slx()),
            m              => Ok(((1u128 << m)-1u128).slx()),
        };
        top.map(|top| {
            let leaves: Vec<u128slx> = (0..nb_leaves).map(|rank|(1u128 << rank).slx()).collect::<Vec<_>>();
            let (tags,untags) = leaves_names.iter().enumerate()
                .map(|(rank,label)| ((leaves[rank],label.clone()),(label.clone(),leaves[rank])))
                .unzip::<_,_,BTreeMap<_,_>,HashMap<_,_>>();
            let unif: f64slx = (nb_leaves as f64).recip().slx();
            let weighted_leaves = leaves.iter().map(|k| (*k,unif)).collect::<HashMap<_,_>>();
            let mut sorted_tags = tags.iter().map(
                |(u,s)| ((*u).unslx(),s)
            ).collect::<Vec<_>>(); sorted_tags.sort_by_key(|(k,_)| *k);
            let mut sorted_leaves = weighted_leaves.iter().map(
                |(u,w)| ((*u).unslx(),(*w).unslx())
            ).collect::<Vec<_>>(); sorted_leaves.sort_by_key(|(k,_)| *k);
            let lattice_hash = {
                let mut lattice_hash = Powerset::TYPE_HASH_NATIVE;
                lattice_hash = add_hash_fnv1a(&nb_leaves.to_le_bytes(), lattice_hash);
                for (u,s) in &sorted_tags {
                    lattice_hash = add_hash_fnv1a(&(*u).to_le_bytes(), lattice_hash);
                    lattice_hash = add_hash_fnv1a(s.as_bytes(), lattice_hash);
                }
                for (u,w) in &sorted_leaves {
                    lattice_hash = add_hash_fnv1a(&(*u).to_le_bytes(), lattice_hash);
                    lattice_hash = add_hash_fnv1a(&(*w).to_le_bytes(), lattice_hash);
                }
                lattice_hash.slx()
            };
            let max_iter_len = (max_iter_len as u128).slx();
            let bottom = SafeElement { code: 0u128.slx(), lattice_hash, };
            let top = SafeElement { code: top, lattice_hash, };
            Powerset { bottom, top, leaves, weighted_leaves, tags, untags, max_iter_len, bottom_to_top: None, }
        })
    }

    #[cfg(feature = "serde")] 
    /// Internal use for serde: empty Powerset
    fn empty() -> Powerset {
        let zero = 0u128.slx();
        let top = SafeElement{ code: zero, lattice_hash: zero };
        let bottom = top;
        Powerset {
            max_iter_len: zero, top, bottom, tags: BTreeMap::new(), untags: HashMap::new(), 
            leaves: Vec::new(), weighted_leaves: HashMap::new(), bottom_to_top: None,
        }
    }

    /// Internal use: for building interator
    fn build_double_sequence_up(&self) -> Vec<Vec<u128slx>> {
        // sequence for nb_usize == 0 --> degenerated powerset
        let mut double_sequence = vec![vec![0u128.slx()]];
        for (n,(leaf,_)) in self.weighted_leaves.iter().enumerate() {
            let mut next_ds = Vec::with_capacity(n+1);
            next_ds.push(vec![0u128.slx()]);
            for k in 0..n {
                next_ds.push(double_sequence[k+1].iter().copied().chain(
                    double_sequence[k].iter().copied().map(|u| u | *leaf)
                ).collect());
            }
            next_ds.push(vec![double_sequence[n][0] | *leaf]);
            double_sequence = next_ds;
        }
        double_sequence
    }

    /// Implement powerset iterators with a view to use methods `IterableLattice::unsafe_bottom_to_top` and `IterableLattice::unsafe_top_to_bottom`
    /// * These iterators are not defined by the powerset constructor due to the amount of resources required for some large powersets
    /// * Output: powerset implementing the iterators
    pub fn set_iterators(mut self) -> Self {
        if self.top.code < self.max_iter_len && self.bottom_to_top.is_none() {
            self.bottom_to_top = Some(
                self.build_double_sequence_up().into_iter().flat_map(|v|v).collect()
            );
        } self
    }
}

impl Lattice for Powerset {
    type Item = u128slx;

    fn rand_lattice<R: Rng>(rng: &mut R) -> Self {
        let nb_leaves = rng.gen_range(1..=(std::mem::size_of::<u128>() << 3));
        Self::new(nb_leaves,DEFAULT_MAX_ITER_LEN).expect("unexpected: None returned")
    }

    fn rand_element<R: Rng>(&self, rng: &mut R) -> SafeElement<Self::Item> {
        let SafeElement { code: top, lattice_hash } = self.top;
        let top = top.unslx();
        let element = rng.gen_range(0..=top).slx();
        SafeElement { code: element, lattice_hash }
    }

    fn ref_lattice_hash(&self) -> &u128slx { &self.bottom.lattice_hash }

    fn contains(&self, element: &Self::Item) -> bool { element <= &self.top.code }

    fn ref_bottom(&self) -> &SafeElement<Self::Item> { &self.bottom }

    fn ref_top(&self) -> &SafeElement<Self::Item> { &self.top }

    unsafe fn unsafe_meet(&self, element_left: &Self::Item, element_right: &Self::Item) -> Self::Item {
        *element_left & *element_right
    }

    unsafe fn unsafe_join(&self, element_left: &Self::Item, element_right: &Self::Item) -> Self::Item {
        *element_left | *element_right
    }

    fn from_str(&self, s: &str) -> Result<SafeElement<Self::Item>,String> {
        let tokens = s.split('|')
                .map(|s| s.split_whitespace().fold(String::new(),|acc,u| {
            if acc.is_empty() { u.to_string() } else { format!("{acc} {u}") }
        }));
        let SafeElement { code: mut element, lattice_hash } = self.bottom;
        for token in tokens {
            match (&token == "\u{22A5}",&token == "\u{22A4}") {
                (true, true) => panic!("unexpected error: \u{22A5} == \u{22A4}"),
                (true, false) => (), // case where token is bottom
                (false, true) => element = self.top.code, // case where token is top
                (false, false) => match self.untags.get(&token) {
                    Some(l) => element |= *l,
                    None => return Err(format!("leaf {token} is unknown")),
                },
            }
        }
        Ok(SafeElement { code: element, lattice_hash })
    }

    fn to_string(&self, element: &SafeElement<Self::Item>) -> Result<String,String> {
        let SafeElement { code: element, lattice_hash } = element;
        let element = *element;
        if lattice_hash == &self.bottom.lattice_hash {
            match (element == self.bottom.code,element == self.top.code) {
                (true, true) => panic!("unexpected error: \u{22A5} == \u{22A4}"),
                (true, false) => Ok("\u{22A5}".to_string()),
                (false, true) => Ok("\u{22A4}".to_string()),
                (false, false) => {
                    Ok(self.tags.iter().filter(|(l,_)| { let l = **l; (element & l) == l } )
                            .fold(String::new(), |acc,(_,s)| {
                        if acc.is_empty() { s.to_string() } else { format!("{acc} | {s}") }
                    }))
                },
            }
        } else { Err("lattice does not contain element".to_string()) }
    }
}

impl ComplementedLattice for Powerset {
    unsafe fn unsafe_not(&self, element: &Self::Item) -> Self::Item { self.top.code ^ *element }
}

impl IterableLattice for Powerset {
    type IntoIterUp = vec::IntoIter<u128slx>;

    type IntoIterDown = vec::IntoIter<u128slx>;

    unsafe fn unsafe_bottom_to_top(&self) -> Result<Self::IntoIterUp,String> {
        match &self.bottom_to_top {
            Some(btt) => Ok(btt.clone().into_iter()),
            None => Err("Iterator is not set or is exceeding allowed size".to_string()),            
        }
    }

    unsafe fn unsafe_top_to_bottom(&self) -> Result<Self::IntoIterDown,String> {
        match &self.bottom_to_top {
            Some(btt) => Ok(btt.iter().copied().rev().collect::<Vec<_>>().into_iter()),
            None => Err("Iterator is not set or is exceeding allowed size".to_string()),            
        }
    }

}

impl LatticeWithLeaves for Powerset {
    type IntoIterLeaves = hash_map::IntoIter<Self::Item, f64slx>;

    unsafe fn unsafe_leaves(&self) -> Result<Self::IntoIterLeaves,String> {
        let len_slx: u128slx = (self.weighted_leaves.len() as u128).slx();
        if len_slx >= self.max_iter_len {
            Err("Iterator is exceeding allowed size".to_string())
        } else {
            Ok(self.weighted_leaves.clone().into_iter())
        }
    }

    unsafe fn unsafe_leaf(&self, u: usize) -> Result<&Self::Item,String> {
        match self.leaves.get(u) {
            Some(x) => Ok(x),
            None => Err(format!("Leaf of index {u} is not found within lattice")),
        }
    }

    unsafe fn unsafe_weighted_leaf(&self, u: usize) -> Result<(&Self::Item,&f64slx),String> {
        let leaf = self.unsafe_leaf(u)?;
        Ok((leaf,&self.weighted_leaves[leaf]))
    }
}
