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


use std::{ collections::{BTreeSet, HashMap}, fmt::{Debug, Display}, hash::Hash, ops::{ Add, Index }, sync::OnceLock };

use hashed_type_def::HashedTypeDef;
// #[cfg(not(feature = "silx-types"))] use crate::fake_slx::{ u128slx, u32slx, f64slx, FakeSlx, };
// #[cfg(feature = "silx-types")] use silx_types::{ u128slx, u32slx, f64slx, SlxInto, IntoSlx, Float, };
use crate::types::{ u128slx, u32slx, f64slx, SlxInto, IntoSlx, };
#[cfg(feature = "silx-types")] use silx_types::Float;

#[cfg(feature = "serde")] use serde::{ Serialize as SerdeSerialize, Deserialize as SerdeDeserialize, };
#[cfg(feature = "rkyv")] use rkyv::{ Archive, Serialize as RkyvSerialize, Deserialize as RkyvDeserialize, };
use self::hidden::OrdMap;

use super::SafeElement;

pub const  ASSIGNMENT_EPSILON: f64 = 1e-8;
static ZERO_F64_SLX: OnceLock<f64slx> = OnceLock::new();
static ONE_F64_SLX: OnceLock<f64slx> = OnceLock::new();

pub fn zero_f64slx() -> &'static f64slx { ZERO_F64_SLX.get_or_init(|| 0.0.slx()) }
pub fn one_f64slx() -> &'static f64slx { ONE_F64_SLX.get_or_init(|| 1.0.slx()) }

#[derive(Clone,HashedTypeDef)]
/// Intermediate structure for building assignments
/// * In particular, this structrure will handle mass discounting
/// * There is no constructor for `AssignmentBuilder`: methods `init_assignment` or `init_assignment_with_capacity` of trait `Lattice` are used
/// * `X` : type of lattice element encoding
pub struct AssignmentBuilder<X> where X: Eq + Ord + Hash {
    pub (crate) elements: hidden::OrdMap<X>,
    pub (crate) lattice_hash: u128slx,
    pub (crate) length_mid: u32slx,
    pub (crate) length_max: u32slx,
}

#[derive(Clone,HashedTypeDef,)]
#[cfg_attr(feature = "rkyv", derive(Archive,RkyvSerialize,RkyvDeserialize))]
/// Mass assignment: should be build from AssignmentBuilder
/// * Assignment contains a lattice hash and a sequence of weighted encoded element from this lattice
/// * Assignments are used in order to encode basic belief function and other belief functions 
/// * There is no constructor for `Assignment`: use method `From::from(...)` to convert from a builder `AssignmentBuilder`
/// * `X` : type of lattice element encoding
pub struct Assignment<X> where X: Eq + Hash {
    pub elements: HashMap<X,f64slx>,
    pub lattice_hash: u128slx,
}

impl <X> From<AssignmentBuilder<X>> for Assignment<X> where X: Eq + Ord + Hash {
    fn from(value: AssignmentBuilder<X>) -> Self {
        let AssignmentBuilder { 
            elements: OrdMap { elements, .. }, lattice_hash, .. 
        } = value;
        Self { elements, lattice_hash, }
    }
}

// implementation of Serde serialization
#[cfg(feature = "serde")] 
mod serding {
    use std::collections::BTreeMap;
    use super::{ Assignment as SerdingAssignment, SerdeSerialize, SerdeDeserialize, Hash, };
    // #[cfg(feature = "silx-types")] use super::{ IntoSlx, SlxInto, };
    // #[cfg(not(feature = "silx-types"))] use crate::fake_slx::FakeSlx;
    use crate::types::{ SlxInto, IntoSlx, };
    #[derive(SerdeSerialize,SerdeDeserialize)]
    pub struct Assignment<X> where X: Eq + Ord, {
        elements: BTreeMap<X,f64>,
        lattice_hash: u128,
    }

    impl<'de, X> SerdeDeserialize<'de> for SerdingAssignment<X>  
                                        where X: Clone + Eq + Ord + Hash + SerdeDeserialize<'de>, {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de> {
            let Assignment { 
                elements, lattice_hash,
            } = Assignment::<X>::deserialize(deserializer)?;
            let lattice_hash = lattice_hash.slx();
            let elements = elements.into_iter().map(|(x,w)| (x,w.slx())).collect(); 
            Ok(Self { elements, lattice_hash, })
        }
    }
    
    impl<X> SerdeSerialize for SerdingAssignment<X>  where X: Clone + Eq + Ord + Hash + SerdeSerialize, {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer {
            let SerdingAssignment { 
                elements, lattice_hash,
            } = self;
            let elements = elements.iter().map(|(x,w)| {
                (x.clone(), (*w).unslx())  
            }).collect();
            let lattice_hash = (*lattice_hash).unslx();
            let assignment = Assignment { elements, lattice_hash, };
            assignment.serialize(serializer)
        }
    }
}

impl<X,T> Add<(SafeElement<X>,T)> for AssignmentBuilder<X> where X: Eq + Ord + Hash + Clone, T: Into<f64slx> {
    type Output = Self;

    fn add(mut self, (x,w): (SafeElement<X>,T)) -> Self::Output {
        self.push(x,w.into()).unwrap(); self
    }
}

impl<X> Add<()> for AssignmentBuilder<X> where X: Eq + Ord + Hash + Clone {
    type Output = Assignment<X>;

    fn add(mut self, _: ()) -> Self::Output {
        self.normalize().unwrap(); self.into()
    }
}

impl<X> Debug for Assignment<X> where X: Eq + Hash + Debug, {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.elements.iter()
            .fold(" ".to_string(),|acc,(u,w)| format!("{acc}{u:?} -> {w}, "));
        f.debug_struct("Assignment").field("elements", &value).field("lattice_hash", &self.lattice_hash).finish()
    }
}

impl<X> Display for Assignment<X> where X: Eq + Hash + Display, {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.elements.iter()
            .fold("[ ".to_string(),|acc,(u,w)| format!("{acc}{u} -> {w:.4}, "));
        f.write_str(&value)?;
        f.write_str("]")
    }
}

impl<X> AssignmentBuilder<X> where X: Clone + Eq + Ord + Hash, {
    /// Unsafe push weighted element within assignment; weights smaller than `ASSIGNMENT_EPSILON` are discarded (result in `Ok(false)`)
    /// * method is unsafe since there is no consistency check that the encoded element comes from the lattice of the builder
    /// * `element: X` : encoded element
    /// * `weight: f64slx` : weight of the element
    /// * Output: 
    ///   * `true` if weighted element is inserted
    ///   * `false` if the weighted element is discarded
    ///   * error if the weight is non finite or negative
    pub unsafe fn unsafe_push(&mut self, element: X, weight:f64slx) -> Result<bool, String> {
        let Self { elements, .. } = self;
        let w_native = weight.unslx();
        if w_native.is_finite() && !w_native.is_sign_negative() {
            if w_native > ASSIGNMENT_EPSILON { Ok({ elements.push(element,weight); true }) } else {Ok(false)}
        } else { Err("non finite or negative weights are forbidden".to_string()) }
    }

    /// Push weighted element within assignment; weights smaller than `ASSIGNMENT_EPSILON` are discarded (result in `Ok(false)`)
    /// * `safe_element: SafeElement<X>` : element with safe encoding
    /// * `weight: f64slx` : weight of the element
    /// * Output: 
    ///   * `true` if weighted element is inserted
    ///   * `false` if the weighted element is discarded
    ///   * error if the weight is non finite or negative
    pub fn push(&mut self, safe_element: SafeElement<X>, weight: f64slx) -> Result<bool, String> {
        let Self { elements, lattice_hash, .. } = self;
        let w_native = weight.unslx();
        let assign_lattice_hash = *lattice_hash;
        let SafeElement { code: element, lattice_hash } = safe_element;
        if lattice_hash == assign_lattice_hash {
            if w_native.is_finite() && !w_native.is_sign_negative() {
                if w_native > ASSIGNMENT_EPSILON { Ok({ elements.push(element,weight); true }) } 
                else {Ok(false)}
            } else { Err("non finite or negative weights are forbidden".to_string()) }
        } else { Err("lattice hash mismatch".to_string()) } 
    }

    /// Remove element from assignment
    /// * `safe_element: &SafeElement<X>` : reference to lement to be removed
    /// * Output: 
    ///   * Some safe element `Some(se)` if element was found
    ///   * `None` if element was not found
    ///   * error in case of lattice hash mismatch
    pub fn remove(&mut self, safe_element: &SafeElement<X>) -> Result<Option<(SafeElement<X>,f64slx)>, String> {
        let Self { elements, lattice_hash: assign_lattice_hash, .. } = self;
        let SafeElement { code: element, lattice_hash } = safe_element;
        if lattice_hash == assign_lattice_hash { match elements.remove(element) {
            Some((element,w)) => Ok(Some((SafeElement { code: element, lattice_hash: *lattice_hash },w))),
            None => Ok(None),
        } } else { Err("lattice hash mismatch".to_string()) } 
    }

    /// Prune the assignment in order to reduce its size within acceptable range
    /// * `pruner: F` : prunning function
    ///   * two weighted encoded elements `(x,wx)` and `(y,wy)` to be prunned will be replaced by (pruner(x,y),wx + wy)
    /// * `F` : type of pruner 
    /// * Output: nothing
    pub fn prune<F>(&mut self, pruner: F) where F: Fn(X,X) -> X {
        let Self { elements, length_mid, length_max, .. } = self;
        let length_mid = (*length_mid).unslx() as usize;
        let length_max = (*length_max).unslx() as usize;
        if elements.len() > length_max {
            let length_mid = length_mid.max(1); // impossible to prune over 0 or 1 element
            while elements.len() > length_mid {
                let (x,v) = elements.pop_first().unwrap();
                let (y,w) = elements.pop_first().unwrap();
                elements.push(pruner(x,y), v + w);
            }    
        }
    }

    /// Scale the assignment weights
    /// * `scaler: f64slx` : scale multiplier
    /// * Output: nothing or error
    pub fn scale(&mut self, scaler: f64slx) -> Result<(), String> {
        self.self_map(|w| w * scaler)
    }

    /// Shift the assignment negatively
    /// * Weighted element `(x,w)` is replaced by `(x,w - neg_shift)`
    /// * `neg_shift: f64slx` : negative shift
    /// * Output: nothing or error
    pub fn neg_shift(&mut self, neg_shift: f64slx) -> Result<(), String> {
        self.self_map(|w| w - neg_shift)
    }

    /// Compute the cumulative weight of the assignment
    /// * Output: the cumulative weight or an error if some weights are non finite or negative
    pub fn cumul_weight(&self) -> Result<f64slx, String> {
        let mut cumul = 0.0;
        let elements = &self.elements.elements;
        for (_,rw) in elements.iter() {
            let w = (*rw).unslx();
            if w.is_finite() && w >= 0.0 {
                cumul += w;
            } else { return Err("weight is not finite or not positive".to_string()); }
        }
        Ok(cumul.slx())
    }

    /// Normalize the assignment
    /// * Output: nothing or error
    pub fn normalize(&mut self) -> Result<(), String> {
        let norm = self.cumul_weight()?;
        if &norm == zero_f64slx() {
            Err("Cumulative weight is zero, cannot be normalized".to_string())
        } else { self.scale(norm.recip()) }
    }

    /// Map the assignment with a closure
    /// * `f: F` : a closure
    /// * `F` : type of the closure
    /// * Output: mapped assignment or an error if some mapped weights are non finite or negative
    pub fn map<F>(self, mut f: F) -> Result<Self,String> where F: FnMut(f64slx) -> f64slx {
        let Self { lattice_hash, length_mid, length_max, elements, } = self;
        let mapped = elements.ord_elements.into_iter()
            .map(|hidden::OrdData((x,w))| (x,f(w))).collect::<Vec<_>>();
        for (_,w) in &mapped { 
            let w = (*w).unslx();
            if !w.is_finite() || w.is_sign_negative() {
                return Err(format!("mapped weight {w} is not finite or is sign negative"));
            } 
        }
        let mapped_filtered = mapped.into_iter().filter(|(_,w)| (*w).unslx() > ASSIGNMENT_EPSILON).collect::<Vec<_>>();
        let elements = mapped_filtered.iter().cloned().collect::<HashMap<_,_>>();
        let ord_elements = mapped_filtered.into_iter()
                                    .map(|xw| hidden::OrdData(xw)).collect::<BTreeSet<_>>();
        let elements = OrdMap { elements, ord_elements, };
        Ok(Self { lattice_hash, length_mid, length_max, elements })
    }

    /// Self-map the assignment with a closure
    /// * `f: F` : a closure
    /// * `F` : type of the closure
    /// * Output: nothing or an error if some mapped weights are non finite or negative
    pub fn self_map<F>(&mut self, f: F) -> Result<(),String> where F: FnMut(f64slx) -> f64slx {
        let mut assign_tmp = AssignmentBuilder { 
            elements: OrdMap::new(), lattice_hash: 0u128.slx(), length_mid: 0u32.slx(), length_max: 0u32.slx() 
        };
        std::mem::swap(self, &mut assign_tmp);
        *self = assign_tmp.map(f)?;
        Ok(())
    }
}


impl<X> Index<&SafeElement<X>> for Assignment<X> where X: Eq + Ord + Hash, {
    type Output = f64slx;

    fn index(&self, SafeElement { code: element, lattice_hash }: &SafeElement<X>) -> &Self::Output {
        if lattice_hash != &self.lattice_hash { panic!("mismatching lattice hash"); }
        self.elements.index(element)
    }
}

impl<X> IntoIterator for Assignment<X> where X: Eq + Ord + Hash, {
    type Item = (SafeElement<X>,f64slx);

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let Self { elements, lattice_hash, .. } = self;
        elements.into_iter().map(move |(rce,w)| (SafeElement { 
            code: rce, lattice_hash 
        },w)).collect::<Vec<_>>().into_iter()   
    }
}

impl<'a, X> IntoIterator for &'a Assignment<X> where X: Eq + Ord + Hash, {
    type Item = (SafeElement<&'a X>,f64slx);

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let Assignment { elements,  lattice_hash, .. } = self;
        elements.iter().map(move |(rce,w)| (SafeElement { 
            code: rce, lattice_hash: *lattice_hash
        },*w)).collect::<Vec<_>>().into_iter()   
    }
}

pub (crate) mod hidden{
    use std::ops::Index;

    use super::*;

    #[derive(PartialEq,HashedTypeDef,)]
    #[cfg_attr(feature = "rkyv", derive(Archive,RkyvSerialize,RkyvDeserialize))]
    #[repr(transparent)]
    /// a structure for internal use
    pub struct OrdData<X>(pub (X, f64slx,));

    impl<X> Clone for OrdData<X> where X: Clone {
        fn clone(&self) -> Self {
            let Self((x,w)) = self;
            Self((x.clone(),*w))
        }
    }

    impl<X> PartialOrd for OrdData<X> where X: PartialOrd {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            match self.0.1.partial_cmp(&other.0.1) {
                Some(core::cmp::Ordering::Equal) => {}, ord => return ord,
            } self.0.0.partial_cmp(&other.0.0)
        }
    }

    impl<X> Eq for OrdData<X> where X: PartialEq { }

    impl<X> Ord for OrdData<X> where X: Ord + Eq {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering { 
            match self.0.1.partial_cmp(&other.0.1) {
                Some(core::cmp::Ordering::Equal) => {}, Some(ord) => return ord,
                None => panic!("Comparison of {} and {} failed", self.0.1, other.0.1),
            } self.0.0.cmp(&other.0.0)
        }
    }

    #[derive(HashedTypeDef,)]
    #[cfg_attr(feature = "rkyv", derive(Archive,RkyvSerialize,RkyvDeserialize))]
    /// a map structure for internal use; element are sorted with weight
    pub struct OrdMap<X> where X: Eq + Ord + Hash, {
        pub (crate) elements: HashMap<X,f64slx>,
        pub (crate) ord_elements: BTreeSet<OrdData<X>>,    
    }

    impl<X> Clone for OrdMap<X> where X: Eq + Ord + Hash + Clone, {
        fn clone(&self) -> Self {
            let Self { ord_elements, .. } = self;
            // build a deep clone of the data (cloning OrdData is deep)
            let weighted = ord_elements.iter().cloned().collect::<Vec<_>>();
            // collect on weak clone (OrdData is removed first) 
            let elements = weighted.iter().map(|OrdData((x,w))| (x.clone(),*w)).collect();
            let ord_elements = weighted.into_iter().collect();
            Self { elements, ord_elements, }
        }
    }

    impl<X> OrdMap<X> where X: Clone + Eq + Ord + Hash, {
        /// Constructor
        pub fn new() -> Self {
            Self { elements: HashMap::new(), ord_elements: BTreeSet::new(), }
        }
        /// Constructor with given capacity
        pub fn with_capacity(capacity: usize) -> Self {
            Self { elements: HashMap::with_capacity(capacity), ord_elements: BTreeSet::new(), }
        }
        /// Take first element (i.e. with the smallest weight)
        pub fn pop_first(&mut self) -> Option<(X,f64slx)> {
            match self.ord_elements.pop_first() {
                Some(OrdData((xx,_))) => match self.elements.remove_entry(&xx) {
                    None => panic!("unexpected missing entry"), 
                    some => some.map(|(x,w)| {
                        drop(xx); (x,w)
                    }),
                }, None => None,
            }
        }
        /// remove element and get its weight
        pub fn remove(&mut self, x: &X) -> Option<(X,f64slx)> {
            match self.elements.remove_entry(x) {
                Some(xw) => {
                    match self.ord_elements.take(unsafe{ std::mem::transmute(&xw)}) {
                        None => panic!("unexpected missing element"), 
                        Some(OrdData((xx,w))) => {
                            drop(xw);
                            Some((xx, w))
                        },
                    }
                },
                None => None,
            }
        }
        /// push new weighted element
        pub fn push(&mut self, x: X, w: f64slx) {
            let (x,w) = match self.remove(&x) { Some((_,v)) => (x,v+w), None => (x,w), };
            self.ord_elements.insert(OrdData((x.clone(),w))); // Nota: weak cloning here
            self.elements.insert(x, w);
        }
        /// Collection length
        pub fn len(&self) -> usize {
            let len1 = self.elements.len();
            let len2 = self.ord_elements.len();
            if len1 != len2 { panic!("unexpected error: mismatching lens") }
            len1
        }
    }

    impl<X> Index<&X> for OrdMap<X> where X: Eq + Ord + Hash, {
        type Output = f64slx;

        fn index(&self, index: &X) -> &Self::Output {
            match self.elements.get(index) { Some(x) => x, None => zero_f64slx(), }
        }
    }
}

/// Experimentation for internal test
pub fn exp_hidden() {
    let mut om = hidden::OrdMap::<&'static str>::new();
    om.push("A",0.125.slx());
    om.push("B",0.25.slx());
    om.push("C",0.25.slx());
    om.push("B",0.25.slx());
    om.push("C",0.125.slx());
    println!("om.len() -> {}", om.len());
    println!("om.remove(&\"B\") -> {:?}", om.remove(&"B"));
    println!("om.len() -> {}", om.len());
    println!("om.pop_first() -> {:?}", om.pop_first());
    println!("om.len() -> {}", om.len());
    println!("om.remove(&\"B\") -> {:?}", om.remove(&"B"));
    println!("om.len() -> {}", om.len());
    println!("om.pop_first() -> {:?}", om.pop_first());
    println!("om.len() -> {}", om.len());
    println!("om.pop_first() -> {:?}", om.pop_first());
    println!("om.len() -> {}", om.len());
}