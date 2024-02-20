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


use std::{ rc::Rc, sync::Arc, hash::Hash, collections::HashMap, };
use hashed_type_def::HashedTypeDef;
use rand::prelude::*;
use silx_types::{ u128slx, IntoSlx, Float, };

use crate::{ 
    structs::{
        SafeElement, AssignmentBuilder, Assignment, hidden, 
        ASSIGNMENT_EPSILON, one_f64slx, zero_f64slx, 
    }, 
    traits::CollectionFamily1, 
};

/// General trait for lattices
pub trait Lattice: Sized {
    /// Type for encoding lattice elements: 
    ///   * type `Self::Item` may contain element which are not in the lattice
    ///   * type `SafeElement<Self::Item>` contains both the element and the hash of lattice in order to assert its origin:
    ///     * `SafeElement<Self::Item>` is only produced by method `check_safe` which control if raw element is within the lattice
    ///     * `SafeElement<Self::Item>`  or `SafeCollection<Self::Item,I>` contains checked elements; 
    ///     * Safe elements are helper, but in last ressort, the user is responsible to ensure the data validity
    type Item: Clone + Eq + HashedTypeDef; 

    /// Random lattice generator (for tests)
    /// * `rng: &mut R` : random number generator
    /// * `R: Rng` : type of random number generator
    /// * Output: random lattice
    fn rand_lattice<R: Rng>(rng: &mut R) -> Self;

    /// Random element generator (for tests); elements are built safe
    /// * `rng: &mut R` : random number generator
    /// * `R: Rng` : type of random number generator
    /// * Output: random element
    fn rand_element<R: Rng>(&self, rng: &mut R) -> SafeElement<Self::Item>;

    /// Reference to lattice hash
    fn ref_lattice_hash(&self) -> &u128slx;

    /// Test if element is from lattice
    /// * this is necessary since type Self::Item may contain more potential elements than the lattice 
    /// * `element: &Self::Item` : element to be tested
    /// * Output: boolean
    fn contains(&self, element: &Self::Item) -> bool;

    /// Reference to least element of the lattice
    /// * Output: reference to least element
    fn ref_bottom(&self) -> &SafeElement<Self::Item>;

    /// Reference to greatest element of the lattice
    /// * Output: reference to greatest element
    fn ref_top(&self) -> &SafeElement<Self::Item>;
        
    /// Unsafe greatest lower bound
    /// * values are not tested to be within lattice.
    /// * contract: operator should be associative
    /// * `element_left: &Self::Item` : left operand
    /// * `element_right: &Self::Item` : right operand
    /// * Output: unsafe greatest lower bound
    unsafe fn unsafe_meet(&self, element_left: &Self::Item, element_right: &Self::Item) -> Self::Item;

    /// Unsafe least upper bound
    /// * values are not tested to be within lattice.
    /// * contract: operator should be associative!
    /// * `element_left: &Self::Item` : left operand
    /// * `element_right: &Self::Item` : right operand
    /// * Output: unsafe least upper bound
    unsafe fn unsafe_join(&self, element_left: &Self::Item, element_right: &Self::Item) -> Self::Item;

    /// Parse safe element from str
    /// * `s: &str` : string to be parsed
    /// * Output: parsed safe element or error
    fn from_str(&self, s: &str) -> Result<SafeElement<Self::Item>,String>;

    /// Format safe element into String
    /// * `element: &SafeElement<Self::Item>` : safe element to be formated into string
    /// * Output: formated string or error
    fn to_string(&self, element: &SafeElement<Self::Item>) -> Result<String,String>; 
    
    /// Lattice hash
    /// * Output: lattice hash
    fn lattice_hash(&self) -> u128slx { *self.ref_lattice_hash() }

    /// Least element of the lattice
    /// * Output: least element of the lattice
    fn bottom(&self) -> SafeElement<Self::Item> { self.ref_bottom().clone() }

    /// Greatest element of the lattice
    /// * Output: greatest element of the lattice
    fn top(&self) -> SafeElement<Self::Item> { self.ref_top().clone() }

    /// Is safe element the least element of the lattice?
    /// * `safe_element: &SafeElement<Self::Item>` : safe element
    /// * Output: boolean or error
    fn is_bottom(&self, safe_element: &SafeElement<Self::Item>) -> Result<bool, String> {
        match (safe_element,self.ref_bottom()) {
            (
                SafeElement { code: element, lattice_hash },
                SafeElement { code: bottom, lattice_hash: self_hash }
            ) if lattice_hash == self_hash => Ok(element == bottom),
            _ => Err("element is not within lattice".to_string()),
        }
    }

    /// Is safe element the greatest element of the lattice?
    /// * `safe_element: &SafeElement<Self::Item>` : safe element
    /// * Output: boolean or error
    fn is_top(&self, safe_element: &SafeElement<Self::Item>) -> Result<bool, String> {
        match (safe_element,self.ref_top()) {
            (
                SafeElement { code: element, lattice_hash },
                SafeElement { code: top, lattice_hash: self_hash }
            ) if lattice_hash == self_hash => Ok(element == top),
            _ => Err("element is not within lattice".to_string()),
        }
    }
    
    /// Is unsafe element the least element of the lattice?
    /// * `element: &Self::Item` : unsafe element
    /// * Output: boolean or error
    unsafe fn unsafe_is_bottom(&self, element: &Self::Item) -> bool {
        element == &self.ref_bottom().code
    }

    /// Is unsafe element the greatest element of the lattice?
    /// * `element: &Self::Item` : unsafe element
    /// * Output: boolean or error
    unsafe fn unsafe_is_top(&self, element: &Self::Item) -> bool {
        element == &self.ref_top().code
    }
    
    /// Greatest lower bound
    /// * `left: &SafeElement<Self::Item>` : left operand
    /// * `right: &SafeElement<Self::Item>` : right operand
    /// * Output: greatest lower bound or error
    #[inline] fn meet(&self, left: &SafeElement<Self::Item>, right: &SafeElement<Self::Item>) -> Result<SafeElement<Self::Item>, String> {
        let SafeElement { code: element_left, lattice_hash: lattice_hash_left, } = left;
        let SafeElement { code: element_right, lattice_hash: lattice_hash_right, } = right;
        let lattice_hash = self.ref_lattice_hash();
        match (lattice_hash_left == lattice_hash, lattice_hash_right == lattice_hash) {
            (true,true) => Ok(
                SafeElement { code: unsafe { self.unsafe_meet(element_left,element_right) }, lattice_hash: *lattice_hash, }
            ),
            (false,true) => Err("left is not within lattice".to_string()),
            (true,false) => Err("right is not within lattice".to_string()),
            (false,false) => Err("entries are not within lattice".to_string()),
        }
    }

    /// Greatest lower bound of a collection
    /// * `it: I` : collection of safe elements
    /// * `I` : type of collection
    /// * Output: greatest lower bound or error
    #[inline] fn meet_some<'a, I>(&self, it: I) -> Result<SafeElement<Self::Item>, String>
                                    where I: IntoIterator<Item=&'a SafeElement<Self::Item>>, Self: 'a {
        let SafeElement { code: mut cumul, lattice_hash: self_hash } = self.top();
        for (idx,SafeElement { code: element, lattice_hash }) in it.into_iter().enumerate() {
            if *lattice_hash == self_hash {
                cumul = unsafe { self.unsafe_meet(&cumul,element) };
            } else { return Err(format!("element at index {idx} is not within lattice")); }
        }
        Ok(SafeElement { code: cumul, lattice_hash: self_hash })
    }

    /// Least upper bound
    /// * `left: &SafeElement<Self::Item>` : left operand
    /// * `right: &SafeElement<Self::Item>` : right operand
    /// * Output: least upper bound or error
    fn join(&self, left: &SafeElement<Self::Item>, right: &SafeElement<Self::Item>) -> Result<SafeElement<Self::Item>, String> {
        let SafeElement { code: element_left, lattice_hash: lattice_hash_left, } = left;
        let SafeElement { code: element_right, lattice_hash: lattice_hash_right, } = right;
        let lattice_hash = self.ref_lattice_hash();
        match (lattice_hash_left == lattice_hash, lattice_hash_right == lattice_hash) {
            (true,true) => Ok(
                SafeElement { code: unsafe { self.unsafe_join(element_left,element_right) }, lattice_hash: *lattice_hash, }
            ),
            (false,true) => Err("left is not within lattice".to_string()),
            (true,false) => Err("right is not within lattice".to_string()),
            (false,false) => Err("entries are not within lattice".to_string()),
        }
    }

    /// Least upper bound of a collection
    /// * `it: I` : collection of safe elements
    /// * `I` : type of collection
    /// * Output: least upper bound or error
    fn join_some<'a, I>(&self, it: I) -> Result<SafeElement<Self::Item>, String>
                            where I: IntoIterator<Item=&'a SafeElement<Self::Item>>, Self: 'a {
        let SafeElement { code: mut cumul, lattice_hash: self_hash } = self.bottom();
        for (idx,SafeElement { code: element, lattice_hash }) in it.into_iter().enumerate() {
            if *lattice_hash == self_hash {
                cumul = unsafe { self.unsafe_join(&cumul,element) };
            } else { return Err(format!("element at index {idx} is not within lattice")); }
        }
        Ok(SafeElement { code: cumul, lattice_hash: self_hash })
    }

    /// Check if unsafe element is in lattice and then build safe element
    /// * `element: T` : unsafe element
    /// * `T` : type of unsafe element; actually any type which implement `Into<Self::Item>`
    /// * Output: safe element or error
    fn check_safe<T>(&self, element: T) -> Result<SafeElement<Self::Item>, String> where T: Into<Self::Item> {
        let element: Self::Item = element.into();
        if self.contains(&element) { 
            Ok(SafeElement { code: element, lattice_hash: self.lattice_hash(), }) 
        } else { Err("lattice does not contain element".to_string()) }
    }

    /// Elements collection generator (for tests); elements are built safe
    /// * `len: usize` : size of the collection
    /// * `rng: &mut R` : random number generator
    /// * `R: Rng` : type of random number generator
    /// * `I` : type of collection
    /// * Output: collection of random safe elements 
    fn rand_elements<R: Rng,I>(&self, len: usize, rng: &mut R) -> I::Type<SafeElement<Self::Item>> where I: CollectionFamily1 {
        (0..len).map(|_| self.rand_element(rng)).collect()
    }

    /// Init a new assignment builder
    /// * Output: empty assignment builder
    fn assignment(&self,) -> AssignmentBuilder<Self::Item> where Self::Item: Eq + Ord + Hash, {
        let elements = hidden::OrdMap::new();
        AssignmentBuilder { elements, lattice_hash: self.lattice_hash(), length_mid: u32::MAX.slx(), length_max: u32::MAX.slx() }
    }

    /// Init a new assignment builder with capacity
    /// * `capacity: usize` : capacity
    /// * Output: empty assignment builder with capacity `capacity`
    fn assignment_with_capacity(&self, capacity: usize) -> AssignmentBuilder<Self::Item> where Self::Item: Eq + Ord + Hash, {
        let elements = hidden::OrdMap::with_capacity(capacity);
        AssignmentBuilder { elements, lattice_hash: self.lattice_hash(), length_mid: u32::MAX.slx(), length_max: u32::MAX.slx() }
    }

    /// Init a new prunable assignment builder
    /// * `length_mid: u32slx` : max size of pruned assignment
    /// * `length_max: u32slx` : max size of assignment
    /// * Output: empty assignment builder
    fn prunable(&self, length_mid: u32, length_max: u32,) -> AssignmentBuilder<Self::Item> where Self::Item: Eq + Ord + Hash, {
        let elements = hidden::OrdMap::new();
        AssignmentBuilder { elements, lattice_hash: self.lattice_hash(), length_mid: length_mid.slx(), length_max: length_max.slx() }
    }

    /// Init a new prunable assignment builder with capacity
    /// * `length_mid: u32slx` : max size of pruned assignment
    /// * `length_max: u32slx` : max size of assignment
    /// * `capacity: usize` : capacity
    /// * Output: empty assignment builder with capacity `capacity`
    fn prunable_with_capacity(&self, length_mid: u32, length_max: u32, capacity: usize)
                                                                -> AssignmentBuilder<Self::Item> where Self::Item: Eq + Ord + Hash, {
        let elements = hidden::OrdMap::with_capacity(capacity);
        AssignmentBuilder { elements, lattice_hash: self.lattice_hash(), length_mid: length_mid.slx(), length_max: length_max.slx() }
    }

    /// Unsafe test if left and right cover top, ie. union of left and right is top
    /// * `left: &Self::Item` : left operand
    /// * `right: &Self::Item` : right operand
    /// * Output: boolean
    #[inline] unsafe fn unsafe_cover(&self, left: &Self::Item, right: &Self::Item) -> bool {
        &self.unsafe_join(left,right) == &self.ref_top().code
    }

    /// Unsafe test if left and right are disjoint
    /// * `left: &Self::Item` : left operand
    /// * `right: &Self::Item` : right operand
    /// * Output: boolean
    #[inline] unsafe fn unsafe_disjoint(&self, left: &Self::Item, right: &Self::Item) -> bool {
        &self.unsafe_meet(left,right) == &self.ref_bottom().code
    }

    /// Unsafe test if left implies (i.e. is contained by) right
    /// * should be equivalent to implies_meet
    /// * `left: &Self::Item` : left operand
    /// * `right: &Self::Item` : right operand
    /// * Output: boolean
    #[inline] unsafe fn unsafe_implies_join(&self, left: &Self::Item, right: &Self::Item) -> bool {
        &self.unsafe_join(left,right) == right
    }

    /// Unsafe test if left is implied by (i.e. contains) right
    /// * should be equivalent to implied_meet
    /// * `left: &Self::Item` : left operand
    /// * `right: &Self::Item` : right operand
    /// * Output: boolean
    #[inline] unsafe fn unsafe_implied_join(&self, left: &Self::Item, right: &Self::Item) -> bool {
        &self.unsafe_join(left,right) == left
    }
    
    /// Unsafe test if left implies (i.e. is contained by) right
    /// * should be equivalent to implies_join
    /// * `left: &Self::Item` : left operand
    /// * `right: &Self::Item` : right operand
    /// * Output: boolean
    #[inline] unsafe fn unsafe_implies_meet(&self, left: &Self::Item, right: &Self::Item) -> bool {
        &self.unsafe_meet(left,right) == left
    }

    /// Unsafe test if left is implied by (i.e. contains) right
    /// * should be equivalent to implied_joint
    /// * `left: &Self::Item` : left operand
    /// * `right: &Self::Item` : right operand
    /// * Output: boolean
    #[inline] unsafe fn unsafe_implied_meet(&self, left: &Self::Item, right: &Self::Item) -> bool {
        &self.unsafe_meet(left,right) == right
    }

    /// Test if left and right cover top, ie. union of left and right is top
    /// * `left: &SafeElement<Self::Item>` : left operand
    /// * `right: &SafeElement<Self::Item>` : right operand
    /// * Output: boolean or error
    #[inline] fn cover(&self, left: &SafeElement<Self::Item>, right: &SafeElement<Self::Item>) -> Result<bool, String> {
        Ok(&self.join(left,right)? == self.ref_top())
    }

    /// Test if left and right are disjoint
    /// * `left: &SafeElement<Self::Item>` : left operand
    /// * `right: &SafeElement<Self::Item>` : right operand
    /// * Output: boolean or error
    #[inline] fn disjoint(&self, left: &SafeElement<Self::Item>, right: &SafeElement<Self::Item>) -> Result<bool, String> {
        Ok(&self.meet(left,right)? == self.ref_bottom())
    }

    /// Test if left implies (i.e. is contained by) right
    /// * should be equivalent to implies_meet
    /// * `left: &SafeElement<Self::Item>` : left operand
    /// * `right: &SafeElement<Self::Item>` : right operand
    /// * Output: boolean or error
    #[inline] fn implies_join(&self, left: &SafeElement<Self::Item>, right: &SafeElement<Self::Item>) -> Result<bool, String> {
        Ok(&self.join(left,right)? == right)
    }

    /// Test if left is implied by (i.e. contains) right
    /// * should be equivalent to implied_meet
    /// * `left: &SafeElement<Self::Item>` : left operand
    /// * `right: &SafeElement<Self::Item>` : right operand
    /// * Output: boolean or error
    #[inline] fn implied_join(&self, left: &SafeElement<Self::Item>, right: &SafeElement<Self::Item>) -> Result<bool, String> {
        Ok(&self.join(left,right)? == left)
    }
    
    /// Test if left implies (i.e. is contained by) right
    /// * should be equivalent to implies_join
    /// * `left: &SafeElement<Self::Item>` : left operand
    /// * `right: &SafeElement<Self::Item>` : right operand
    /// * Output: boolean or error
    #[inline] fn implies_meet(&self, left: &SafeElement<Self::Item>, right: &SafeElement<Self::Item>) -> Result<bool, String> {
        Ok(&self.meet(left,right)? == left)
    }

    /// Test if left is implied by (i.e. contains) right
    /// * should be equivalent to implied_joint
    /// * `left: &SafeElement<Self::Item>` : left operand
    /// * `right: &SafeElement<Self::Item>` : right operand
    /// * Output: boolean or error
    #[inline] fn implied_meet(&self, left: &SafeElement<Self::Item>, right: &SafeElement<Self::Item>) -> Result<bool, String> {
        Ok(&self.meet(left,right)? == right)
    }
}

/// General trait for complemented lattices
/// * such lattices have logical negation
pub trait ComplementedLattice: Lattice {
    /// Unsafe logical negation
    /// * contract: not has to be bijective, be equal to its inverse, and has to check de Morgan's law
    /// * i.e. double not is identity and not of meet is join of not
    /// * this is unsafe: value is not tested to be within lattice
    /// * `element: &Self::Item` : unsafe element
    /// * Output: logical negation of unsafe element
    unsafe fn unsafe_not(&self, element: &Self::Item) -> Self::Item;

    /// Logical negation
    /// * `safe_element: &SafeElement<Self::Item>` : safe element
    /// * Output: logical negation of safe element
    fn not(&self, safe_element: &SafeElement<Self::Item>) -> Result<SafeElement<Self::Item>, String> {
        let SafeElement { code: element, lattice_hash, } = safe_element;
        let lattice_hash = * lattice_hash;
        if self.lattice_hash() == lattice_hash { 
            Ok(SafeElement { code: unsafe { self.unsafe_not(element) }, lattice_hash, }) 
        } else { Err("element is not within lattice".to_string()) }
    }

    /// Compute the co-assignment of an assignment
    /// * The following computation is done on the weighted elements of the assignment: `(e,w) -> (e.not(), 1 - w)`
    /// * `assignment: &Assignment<Self::Item>` : assigment
    /// * Output: co-assignment
    fn co_assignment(&self, assignment: &Assignment<Self::Item>) -> Result<Assignment<Self::Item>, String> 
                                                                                        where Self::Item: Ord + Hash, {
        let Assignment { lattice_hash, .. } = *assignment;
        let Assignment { elements, .. } = assignment;
        if self.lattice_hash() == lattice_hash {
            let eps = ASSIGNMENT_EPSILON.slx();
            let one = *one_f64slx();
            let zero = *zero_f64slx();
            let mut co_assignment = HashMap::with_capacity(elements.len());
            for (element, weight) in elements {
                if *weight - one > eps { return Err(format!("assigned weight {weight} is greater than 1.0")); }
                co_assignment.insert(unsafe { self.unsafe_not(element) }, (one - *weight).max(zero));
            }    
            Ok(Assignment { elements: co_assignment, lattice_hash }) 
        } else { Err("assignment is not within lattice".to_string()) }
    }
}

macro_rules! impl_as_ref {
    ($($ty: ident,)*) => {
        $(
            impl<L> Lattice for $ty<L> where L: Lattice, {
                type Item = L::Item;
                fn rand_lattice<R: Rng>(rng: &mut R) -> Self { $ty::new(L::rand_lattice(rng)) }
                fn rand_element<R: Rng>(&self, rng: &mut R) -> SafeElement<Self::Item> { 
                    let SafeElement { code, lattice_hash } = self.as_ref().rand_element(rng);
                    SafeElement { code, lattice_hash, }
                }
                fn ref_lattice_hash(&self) -> &u128slx { self.as_ref().ref_lattice_hash() }
                fn contains(&self, element: &Self::Item) -> bool { self.as_ref().contains(element) }
                fn ref_bottom(&self) -> &SafeElement<Self::Item> { self.as_ref().ref_bottom() }
                fn ref_top(&self) -> &SafeElement<Self::Item> { self.as_ref().ref_top() }
                unsafe fn unsafe_meet(&self, element_left: &Self::Item, element_right: &Self::Item) -> Self::Item {
                    self.as_ref().unsafe_meet(element_left, element_right)
                }
                unsafe fn unsafe_join(&self, element_left: &Self::Item, element_right: &Self::Item) -> Self::Item {
                    self.as_ref().unsafe_join(element_left, element_right)
                }
                fn from_str(&self, s: &str) -> Result<SafeElement<Self::Item>,String> {
                    self.as_ref().from_str(s)
                }
                fn to_string(&self, element: &SafeElement<Self::Item>,) -> Result<String,String> {
                    self.as_ref().to_string(element,)
                } 
            }
        )*
    };
}

impl_as_ref!(Arc,Rc,Box,);