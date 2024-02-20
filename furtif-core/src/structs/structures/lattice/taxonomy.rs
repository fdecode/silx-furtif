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


use core::fmt::Debug;
use std::{ 
    collections::{ HashSet, HashMap, BTreeMap, hash_map, BTreeSet, },
    vec, iter::once,
};
use hashed_type_def::{HashedTypeDef, add_hash_fnv1a};
use serde::{ Serialize as SerdeSerialize, Deserialize as SerdeDeserialize, };
use rkyv::{ Archive, Serialize as RkyvSerialize, Deserialize as RkyvDeserialize, };
use rand::prelude::*;
use rand_distr::{ Open01, Poisson, Distribution, };
use silx_types::{ u128slx, IntoSlx, SlxInto, f64slx, };

use crate::{traits::{Lattice, IterableLattice, LatticeWithLeaves}, structs::SafeElement};

#[derive(Archive, RkyvSerialize, RkyvDeserialize, HashedTypeDef, Clone, Debug)]
/// Code table for a taxonomy (to be computed By Taxonomy contructor)
pub struct TaxonCoder(HashMap<u128slx,u128slx>);

impl TaxonCoder { // worst case for taxonomy depth depends on the number of subtaxon by taxon.
    // maximum depth is 3 (including top and bottom) when the maximum of taxons is directly put under top 
    //      (2^121+2 taxons, including top and bot and 2^121 subtaxons of top)
    // maximum depth is 129 (including top and bottom) for a binary taxonomy (or for subtaxonomy of a binary one)
    // with a maximum of  2^122 + 1 taxons (including top and bottom)
    // If the number of subtaxons by taxon is limited to 8, then the maximal taxonomy depth should be 42 (2^3=8 and 121/3=40)

    fn new(map: HashMap<u128slx,u128slx>) -> TaxonCoder { TaxonCoder(map) }

    #[inline(always)]
    fn min(a:u8,b:u8,c:u8) -> u8 {
        if a < b {
            if c < a { c }
            else { a }
        } else {
            if c < b { c }
            else { b }
        }
    }
    #[inline(always)]
    fn to_u8u128(w: u128) -> (u8, u128) { (((w>>121) as u8), (w & 0x01FFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFFu128)) }
    #[inline(always)]
    fn to_u128((h,l): (u8, u128)) -> u128 { ((h as u128) << 121) | l }
    #[inline(always)]
    fn to_u128slx((h,l): (u8, u128slx)) -> u128slx {  Self::to_u128((h,l.unslx())).slx() }
    #[inline(always)]
    pub fn above_top_u128() -> u128 { 0x02000000_00000000_00000000_00000000u128 }
    #[inline(always)]
    pub fn above_top_u128slx() -> u128slx { Self::above_top_u128().slx() }
   #[inline(always)]
    pub fn top_u128() -> u128 { 0x01FFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFFu128 }
    #[inline(always)]
    pub fn top_u128slx() -> u128slx { Self::top_u128().slx() }
    #[inline(always)]
    pub fn bottom_u128() -> u128 { 0x7Fu128 << 121 }
    #[inline(always)]
    pub fn bottom_u128slx() -> u128slx { Self::bottom_u128().slx() }
    #[inline(always)]
    fn bottom_u8u128() -> (u8, u128) { (0x7Fu8,0x0u128) }
    #[inline(always)]
    fn meet_u8u128(a: (u8, u128), b: (u8, u128)) -> (u8, u128) {
        match (a,b) {
            ((ah,al), (bh,bl)) if (ah < bh) && (al == bl | (Self::top_u128() >> ah)) => (bh,bl),
            ((ah,al), (bh,bl)) if (ah > bh) && (bl == al | (Self::top_u128() >> bh)) => (ah,al),
            (a, b) if a == b => a,
            _ => Self::bottom_u8u128(),
        } 
    }
    #[inline(always)]
    fn join_u8u128(a: (u8, u128), b: (u8, u128)) -> (u8, u128) {
        match (a,b) {
            ((0x7Fu8,0x0u128),a) => a,
            (a,(0x7Fu8,0x0u128)) => a,
            ((ah,al), (bh,bl)) => {
                let depth_join = Self::min((((al^bl).leading_zeros() as i8) - 7i8) as u8, ah, bh);
                (depth_join, al | Self::top_u128()>>depth_join)        
            }
        }
    }

    /// Meet operator for this taxon coder for native codes
    /// * `a: u128` : left operand
    /// * `b: u128` : right operand
    /// * Output: meet result
    #[inline(always)]
    pub fn meet(&self, a: u128, b: u128) -> u128 { 
        Self::to_u128(Self::meet_u8u128(Self::to_u8u128(a), Self::to_u8u128(b))) 
    }
    /// Join operator for this taxon coder for native codes
    /// * `a: u128` : left operand
    /// * `b: u128` : right operand
    /// * Output: join result
    #[inline(always)]
    pub fn join(&self, a: u128, b: u128) -> u128 {
        let idx = Self::to_u128(Self::join_u8u128(Self::to_u8u128(a), Self::to_u8u128(b))).slx();
        match self.0.get(&idx) { Some(&v) => v, _ => idx, }.unslx()
    }

    /// Meet operator for this taxon coder for slx codes
    /// * `a: u128slx` : left operand
    /// * `b: u128slx` : right operand
    /// * Output: meet result
    #[inline(always)]
    pub fn meet_slx(&self, a: u128slx, b: u128slx) -> u128slx { self.meet(a.unslx(), b.unslx()).slx() }
    /// Join operator for this taxon coder for slx codes
    /// * `a: u128slx` : left operand
    /// * `b: u128slx` : right operand
    /// * Output: join result
    #[inline(always)]
    pub fn join_slx(&self, a: u128slx, b: u128slx) -> u128slx {
        let idx = Self::to_u128(Self::join_u8u128(
            Self::to_u8u128(a.unslx()), Self::to_u8u128(b.unslx())
        )).slx();
        match self.0.get(&idx) { Some(&v) => v, _ => idx, }
    }
    
}

/// Some useful functions for intern computation
trait RandomTaxon {
    const TAXO_NAMES: [&'static str;26]= [
        "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M",
        "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
    ];
    #[inline] fn bot_str() -> &'static str { "\u{22A5}" } 
    #[inline] fn bot_string() -> String { Self::bot_str().to_string() } 
    #[inline] fn max_leaf() -> usize { Self::TAXO_NAMES.len() } 
    #[inline] fn rand_letter<R: Rng>(rng: &mut R) -> &'static str { Self::TAXO_NAMES[rng.gen_range(0..26)] }
    #[inline] fn append(a: &str, b: &str) -> String { format!("{a}{b}") }
    #[inline] fn prefix(a: &str, b: &str) -> String {
        let z = Self::bot_str();
        if a == z { b.to_string() } else { 
            if b == z { a.to_string() } else { 
                a.chars().zip(b.chars()).take_while(|(x, y)| x == y).map(|(x,_)|x).collect() 
            } 
        }
    }
    #[inline] fn meet_string(a:&str, b:&str) -> String {
        let common = a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count();
        if common == a.len() { b.to_string() } else { 
            if common == b.len() { a.to_string() } else { Self::bot_string() } 
        }
    }
} 
impl RandomTaxon for () { }

// Rkyv serialization is not possible here (easily) since the type is recursive
#[derive(Debug,Clone,SerdeSerialize,SerdeDeserialize, PartialEq, PartialOrd,)]
/// Taxonomy builder: definition of a taxonomy by means of main taxon and sorted children
/// * This builder is not suitable for Lattice implementation, and Taxonomy should be inited from it
pub enum TaxonomyBuilder { // a Taxon is characterized by its UNIQUE name
    /// Node case
    Node {
        /// Name of the taxon of this node
        name: String, 
        /// List of children of the taxon of this node
        children: BTreeSet<TaxonomyBuilder>,    
    },
    /// Leaf case
    Leaf {
        /// Name of the taxon of this Leaf
        name: String,
        /// Weight of the Leaf
        /// * This weight is used typically for pignistic probability computation
        weight: f64,
    }
}

impl Eq for TaxonomyBuilder {}

impl Ord for TaxonomyBuilder {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self {
            TaxonomyBuilder::Leaf { name, .. } | TaxonomyBuilder::Node { name, .. } => {
                name.cmp(match other {
                    TaxonomyBuilder::Leaf { name, .. } | TaxonomyBuilder::Node { name, .. } => name,
                })
            }
        }
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, HashedTypeDef, Clone, Debug, PartialEq,)]
/// Taxon code-based definition
/// * For this type, children are mapped by means of an extern code table
/// * For this reason, `Taxon` should be used within type `Taxons`
pub enum Taxon { // a Taxon is characterized by its UNIQUE name
    /// Node case
    Node {
        /// Name of the taxon of this Node
        name: String, 
        /// List of children of the taxon of this node
        children: Vec<u128slx>,    
    },
    Leaf {
        /// Name of the taxon of this Leaf
        name: String,
        /// Weight of the Leaf
        /// * This weight is used typically for pignistic probability computation
        weight: f64slx,
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, HashedTypeDef, Clone, Debug, PartialEq,)]
/// Taxonomy described by a list of taxa and their codes
pub struct Taxons { // a Taxon is characterized by its UNIQUE name
    /// Code table for the taxa
    taxons: HashMap<u128slx,Taxon>,
    /// Code of root taxon
    root: u128slx,
}

#[derive(Clone, Debug, PartialEq,)]
/// taxon with sorted children: for internal computation
enum TaxonOrd<D> {
    Node {
        index: u128,
        name: D, 
        children: BTreeMap<(u8,u128),TaxonOrd<D>>,     
    },
    Leaf {
        index: u128,
        name: D,
        weight: f64,
    }
}


#[derive(Archive, RkyvSerialize, RkyvDeserialize, HashedTypeDef, Clone, Debug,)]
/// Structure of a taxonomy lattice
pub struct Taxonomy {
    taxons: Taxons,
    top: SafeElement<u128slx>,
    bottom: SafeElement<u128slx>,
    coder: TaxonCoder,
    tags: HashMap<u128slx,String,>,
    untags: HashMap<String,u128slx,>,
    leaves: Vec<u128slx>,
    weighted_leaves: HashMap<u128slx,f64slx>,
    top_to_bottom: Vec<u128slx>,
}

// implementation of Serde serialization
mod serding {
    use super::{ 
        Taxonomy as SerdingTaxonomy, SerdeSerialize, SerdeDeserialize, TaxonomyBuilder,
    };
    impl<'de> SerdeDeserialize<'de> for SerdingTaxonomy {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de> {
            let taxons_builder = TaxonomyBuilder::deserialize(deserializer)?;
            match Self::new(&taxons_builder) {
                Ok(taxonomy) => Ok(taxonomy),
                Err{..} => { // build empty taxonomy
                    Ok(Self::empty())
                },
            }
        }
    }
    impl SerdeSerialize for SerdingTaxonomy {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer {
            let taxon_builder = self.into_taxon_builder();
            taxon_builder.serialize(serializer)
        }
    }
}

impl TaxonomyBuilder {
    /// Create a new node of taxonomy builder
    /// * `name: String` : name of the taxon node
    /// * `children: I` : children of the taxon node
    /// * `I` : type of collection
    /// * Output: a taxonomy builder node
    pub fn new_node<I>(name: String, children: I,) -> Result<Self,String> 
                                where I: IntoIterator<Item = TaxonomyBuilder> {
        let children = children.into_iter().collect::<BTreeSet<_>>(); 
        if children.is_empty() {
            Err("Node needs at least one child".to_string())
        } else {
            Ok(Self::Node { name, children, }) 
        }
    }

    /// Create a new leaf of taxonomy builder
    /// * `name: String` : name of the taxon leaf
    /// * `weight: f64` : weight of the taxon leaf
    /// * Output: a taxonomy builder leaf
    pub fn new_leaf(name: String, weight: f64,) -> Self {
        Self::Leaf { name, weight }
    }

    /// Internal use: compute into TaxonOrd
    fn inner_into_taxonord<D, F: Fn(&str) -> D + Copy>(&self, next_index: &mut u128, f: F) -> TaxonOrd<D> {
        let index = *next_index;
        *next_index += 1;
        match self {
            TaxonomyBuilder::Node { name, children } => {
                TaxonOrd::Node { name: f(name), 
                    children: children.into_iter().map(|r| {
                        let index = *next_index;
                        ((0,index),Self::inner_into_taxonord(r, next_index, f))
                    } ).collect(),
                    index,
                }
            },
            TaxonomyBuilder::Leaf { name, weight } => {
                TaxonOrd::Leaf { name: f(name), weight: *weight, index }
            },
        }
    }
    
    /// Internal use: produce TaxonOrd from TaxonomyBuilder together with the next free index 
    fn into_taxonord<D, F: Fn(&str) -> D + Copy>(&self, f: F) -> (TaxonOrd<D>,u128) {
        let mut next_index = 0u128;
        (self.inner_into_taxonord(&mut next_index, f),next_index)
    }
    
    /// Internal use: compute into code-to-taxon map
    fn inner_into_taxons(&self, next_index: &mut u128slx, taxons: &mut HashMap<u128slx,Taxon>) -> u128slx {
        let index = *next_index;
        *next_index += 1u128.slx();
        match self {
            TaxonomyBuilder::Node { name, children } => {
                let children = children.iter().map(|tb| {
                    tb.inner_into_taxons(next_index,taxons)
                }).collect();
                taxons.insert(index, Taxon::Node { name: name.to_string(), children }).expect("unexpected error");
            },
            TaxonomyBuilder::Leaf { name, weight } => {
                taxons.insert(index, Taxon::Leaf { name: name.to_string(), weight: (*weight).slx() });
            },
        }
        index
    }
    
    /// Convert taxonomy builder into taxons
    /// * Output: taxons
    pub fn into_taxons(&self) -> Taxons {
        let mut next_index = 0u128.slx();
        let root = next_index;
        let mut taxons = HashMap::new();
        self.inner_into_taxons(&mut next_index, &mut taxons);
        Taxons { taxons, root, }
    }
    
    /// Build random taxonomy builder
    /// * `prefix: &str` : common prefix to the names of all taxa of the builder
    /// * `rng: &mut R` : random number generator
    /// * `rate: &Poisson<f32>` : law for sampling the umber of children
    /// * `depth: i8` : maximal depth of the taxonomy
    /// * `R: Rng` : type of random number generator
    /// * Output: random taxonomy builder
    pub fn rand_taxon_builder<R: Rng>(
        prefix: &str, rng: &mut R, rate: &Poisson<f32>, depth: i8,
    ) -> Self {
        let nb_children = if depth > 0 { rate.sample(rng) as usize } else { 0usize };
        if nb_children == 0 {
            Self::Leaf { name: prefix.to_string(), weight: Open01.sample(rng), }
        } else {
            // using set prevents name repetition
            let letters = (0..nb_children).map(|_| <()>::rand_letter(rng)).collect::<HashSet<_>>();
            let children = letters.into_iter().map(|postfix| 
                Self::rand_taxon_builder(&<()>::append(prefix,postfix),rng,rate, depth - 1)
            ).collect();
            Self::Node { name: prefix.to_string(), children, }
        }
    }        
}

impl Taxons {
    /// Internal use: get back taxonomy builder from code-to-taxon map
    fn inner_into_taxons_builder(taxons: &HashMap<u128slx,Taxon>, root: &u128slx) -> TaxonomyBuilder {
        let taxon_root = &taxons[root];
        match taxon_root {
            Taxon::Node { name, children } => {
                let children = children.iter().map(
                    |idx| Self::inner_into_taxons_builder(taxons, idx)
                ).collect();
                TaxonomyBuilder::Node { name: name.to_string(), children, }
            },
            Taxon::Leaf { name, weight } => TaxonomyBuilder::Leaf { 
                name: name.to_string(), weight: (*weight).unslx() 
            },
        }
    }
    /// Get back the taxonomy builder for these taxons
    /// * Output: taxonomy builder
    pub fn into_taxons_builder(&self) -> TaxonomyBuilder {
        let Self { taxons, root } = self;
        Self::inner_into_taxons_builder(taxons, root)
    }
}

impl Taxonomy {
    /// Internal use: taxonord is made into a binary tree by adding fake node (With None name)
    /// the process runs by priorizing subtrees with small deep (that why reordering is necessary)
    fn build_binary(rt: &mut TaxonOrd<(u8,Option<String>)>, next_index: &mut u128) -> bool {
        match rt {
            TaxonOrd::Node { children, .. } => { // compute on children
                let mut new_children = BTreeMap::new();
                std::mem::swap(children, &mut new_children);
                // children upper indexes to be recomputed:
                for((_,k), mut txn) in new_children {
                    if Self::build_binary(&mut txn, next_index) { // Thus, the subtree is computed
                        let h = match txn {
                            TaxonOrd::Node { name: (h,_), .. } | TaxonOrd::Leaf { name: (h,_), .. } => h,
                        }; 
                        children.insert((h,k),txn); // <- and reordered with upper index 
                    } else { return false; }
                }
            },
            TaxonOrd::Leaf { .. } => (), // if no children, do nothing (leaf keep index 0)
        }
        match rt {
            TaxonOrd::Node { name, children, .. } => {
                while children.len() >= 3 {
                    // take two childrens with least level
                    let mut kys = children.keys().copied();
                    let k1 = kys.next().unwrap();
                    let k2 = kys.next().unwrap();
                    let v1 = children.remove(&k1).unwrap();
                    let v2 = children.remove(&k2).unwrap();
                    // get the levels
                    let h1 = match v1 {
                        TaxonOrd::Node { name: (h,_), .. } | TaxonOrd::Leaf { name: (h,_), .. } => h,
                    };
                    let h2 = match v2 {
                        TaxonOrd::Node { name: (h,_), .. } | TaxonOrd::Leaf { name: (h,_), .. } => h,
                    };
                    let hf = if h1 > h2 { h1 + 1 } else { h2 + 1 }; // increment best level by 1
                    if hf > 121u8 { return false; }; // result level cannot exceed u128 capacity
                    // Now buil a fake uppeer node
                    let index = *next_index; *next_index += 1; 
                    let btx = TaxonOrd::Node { 
                        index, name: (hf,None), 
                        children: [(k1,v1),(k2,v2)].into_iter().collect() // key/value should be Ok
                    }; 
                    children.insert((hf,index),btx);
                }
                // update taxon name
                let new_h = children.keys().map(|(h,_)| *h).max().expect(
                    "unexpected error: children cannot be empty!"
                ) + 1;
                name.0 = new_h;
            },
            TaxonOrd::Leaf { .. } => (), // nothing changes for leaf
        }
        true
    }
    /// Internal use: build coder from binary tree
    fn build_code(
        level: u8, code: u128slx, up_full_code: u128slx, rt: TaxonOrd<(u8,Option<String>)>,
        rcoder_0: &mut HashMap::<u128slx,u128slx>, pre_taxons: &mut BTreeMap<(u8,u128slx),Taxon>,
        in_father: &mut Vec<u128slx>,
    ) {
        match rt {
            TaxonOrd::Node { name: (_, Some(name)), mut children, .. } => {
                let full_code = TaxonCoder::to_u128slx((level,code));
                // taxon is not fake:
                let up_full_code = full_code;
                // complete coder
                rcoder_0.insert(full_code,up_full_code);
                // prepare pre_taxons completion
                let pre_taxons_key = (level,full_code);
                let mut pre_taxons_children = Vec::new();
                // insert this in father's list
                in_father.push(full_code);
                // this node becomes new father
                let in_father = &mut pre_taxons_children;
                // get the children of current ord_taxon (should be binary at this time!)
                let mut keys = children.keys().cloned();
                let ok0 = keys.next(); let ok1 = keys.next();
                // a new level for the children
                let level = level+1;
                match (ok0,ok1) {
                    // two children
                    (Some(k0), Some(k1)) => {
                        let mask = (!(TaxonCoder::above_top_u128() >> level)).slx();
                        Self::build_code(
                            level,code & mask, up_full_code, children.remove(&k0).unwrap(), 
                            rcoder_0, pre_taxons, in_father
                        );
                        Self::build_code(
                            level,code, up_full_code, children.remove(&k1).unwrap(),
                            rcoder_0, pre_taxons, in_father
                        );
                    },
                    // one children
                    (Some(k0), _) => {
                        let mask = (!(TaxonCoder::above_top_u128() >> level)).slx();
                        Self::build_code(
                            level,code & mask, up_full_code, children.remove(&k0).unwrap(),
                            rcoder_0, pre_taxons, in_father
                        );
                    },
                    _ => panic!("unexpected error: node without children"),
                };
                // complete pre_taxons
                pre_taxons.insert(pre_taxons_key, Taxon::Node { name, children: pre_taxons_children });
            },
            TaxonOrd::Node { name: (_, None), mut children, .. } => {
                let full_code = TaxonCoder::to_u128slx((level,code));
                // taxon is fake; up_full_code is unchanged
                // complete coder
                rcoder_0.insert(full_code,up_full_code);
                // this is fake and is not inserted in father's list
                // get the children of current ord_taxon (should be binary at this time!)
                let mut keys = children.keys().cloned();
                let ok0 = keys.next(); let ok1 = keys.next();
                // a new level for the children
                let level = level+1;
                match (ok0,ok1) {
                    // two children
                    (Some(k0), Some(k1)) => {
                        let mask = (!(TaxonCoder::above_top_u128() >> level)).slx();
                        Self::build_code(
                            level,code & mask, up_full_code, children.remove(&k0).unwrap(), 
                            rcoder_0, pre_taxons, in_father
                        );
                        Self::build_code(
                            level,code, up_full_code, children.remove(&k1).unwrap(),
                            rcoder_0, pre_taxons, in_father
                        );
                    },
                    // one children
                    (Some(k0), _) => {
                        let mask = (!(TaxonCoder::above_top_u128() >> level)).slx();
                        Self::build_code(
                            level,code & mask, up_full_code, children.remove(&k0).unwrap(),
                            rcoder_0, pre_taxons, in_father
                        );
                    },
                    _ => panic!("unexpected error: node without children"),
                };
                // pre_taxons is unchanged
            },
            TaxonOrd::Leaf { name: (_,Some(name)), weight, .. } => {
                let full_code = TaxonCoder::to_u128slx((level,code));
                // taxon is not fake:
                let up_full_code = full_code;
                // complete coder
                rcoder_0.insert(full_code,up_full_code);
                // insert this in father's list
                in_father.push(full_code);
                // complete pre_taxons
                pre_taxons.insert((level,full_code), Taxon::Leaf { name, weight: weight.slx(), });
            },
            TaxonOrd::Leaf { name: (_,None), .. } => {
                panic!("unexpected error: leaf with fake marker")
            },
        }
    }

    /// Internal use: compute code-to-taxon map to taxonomy builder
    fn inner_into_taxon_builder(node: &u128slx, taxons: &HashMap<u128slx,Taxon>) -> TaxonomyBuilder {
        match taxons.get(node) {
            Some(Taxon::Node { name, children }) => {
                let children = children.iter().map(
                    |node| Self::inner_into_taxon_builder(node, taxons)
                ).collect();
                TaxonomyBuilder::Node { name: name.to_string(), children }
            },
            Some(Taxon::Leaf { name, weight }) => {
                TaxonomyBuilder::Leaf { name: name.to_string(), weight: (*weight).unslx() }
            },
            None => panic!("unexpected error"),
        }
    }

    /// Derive taxonomy builder from the taxonomy
    /// * Output: taxonomy builder
    pub fn into_taxon_builder(&self) -> TaxonomyBuilder {
        let Taxons { taxons, root } = &self.taxons;
        Self::inner_into_taxon_builder(root, taxons)
    }

    /// Build Taxonomy from taxonomy builder
    /// * `root: &TaxonomyBuilder` : taxonomy builder of the root taxon
    /// * Output: a taxonomy or an error
    pub fn new(root: &TaxonomyBuilder) -> Result<Taxonomy,String> {
        // construction d'une taxonomie de calcul en vue de construire la structure de donnée
        // (u8,u128) sera le code du Taxon
        // bool indiquera un taxon actif (true) ou instrumental (embranchement pour une taxonomie non binaire)
        // u128 indique le poids de la taxonomie dont le taxon est racine
        let (
            mut root_comp, mut next_index
        ) = root.into_taxonord(|t|(0,Some(t.to_string())));
        if Self::build_binary(&mut root_comp, &mut next_index) {
            let bottom = TaxonCoder::bottom_u128slx();
            let top = TaxonCoder::top_u128slx();
            let mut coder = TaxonCoder::new(HashMap::<u128slx,u128slx>::new());
            let mut pre_taxons = BTreeMap::new();
            let mut root_wrapper = vec![];
            Self::build_code( // nota: root level is 0
                              // code and up_full_code of root are top
                0u8,top,top,root_comp,
                              // root does not have father, but we fake it
                &mut coder.0, &mut pre_taxons, &mut root_wrapper,
            );
            // a small process control
            if root_wrapper.len() != 1 || top != root_wrapper[0] {
                panic!("root value mismatch: [{top}] =/= {:?}",root_wrapper);
            }
            let taxons = {
                let root = top;
                let taxons = pre_taxons.clone().into_iter()
                                                            .map(|((_,e), t,)| (e,t) ).collect();
                Taxons { taxons, root }
            };
            let (tags,untags) = pre_taxons.iter().map(|((_,e),t)| {
                match t {
                    Taxon::Node { name, .. } | Taxon::Leaf { name, .. } => {
                        ((*e,name.to_string()),(name.to_string(),*e))       
                    },
                }
            }).chain(once((
                (bottom, <() as RandomTaxon>::bot_string()),(<() as RandomTaxon>::bot_string(),bottom)
            ))).unzip();
            let (leaves,weighted_leaves) = pre_taxons.iter().filter_map(|((_,e),t)| {
                match t {
                    Taxon::Node { .. } => None, Taxon::Leaf { weight, .. } => Some((*e,(*e,*weight))),
                }
            }).unzip();
            let top_to_bottom = pre_taxons.into_iter()
                                .map(|((_,e),_)| e).chain(once(bottom)).collect();
            let lattice_hash = {
                let mut lattice_hash = Taxonomy::TYPE_HASH_NATIVE;
                lattice_hash = add_hash_fnv1a(&coder.0.len().to_le_bytes(), lattice_hash);
                let coder = coder.0.iter().collect::<BTreeMap<_,_>>();
                for (u,v) in coder {
                    lattice_hash = add_hash_fnv1a(&(*u).unslx().to_le_bytes(), lattice_hash);
                    lattice_hash = add_hash_fnv1a(&(*v).unslx().to_le_bytes(), lattice_hash);
                }
                let taxons = taxons.taxons.iter().collect::<BTreeMap<_,_>>();
                for (u, taxon) in taxons {
                    lattice_hash = add_hash_fnv1a(&(*u).unslx().to_le_bytes(), lattice_hash);
                    match taxon {
                        Taxon::Node { name, children } => {
                            lattice_hash = add_hash_fnv1a(b"Node", lattice_hash);
                            lattice_hash = add_hash_fnv1a(name.as_bytes(), lattice_hash);
                            lattice_hash = add_hash_fnv1a(&children.len().to_le_bytes(), lattice_hash);
                            let children = children.iter().collect::<BTreeSet<_>>();
                            for child in children {
                                lattice_hash = add_hash_fnv1a(&child.unslx().to_le_bytes(), lattice_hash);
                            }
                        },
                        Taxon::Leaf { name, weight } => {
                            lattice_hash = add_hash_fnv1a(b"Leaf", lattice_hash);
                            lattice_hash = add_hash_fnv1a(name.as_bytes(), lattice_hash);
                            lattice_hash = add_hash_fnv1a(&(*weight).unslx().to_le_bytes(), lattice_hash);
                            
                        },
                    }
                }
                lattice_hash.slx()
            };
            let bottom = SafeElement { code: bottom, lattice_hash };
            let top = SafeElement { code: top, lattice_hash};
            Ok(Taxonomy { taxons, top, bottom, coder, tags, untags, leaves, weighted_leaves, top_to_bottom, })    
        } else { Err("Failed to build taxonomy".to_string()) }
    }

    /// for internal use: an empty taxonomy -> produced in case of deserialization error
    fn empty() -> Self {
        let zero = 0u128.slx();
        let taxons = Taxons { taxons: HashMap::new(), root: zero };
        let top = SafeElement { code: zero, lattice_hash: zero };
        let bottom = top;
        let coder = TaxonCoder(HashMap::new());
        let tags = HashMap::new();
        let untags = HashMap::new();
        let weighted_leaves = HashMap::new();
        let leaves = Vec::new();
        let top_to_bottom = Vec::new();
        Taxonomy { 
            taxons, top, bottom, coder, tags, untags, leaves, weighted_leaves, top_to_bottom, 
        }
    }

    /// Generate random taxonomy
    /// * `rng: &mut R` : random nuber generator
    /// * `R: Rng` : type of random number generator
    /// * Output: a random taxonomy
    pub fn rand_taxonomy<R: Rng>(rng: &mut R) -> Taxonomy {
        let rate = Poisson::new(3f32).unwrap();
        let root = TaxonomyBuilder::rand_taxon_builder(
            <()>::rand_letter(rng),rng,&rate,5
        );
        Self::new(&root).expect("unexpected taxonomy build failure")
    }
}

impl Lattice for Taxonomy {
    type Item = u128slx;

    fn rand_lattice<R: Rng>(rng: &mut R) -> Self { Self::rand_taxonomy(rng) }

    fn rand_element<R: Rng>(&self, rng: &mut R) -> SafeElement<Self::Item> {
        let Self { bottom, top_to_bottom, .. } = self;
        let element = top_to_bottom[rng.gen_range(0..top_to_bottom.len())];
        let lattice_hash = bottom.lattice_hash;
        SafeElement { code: element, lattice_hash }
    }

    fn ref_lattice_hash(&self) -> &u128slx { &self.bottom.lattice_hash }

    fn contains(&self, element: &Self::Item) -> bool { self.taxons.taxons.contains_key(element) }

    fn ref_bottom(&self) -> &crate::structs::SafeElement<Self::Item> { &self.bottom }

    fn ref_top(&self) -> &crate::structs::SafeElement<Self::Item> { &self.top }

    unsafe fn unsafe_meet(&self, element_left: &Self::Item, element_right: &Self::Item) -> Self::Item {
        self.coder.meet_slx(*element_left, *element_right)
    }

    unsafe fn unsafe_join(&self, element_left: &Self::Item, element_right: &Self::Item) -> Self::Item {
        self.coder.join_slx(*element_left, *element_right)
    }

    fn from_str(&self, s: &str) -> Result<crate::structs::SafeElement<Self::Item>,String> {
        let element = match self.untags.get(s) {
            Some(e) => *e,
            None => return Err(format!("Taxon {s} is unknown")),
        };
        let lattice_hash = self.bottom.lattice_hash;
        Ok(SafeElement { code: element, lattice_hash })
    }

    fn to_string(&self, element: &crate::structs::SafeElement<Self::Item>) -> Result<String,String> {
        let SafeElement { code: element, lattice_hash } = element;
        if lattice_hash == &self.bottom.lattice_hash {
            match self.tags.get(element) {
                Some(s) => Ok(s.to_string()),
                None => Err("Unexpected: element is not within lattice, although lattice hashes are same".to_string()),
            }
        } else {
            Err("Lattice hash mismatch: element is not within lattice".to_string())
        }
    }
}

impl IterableLattice for Taxonomy {
    type IntoIterUp = vec::IntoIter<u128slx>;

    type IntoIterDown = vec::IntoIter<u128slx>;

    unsafe fn unsafe_bottom_to_top(&self) -> Result<Self::IntoIterUp,String> {
        Ok(self.top_to_bottom.iter().rev().copied().collect::<Vec<_>>().into_iter())
    }

    unsafe fn unsafe_top_to_bottom(&self) -> Result<Self::IntoIterDown,String> {
        Ok(self.top_to_bottom.clone().into_iter())
    }
}

impl LatticeWithLeaves for Taxonomy {
    type IntoIterLeaves = hash_map::IntoIter<Self::Item, f64slx>;

    unsafe fn unsafe_leaves(&self) -> Result<Self::IntoIterLeaves,String> {
        Ok(self.weighted_leaves.clone().into_iter())
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

pub mod experiment {
    use rand::Rng;
    use rand_distr::Poisson;

    use crate::{structs::{ TaxonomyBuilder, Taxonomy }, traits::Lattice};
    use super::RandomTaxon;

    fn is_node(tb: &TaxonomyBuilder, prop: &str,) -> bool {
        match tb {
            TaxonomyBuilder::Node { name, .. } | TaxonomyBuilder::Leaf { name, .. } => name == prop,
        }
    }

    fn contain(tb: &TaxonomyBuilder, prop: &str,) -> bool {
        match tb {
            TaxonomyBuilder::Node { name, children } => {
                if name == prop { true } else {
                    for tb in children {
                        if contain(tb, prop) { return true; }
                    }
                    false
                }
            },
            TaxonomyBuilder::Leaf { name, .. } => name == prop,
        }
    }

    fn find_meet(tb: &TaxonomyBuilder, left: &str, right: &str) -> Option<String> {
        match (left == <()>::bot_str(), right == <()>::bot_str()) {
            (false, false) => {
                match (is_node(tb, left), is_node(tb, right), contain(tb, left), contain(tb, right)) {
                    (_, _, true, false) | (_, _, false, true) | (_, _, false, false)  => None,
                    (_, true, _, _) => Some(left.to_string()),
                    (true, false, _, _) => Some(right.to_string()),
                    (false, false, _, _) => {
                        match tb {
                            TaxonomyBuilder::Node { children, .. } => {
                                for child in children  {
                                    match (contain(child, left),contain(child, right)) {
                                        (true, true) => return find_meet(child,left,right),
                                        (false, false) => (),
                                        _ => return Some(<()>::bot_str().to_string()),                                    }
                                }
                                panic!("unexpected error")
                            },
                            TaxonomyBuilder::Leaf { .. } => panic!("unexpected error"),
                        }
                    },
                }
            },
            (true,true) => Some(<()>::bot_str().to_string()),
            (false, true) => if contain(tb, left) { Some(<()>::bot_str().to_string()) } else { None },
            (true, false) => if contain(tb, right) { Some(<()>::bot_str().to_string()) } else { None },
        }
    }

    fn find_join(tb: &TaxonomyBuilder, left: &str, right: &str) -> Option<String> {
        match (left == <()>::bot_str(), right == <()>::bot_str()) {
            (false, false) => {
                match (is_node(tb, left), is_node(tb, right), contain(tb, left), contain(tb, right)) {
                    (_, _, true, false) | (_, _, false, true) | (_, _, false, false)  => None,
                    (true, _, _, _) => Some(left.to_string()),
                    (false, true, _, _) => Some(right.to_string()),
                    (false, false, _, _) => {
                        match tb {
                            TaxonomyBuilder::Node { name, children } => {
                                for child in children  {
                                    match (contain(child, left),contain(child, right)) {
                                        (true, true) => return find_join(child,left,right),
                                        (false, false) => (),
                                        _ => return Some(name.to_string()),                                    }
                                }
                                panic!("unexpected error")
                            },
                            TaxonomyBuilder::Leaf { .. } => panic!("unexpected error"),
                        }
                    },
                }
            },
            (false, true) => if contain(tb, left) { Some(left.to_string()) } else { None },
            (true, false) => if contain(tb, right) { Some(right.to_string()) } else { None },
            (true, true) => Some(<()>::bot_str().to_string()),
        }
    }

    /// Experimentation 1 with taxonomy
    pub fn exp_taxonomy_1() -> Result<(),String> { // OK
        let taxon_e = TaxonomyBuilder::new_leaf("E".to_string(), 0.1);
        let taxon_f = TaxonomyBuilder::new_leaf("F".to_string(), 0.05);
        let taxon_g = TaxonomyBuilder::new_leaf("G".to_string(), 0.1);
        let taxon_h = TaxonomyBuilder::new_leaf("H".to_string(), 0.15);
        let taxon_i = TaxonomyBuilder::new_leaf("I".to_string(), 0.2);
        let taxon_k = TaxonomyBuilder::new_leaf("K".to_string(), 0.05);
        let taxon_l = TaxonomyBuilder::new_leaf("L".to_string(), 0.1);
        let taxon_m = TaxonomyBuilder::new_leaf("M".to_string(), 0.05);
        let taxon_n = TaxonomyBuilder::new_leaf("N".to_string(), 0.05);
        let taxon_o = TaxonomyBuilder::new_leaf("O".to_string(), 0.15);
        let taxon_b = TaxonomyBuilder::new_node(
            "B".to_string(), vec![taxon_e,taxon_f,taxon_g],
        )?;
        let taxon_c = TaxonomyBuilder::new_node(
            "C".to_string(), vec![taxon_h],
        )?;
        let taxon_j = TaxonomyBuilder::new_node(
            "J".to_string(), vec![taxon_m,taxon_n,taxon_o],
        )?;
        let taxon_d = TaxonomyBuilder::new_node(
            "D".to_string(), vec![taxon_i,taxon_j,taxon_k,taxon_l],
        )?;
        let taxon_a = TaxonomyBuilder::new_node(
            "A".to_string(), vec![taxon_b,taxon_c,taxon_d],
        )?;
        let taxonomy = Taxonomy::new(&taxon_a)?;
        println!("taxon_a: {:#?}", taxon_a);
        println!("taxonomy.into_taaxon_builder(): {:#?}", taxonomy.into_taxon_builder());
        println!("-------- meet ---");
        let tx_names = taxonomy.tags
            .iter().map(|(_,n)|n.to_string()).collect::<Vec<_>>();
        let mut fail_meet = 0;
        let mut success_meet = 0;
        for n in &tx_names {
            let c_n = &taxonomy.untags[n]; 
            for m in &tx_names {
                let c_m = &taxonomy.untags[m]; 
                let meet = &taxonomy.tags[&unsafe { taxonomy.unsafe_meet(c_n, c_m) }];
                let theoric_meet = find_meet(&taxon_a, &n, &m).unwrap();
                if meet == &theoric_meet { success_meet += 1; } else { fail_meet +=1; }
                println!("{n} & {m} -> {meet} ; theoric: {theoric_meet}");
            }
        }
        println!("-------- join ---");
        let mut fail_join = 0;
        let mut success_join = 0;
        for n in &tx_names {
            let c_n = &taxonomy.untags[n]; 
            for m in &tx_names {
                let c_m = &taxonomy.untags[m]; 
                let join = &taxonomy.tags[&unsafe { taxonomy.unsafe_join(c_n, c_m) }];
                let theoric_join = find_join(&taxon_a, &n, &m).unwrap();
                if join == &theoric_join { success_join += 1; } else { fail_join +=1; }
                println!("{n} | {m} -> {join} ; theoric: {theoric_join}");
            }
        }
        println!("------- stats ---");
        println!("meet: fail / success = {fail_meet} / {success_meet}");
        println!("join: fail / success = {fail_join} / {success_join}");
        Ok(())
    }

    /// Experimentation 2 with taxonomy
    pub fn exp_taxonomy_2() -> Result<(),String> { // OK
        let mut rng = rand::thread_rng();
        let mut fail_meet = 0;
        let mut success_meet = 0;
        let mut fail_join = 0;
        let mut success_join = 0;
        for u in 0..100 {
            let rate = Poisson::new(3f32).unwrap();
            let depth = rng.gen_range(2..7);
            let root = TaxonomyBuilder::rand_taxon_builder(
                <()>::rand_letter(&mut rng),&mut rng,&rate,depth
            );
            let taxonomy = Taxonomy::new(&root)?;
            println!("taxonomy {u} ; size: {}", taxonomy.top_to_bottom.len());
            let tx_names = taxonomy.tags
                .iter().map(|(_,n)|n.to_string()).collect::<Vec<_>>();
            for n in &tx_names {
                let c_n = &taxonomy.untags[n]; 
                for m in &tx_names {
                    let c_m = &taxonomy.untags[m]; 
                    let meet = &taxonomy.tags[&unsafe { taxonomy.unsafe_meet(c_n, c_m) }];
                    let theoric_meet = find_meet(&root, &n, &m).unwrap();
                    if meet == &theoric_meet { success_meet += 1; } else { fail_meet +=1; }
                }
            }
            for n in &tx_names {
                let c_n = &taxonomy.untags[n]; 
                for m in &tx_names {
                    let c_m = &taxonomy.untags[m]; 
                    let join = &taxonomy.tags[&unsafe { taxonomy.unsafe_join(c_n, c_m) }];
                    let theoric_join = find_join(&root, &n, &m).unwrap();
                    if join == &theoric_join { success_join += 1; } else { fail_join +=1; }
                }
            }
        }
        println!("------- stats ---");
        println!("meet: fail / success = {fail_meet} / {success_meet}");
        println!("join: fail / success = {fail_join} / {success_join}");
        Ok(())
    }
}
