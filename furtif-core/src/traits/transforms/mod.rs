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


use std::{hash::Hash, collections::HashMap, };

// #[cfg(not(feature = "silx-types"))] use crate::fake_slx::{f64slx, u32slx,  FakeSlx};
// #[cfg(feature = "silx-types")] use silx_types::{ f64slx, u32slx, Float, IntoSlx, SlxInto};

#[cfg(feature = "silx-types")] use silx_types::Float;

use crate::{
    types::{ u32slx, f64slx, SlxInto, IntoSlx, },
    structs::{Assignment, SafeElement, ASSIGNMENT_EPSILON, one_f64slx, zero_f64slx}, 
    traits::Lattice,
};

use super::ComplementedLattice;

/// General trait for iterable lattices, ie. lattice with some useful iterators
pub trait IterableLattice: Lattice {
    /// Type of iterator for iterating lattice from bottom to top 
    type IntoIterUp: Iterator<Item = Self::Item>;
    /// Type of iterator for iterating lattice from top to bottom 
    type IntoIterDown: Iterator<Item = Self::Item>;

    /// Unsafe iterator of the full lattice, non decreasing with inference; 
    /// * error means that the iterator cannot be iterated (e.g. lattice with too much elements)
    /// * Output: iterator or error
    unsafe fn unsafe_bottom_to_top(&self) -> Result<Self::IntoIterUp,String>;
    
    /// Unsafe iterator of the full lattice, non increasing with inference; 
    /// * error means that the iterator cannot be iterated (e.g. lattice with too much elements)
    /// * Output: iterator or error
    unsafe fn unsafe_top_to_bottom(&self) -> Result<Self::IntoIterDown,String>;
    
    /// Safe iterator of the full lattice, non decreasing with inference; 
    /// * error means that the iterator cannot be iterated (e.g. lattice with too much elements)
    /// * Output: iterator or error
    fn bottom_to_top(&self) -> Result<std::vec::IntoIter<SafeElement<Self::Item>>,String> {
        let lattice_hash = self.lattice_hash();
        Ok(unsafe{ self.unsafe_bottom_to_top() }?.map(
            move |element| SafeElement { lattice_hash, code: element } 
        ).collect::<Vec<_>>().into_iter())
    }

    /// Safe iterator of the full lattice, non increasing with inference; 
    /// * error means that the iterator cannot be iterated (e.g. lattice with too much elements)
    /// * Output: iterator or error
    fn top_to_bottom(&self) -> Result<std::vec::IntoIter<SafeElement<Self::Item>>,String> {
        let lattice_hash = self.lattice_hash();
        Ok(unsafe{ self.unsafe_top_to_bottom() }?.map(
            move |element| SafeElement { lattice_hash, code: element } 
        ).collect::<Vec<_>>().into_iter())
    }    
}

/// General trait for lattices with leaves.
/// * Leaves are minimal non-empty elements which are generators for the lattice by means of the disjunction operator
/// * Typically, Powerset and Taxonomy have leaves, but hyperpowerset (not implemented here) does not have leaves
pub trait LatticeWithLeaves: Lattice where Self::Item: Ord + Hash, {
    /// Type of leaves iterator
    type IntoIterLeaves: Iterator<Item = (Self::Item,f64slx)>;

    /// Get unsafe weighted leaf at rank `u`
    /// * `u: usize` : leaf rank
    /// * Output: unsafe weighted leaf or error
    unsafe fn unsafe_weighted_leaf(&self, u: usize) -> Result<(&Self::Item,&f64slx),String>;

    /// Unsafe iterator of the weighted leaves of the lattice
    /// * Output: unsafe weighted leaf iterator or error
    unsafe fn unsafe_leaves(&self) -> Result<Self::IntoIterLeaves,String>;

    /// Get unsafe leaf at rank `u`
    /// * `u: usize` : leaf rank
    /// * Output: unsafe leaf or error
    unsafe fn unsafe_leaf(&self, u: usize) -> Result<&Self::Item,String> {
        Ok(&self.unsafe_weighted_leaf(u)?.0)
    }

    /// Get safe weighted leaf at rank `u`
    /// * `u: usize` : leaf rank
    /// * Output: safe weighted leaf or error
    fn weighted_leaf(&self, u: usize) -> Result<(SafeElement<Self::Item>,f64slx),String> {
        let (e,w) = unsafe { self.unsafe_weighted_leaf(u)? };
        let lattice_hash = self.lattice_hash();
        Ok((SafeElement { code: e.clone(), lattice_hash, },*w))
    }

    /// Get safe leaf at rank `u`
    /// * `u: usize` : leaf rank
    /// * Output: safe leaf or error
    fn leaf(&self, u: usize) -> Result<SafeElement<Self::Item>,String> {
        let e = unsafe { self.unsafe_leaf(u)? };
        let lattice_hash = self.lattice_hash();
        Ok(SafeElement { code: e.clone(), lattice_hash, })    
    }

    /// Iterator of the safe weighted leaves of the lattice
    /// * Output: safe weighted leaf iterator or error
    fn leaves(&self) -> Result<std::vec::IntoIter<(SafeElement<Self::Item>,f64slx)>,String> {
        let lattice_hash = self.lattice_hash();
        Ok(unsafe{ self.unsafe_leaves() }?.map(
            move |(element,w)| (SafeElement { lattice_hash, code: element },w) 
        ).collect::<Vec<_>>().into_iter())
    }    

    /// Transform mass to pignistic probability
    /// * `mass: &Assignment<Self::Item>` : mass assignment
    /// * Output: pignistic assignment or error
    fn mass_to_pignistic(&self, mass: &Assignment<Self::Item>) -> Result<Assignment<Self::Item>,String> {
        let Assignment { lattice_hash, elements, .. } = mass;
        if lattice_hash == self.ref_lattice_hash() {
            let leaves = unsafe { self.unsafe_leaves() }?.collect::<HashMap<_,_>>();
            let loc_norm = elements.iter().map(|(x,wx)| {
                let loc_leaves = leaves.iter().filter(
                    |(y,_)|unsafe { self.unsafe_implied_join(x,*y) }
                ).map(|(y,w)| (y,*w)).collect::<Vec<_>>();
                let norm = loc_leaves.iter().map(|(_,w)| *w).sum::<f64slx>();
                (*wx,loc_leaves,norm)
            }).collect::<Vec<_>>();
            let mut pign = self.assignment();
            for (wx,ll,nx) in loc_norm {
                for (l,w) in ll {
                    unsafe { pign.unsafe_push(l.clone(), wx * w / nx)? };
                }
            }
            pign.normalize()?; Ok(pign.into())
        } else { Err("Mismatching lattice hash".to_string()) }
    }
}

/// Trait implementing belief functions transforms
pub trait BeliefTransform: IterableLattice where Self::Item: Ord + Hash, {
    /// Transform mass to commonality
    /// * `mass: &Assignment<Self::Item>` : mass assignment
    /// * Output: commonality assignment or error
    fn mass_to_commonality<>(&self, mass: &Assignment<Self::Item>) -> Result<Assignment<Self::Item>,String> {
        let Assignment { lattice_hash, elements, .. } = mass;
        if lattice_hash == self.ref_lattice_hash() {
            let vec_com = unsafe{ self.unsafe_top_to_bottom() }?
                            .map(|x| {
                                let mut wx = *zero_f64slx();
                                for (y,wy) in elements {
                                    if unsafe{ self.unsafe_implies_join(&x, y) } { wx += *wy; }
                                }
                                (x, wx)
                            }).filter(|(_,w)| w > &zero_f64slx()).collect::<Vec<_>>();
            let elements = vec_com.clone().into_iter().collect();
            Ok(Assignment { 
                lattice_hash: *lattice_hash, elements,
            })
        } else { Err("Mismatching lattice hash".to_string()) }

    }

    /// Transform commonality to mass
    /// * `commonality: &Assignment<Self::Item>` : commonality assignment
    /// * Output: mass assignment or error
    fn mass_from_commonality(&self, commonality: &Assignment<Self::Item>) -> Result<Assignment<Self::Item>,String> {
        let Assignment { lattice_hash, elements, .. } = commonality;
        if lattice_hash == self.ref_lattice_hash() {
            let mut mass = self.assignment();
            let zero = *zero_f64slx();
            for x in unsafe{ self.unsafe_top_to_bottom() }? {
                let mut wx = match elements.get(&x) {
                    Some(w) => *w, None => zero,
                };            
                for (y,wy) in &mass.elements.elements {
                    if unsafe{ self.unsafe_implies_join(&x, y) } { wx -= *wy; }
                }
                if wx.abs() > ASSIGNMENT_EPSILON.slx() { unsafe { mass.unsafe_push(x, wx) }?; }
            }
            mass.length_mid = (mass.elements.len() as u32).slx();
            let slx2u32: u32slx = 2u32.slx();
            mass.length_max = slx2u32 * mass.length_mid;
            mass.normalize()?;
            Ok(mass.into())
        } else { Err("Mismatching lattice hash".to_string()) }
    }

    /// Transform mass to implicability
    /// * `mass: &Assignment<Self::Item>` : mass assignment
    /// * Output: implicability assignment or error
    fn mass_to_implicability(&self, mass: &Assignment<Self::Item>) -> Result<Assignment<Self::Item>,String> {
        let Assignment { lattice_hash, elements, .. } = mass;
        if lattice_hash == self.ref_lattice_hash() {
            let vec_com = unsafe{ self.unsafe_bottom_to_top() }?
                            .map(|x| {
                                let mut wx = *zero_f64slx();
                                for (y,wy) in elements {
                                    if unsafe{ self.unsafe_implied_join(&x, y) } { wx += *wy; }
                                }
                                (x, wx)
                            }).filter(|(_,w)| w > zero_f64slx()).collect::<Vec<_>>();
            let elements = vec_com.clone().into_iter().collect();
            Ok(Assignment { lattice_hash: *lattice_hash, elements, })
        } else { Err("Mismatching lattice hash".to_string()) }
    }
    
    /// Transform implicability to mass
    /// * `implicability: &Assignment<Self::Item>` : implicability assignment 
    /// * Output: mass assignment or error
    fn mass_from_implicability(&self, implicability: &Assignment<Self::Item>) -> Result<Assignment<Self::Item>,String> {
        let Assignment { lattice_hash, elements, .. } = implicability;
        if lattice_hash == self.ref_lattice_hash() {
            let mut mass = self.assignment();
            let zero = *zero_f64slx();
            for x in unsafe{ self.unsafe_bottom_to_top() }? {
                let mut wx = match elements.get(&x) {
                    Some(w) => *w, None => zero,
                };
                for (y,wy) in &mass.elements.elements {
                    if unsafe{ self.unsafe_implied_join(&x, y) } { wx -= *wy; }
                }
                if wx.abs() > ASSIGNMENT_EPSILON.slx() { unsafe { mass.unsafe_push(x, wx) }?; }
            }
            mass.length_mid = (mass.elements.len() as u32).slx();
            let slx2u32: u32slx = 2u32.slx();
            mass.length_max = slx2u32 * mass.length_mid;

            mass.normalize()?;
            Ok(mass.into())
        } else { Err("Mismatching lattice hash".to_string()) }
    }
    
    /// Transform mass to credibility
    /// * `mass: &Assignment<Self::Item>` : mass assignment
    /// * Output: credibility assignment or error
    fn mass_to_credibility(&self, mass: &Assignment<Self::Item>) -> Result<Assignment<Self::Item>,String> {
        let Assignment { lattice_hash, elements, .. } = mass;
        if lattice_hash == self.ref_lattice_hash() {
            let vec_com = unsafe{ self.unsafe_bottom_to_top() }?
                            .map(|x| {
                                let mut wx = *zero_f64slx();
                                for (y,wy) in elements.iter()
                                        .filter(|(x,_)| !unsafe { self.unsafe_is_bottom(*x) }) {
                                    if unsafe{ self.unsafe_implied_join(&x, y) } { wx += *wy; }
                                }
                                (x, wx)
                            }).filter(|(_,w)| w > zero_f64slx()).collect::<Vec<_>>();
            let elements = vec_com.clone().into_iter().collect();
            Ok(Assignment { lattice_hash: *lattice_hash, elements, })
        } else { Err("Mismatching lattice hash".to_string()) }
    }
    
    /// Transform credibility to mass
    /// * `credibility: &Assignment<Self::Item>` : credibility assignment
    /// * Output: mass assignment or error
    fn mass_from_credibility(&self, credibility: &Assignment<Self::Item>) -> Result<Assignment<Self::Item>,String> {
        let Assignment { lattice_hash, elements, .. } = credibility;
        if lattice_hash == self.ref_lattice_hash() {
            let mut mass = self.assignment();
            let zero = *zero_f64slx();
            let mut full_w = zero;
            for x in unsafe{ self.unsafe_bottom_to_top() }? {
                let mut wx = match elements.get(&x) {
                    Some(w) => *w, None => zero,
                };
                for (y,wy) in &mass.elements.elements {
                    if unsafe{ self.unsafe_implied_join(&x, y) } { wx -= *wy; }
                }
                if wx.abs() > ASSIGNMENT_EPSILON.slx() { 
                    unsafe { mass.unsafe_push(x, wx) }?; 
                    full_w += wx;
                }

            }
            if full_w > *one_f64slx() { panic!("exceeding weights"); }
            unsafe { mass.unsafe_push(self.bottom().code, *one_f64slx() - full_w)?;  }
            mass.length_mid = (mass.elements.len() as u32).slx();
            let slx2u32: u32slx = 2u32.slx();
            mass.length_max = slx2u32 * mass.length_mid;
            mass.normalize()?;
            Ok(mass.into())
        } else { Err("Mismatching lattice hash".to_string()) }
    }
    
    /// Transform mass to plausibility
    /// * `mass: &Assignment<Self::Item>` : mass assignment
    /// * Output: plausibility assignment or error
    fn mass_to_plausibility(&self, mass: &Assignment<Self::Item>) -> Result<Assignment<Self::Item>,String> {
        let Assignment { lattice_hash, elements, .. } = mass;
        if lattice_hash == self.ref_lattice_hash() {
            let vec_com = unsafe{ self.unsafe_bottom_to_top() }?
                            .map(|x| {
                                let mut wx = *zero_f64slx();
                                for (y,wy) in elements {
                                    if !unsafe{ self.unsafe_disjoint(&x, y) } { wx += *wy; }
                                }
                                (x, wx)
                            }).filter(|(_,w)| w > zero_f64slx()).collect::<Vec<_>>();
            let elements = vec_com.clone().into_iter().collect();
            Ok(Assignment { lattice_hash: *lattice_hash, elements, })
        } else { Err("Mismatching lattice hash".to_string()) }
    }


    /// Transform implicability to credibility
    /// * `implicability: &Assignment<Self::Item>` : implicability assignment
    /// * Output: credibility assignment or error
    fn implicability_to_credibility(&self, implicability: &Assignment<Self::Item>) -> Result<Assignment<Self::Item>,String> {
        let Assignment { lattice_hash, elements, .. } = implicability;
        let SafeElement { code: bottom, lattice_hash: ref_lattice_hash } = self.ref_bottom();
        if lattice_hash == ref_lattice_hash { Ok( {
            let zero = *zero_f64slx();
            let neg_shift = match elements.get(&bottom) {
                Some(w) => *w, None => zero,
            };
            let mapped = elements.iter()
                .map(|(x,w)| (x.clone(), *w - neg_shift)).collect::<Vec<_>>();
            for (_,w) in &mapped { 
                let w = (*w).unslx();
                if !w.is_finite() || w.is_sign_negative() {
                    return Err(format!("mapped weight {w} is not finite or is sign negative"));
                } 
            }
            let elements = mapped.into_iter().filter(|(_,w)| (*w).unslx() > ASSIGNMENT_EPSILON).collect();
            Assignment { elements, lattice_hash: *lattice_hash }
        } ) } else { Err("Mismatching lattice hash".to_string()) }
    }

    /// Transform credibility to implicability
    /// * `credibility: &Assignment<Self::Item>` : credibility assignment
    /// * Output: implicability assignment or error
    fn implicability_from_credibility(&self, credibility: &Assignment<Self::Item>) -> Result<Assignment<Self::Item>,String> {
        let Assignment { lattice_hash, elements, } = credibility;
        let SafeElement { code: bottom, lattice_hash: ref_lattice_hash } = self.ref_bottom();
        let SafeElement { code: top, .. } = self.ref_top();
        let one = *one_f64slx();
        let slx_assignment_epsilon: f64slx = ASSIGNMENT_EPSILON.slx();
        if lattice_hash == ref_lattice_hash { Ok( {
            let shift = match elements.get(&top) {
                Some(w) => one - *w, None => one,
            };
            if shift <= - slx_assignment_epsilon { return Err("weight on top is greater than 1.0".to_string()) }
            {
                if let Some(w_bottom) = elements.get(&bottom) {
                    return Err(format!("weight on bottom is {w_bottom} not 0.0")) 
                }
            }
            // shifted prefix iterator
            let zero_weighted = unsafe { self.unsafe_bottom_to_top() }?
                .filter(|e| !elements.contains_key(e))
                .map(|e|(e,shift));
            // deep clone of elements shifted and prefixed
            let weighted = zero_weighted.chain(
                elements.clone().into_iter().map(|(x,w)| (x,w + shift))
            ).collect::<Vec<_>>();
            for (_,w) in &weighted { if !w.is_finite() || *w <= -slx_assignment_epsilon {
                return Err(format!("mapped weight {w} is not finite or is sign negative"))
            } }
            let weighted_filtered = weighted.into_iter().filter(|(_,w)| w > &slx_assignment_epsilon).collect::<Vec<_>>();
            // collect on light clone
            let elements = weighted_filtered.clone().into_iter().collect::<HashMap<_,_>>();
            Assignment { lattice_hash: *lattice_hash, elements, }
        } ) } else { Err("Mismatching lattice hash".to_string()) }
    }
}

/// Trait defining transformations based on the complementation
pub trait ComplementedBeliefTransform: IterableLattice + ComplementedLattice where Self::Item: Eq + Ord + Hash + Clone, {    
    /// Transform plausibility to mass
    /// * `plausibility: &Assignment<Self::Item>` : plausibility assignment
    /// * Output: mass assignment or error
    fn mass_from_plausibility(&self, plausibility: &Assignment<Self::Item>) -> Result<Assignment<Self::Item>,String> {
        let Assignment { lattice_hash, elements, .. } = plausibility;
        if lattice_hash == self.ref_lattice_hash() {
            let mut mass = self.assignment();
            let one = *one_f64slx();
            for x in unsafe{ self.unsafe_top_to_bottom() }? {
                let neg_x = unsafe { self.unsafe_not(&x) };
                let mut wx = match elements.get(&x) {
                    Some(w) => one - *w, None => one,
                }; //implicability of neg_x
                for (y,wy) in &mass.elements.elements {
                    if unsafe{ self.unsafe_implied_join(&neg_x, y) } { wx -= *wy; }
                }
                if wx.abs() > ASSIGNMENT_EPSILON.slx() { unsafe { mass.unsafe_push(neg_x, wx) }?; }
            }
            mass.length_mid = (mass.elements.len() as u32).slx();
            let slx2u32: u32slx = 2u32.slx();
            mass.length_max = slx2u32 * mass.length_mid;
            mass.normalize()?;
            Ok(mass.into())
        } else { Err("Mismatching lattice hash".to_string()) }
    }
}

impl<L> BeliefTransform for L where L: IterableLattice, Self::Item: Ord + Hash, { }

impl<L> ComplementedBeliefTransform for L 
            where L: IterableLattice + ComplementedLattice, Self::Item: Ord + Hash, { }


pub mod experiment {
    // #[cfg(not(feature = "silx-types"))] use crate::fake_slx::FakeSlx;
    // #[cfg(feature = "silx-types")] use silx_types::IntoSlx;

    use crate::{
        types::IntoSlx,
        structs::Powerset, 
        traits::{ Lattice, BeliefTransform, ComplementedBeliefTransform, LatticeWithLeaves, }
    };

    /// Experimentation with assignment transforms
    pub fn exp_transform() -> Result<(),String> {
        println!("======================= transform =====");
        let lattice = Powerset::new(3,1024)
                .expect("unexpected powwerset initialisation failure")
                .set_iterators();
        let (mut m1,mut m2) = (
            lattice.assignment(),
            lattice.assignment(),
        );
        let (prop_a, m_a,) = (lattice.leaf(0)?, 0.1.slx());
        let (prop_b, m_b) = (lattice.leaf(1)?, 0.15.slx(),);
        let (prop_c, m_c) = (lattice.leaf(2)?, 0.2.slx());
        let (prop_bc, m_bc) = (lattice.join(&prop_b,&prop_c)?,0.2.slx());
        let (prop_ca, m_ca) = (lattice.join(&prop_c,&prop_a)?,0.1.slx());
        let (prop_ab, m_ab) = (lattice.join(&prop_a,&prop_b)?,0.25.slx());
        let (prop_abc, m_abc) = (lattice.join(&prop_bc,&prop_a)?,0.0.slx());
        m1.push(prop_a,m_a)?;
        m1.push(prop_b,m_b)?;
        m1.push(prop_c,m_c)?;
        m1.push(prop_bc,m_bc)?;
        m1.push(prop_ca,m_ca)?;
        m1.push(prop_ab,m_ab)?;
        m1.push(prop_abc,m_abc)?;
        let m1 = m1.into();
        //
        let (prop_bot, m_bot,) = (lattice.meet(&prop_a,&prop_b)?, 0.05.slx());
        let m_bc = 0.15.slx();
        let m_ab = 0.15.slx();
        let m_abc = 0.1.slx();
        m2.push(prop_bot,m_bot)?;
        m2.push(prop_a,m_a)?;
        m2.push(prop_b,m_b)?;
        m2.push(prop_c,m_c)?;
        m2.push(prop_bc,m_bc)?;
        m2.push(prop_ca,m_ca)?;
        m2.push(prop_ab,m_ab)?;
        m2.push(prop_abc,m_abc)?;
        let m2 = m2.into();
        for (m,nm) in [(m1,"m1"),(m2,"m2")] {
            println!("===================== transform {nm}=====");
            println!("------------------- implicability -----");
            println!("m: {:?}", m);
            let implicability = lattice.mass_to_implicability(&m)?;
            println!("implicability: {:?}",implicability);
            let back_m = lattice.mass_from_implicability(&implicability)?;
            println!("back_m -> {:?}",back_m);
            println!("--------------------- commonality -----");
            println!("m: {:?}", m);
            let commonality = lattice.mass_to_commonality(&m)?;
            println!("commonality: {:?}",commonality);
            let back_m = lattice.mass_from_commonality(&commonality)?;
            println!("back_m -> {:?}",back_m);
            println!("--------------------- credibility -----");
            println!("m: {:?}", m);
            let credibility = lattice.mass_to_credibility(&m)?;
            println!("credibility: {:?}",credibility);
            let back_m = lattice.mass_from_credibility(&credibility)?;
            println!("back_m -> {:?}",back_m);
            let implicability = lattice.implicability_from_credibility(&credibility)?;
            println!("implicability: {:?}",implicability);
            let back_credibility = lattice.implicability_to_credibility(&implicability)?;
            println!("back_credibility: {:?}",back_credibility);
            println!("-------------------- plausibility -----");
            println!("m: {:?}", m);
            let plausibility = lattice.mass_to_plausibility(&m)?;
            println!("plausibility: {:?}",plausibility);
            // ComplementedLattice is necessary here:
            let back_m = lattice.mass_from_plausibility(&plausibility)?;
            println!("back_m -> {:?}",back_m);
            println!("----------------------- pignistic -----");
            println!("m: {:?}", m);
            let pignistic = lattice.mass_to_pignistic(&m)?;
            println!("pignistic: {:?}", pignistic);
            println!();            
        }
        Ok(())
    }
}