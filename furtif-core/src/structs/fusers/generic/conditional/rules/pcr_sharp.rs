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


use std::hash::Hash;

use silx_types::f64slx;

use crate::{
    traits::{ Referee, Lattice, },
    structs::{Assignment, SafeArray, hidden::OrdMap, one_f64slx, },
};

use hashed_type_def::HashedTypeDef;
use rkyv::{Archive, Serialize as RkyvSerialize, Deserialize as RkyvDeserialize, };
use serde::{Serialize as SerdeSerialize, Deserialize as SerdeDeserialize};

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, HashedTypeDef, SerdeSerialize, SerdeDeserialize, 
    Copy, Clone, Debug
)]
/// PCR# referee function
pub struct PcrSharp {
    max_entries: u8,
}

const _MAX_ENTRIES_: u8 = 16;
impl PcrSharp {
    /// Constructor of PCR# referee function
    /// * `max_entries: u8` : maximum number of assignments that could be fused
    /// * Output: PCR# referee function or error
    pub fn new(max_entries: u8,) -> Result<Self,String> {
        if max_entries <= _MAX_ENTRIES_ { Ok(Self { max_entries }) } else {
            Err(format!("max_entries is required to be smaller than {_MAX_ENTRIES_}"))
        }
    }
    fn build_double_sequence_up(nb_leaves: u8) -> Vec<Vec<u128>> {
        let leaves = (0..nb_leaves).map(|u| (1u128 << u,())).collect::<Vec<_>>();
        let mut double_sequence = vec![vec![0u128]];
        for (n,(leaf,_)) in leaves.iter().enumerate() {
            let mut next_ds = Vec::with_capacity(n+1);
            next_ds.push(vec![0u128]);
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

    fn join<L>(lattice: &L, u:u128, bbas: &[&Assignment<L::Item>], product: &Vec<&L::Item>) -> (L::Item,Option<f64slx>) where L: Lattice, L::Item: Eq + Ord + Hash, {
        let mut element = lattice.top().code;
        for (i,x) in product.iter().enumerate() {
            let single = 1u128 << i;
            if (single & u) == single { 
                element = unsafe { lattice.unsafe_meet(&element, x) }; 
            }
        }
        if unsafe { lattice.unsafe_is_bottom(&element) } { (element,None) } else {
            let mut weight = *one_f64slx();
            for (i,w) in product.iter().zip(bbas)
                    .map(|(x,m)| m.elements[*x]).enumerate() {
                let single = 1u128 << i;
                if (single & u) == single { weight *= w; }
            } (element,Some(weight))
        }
    }
}

impl Referee for PcrSharp {
    fn is_allowed<L>(&self, _lattice: &L, bbas: &[&Assignment<L::Item>]) -> bool where L: Lattice, L::Item: Eq + Ord + Hash, {
        bbas.len() <= self.max_entries as usize
    }

    unsafe fn unsafe_from_conditions<L>(&self, lattice: &L, bbas: &[&Assignment<L::Item>], conditions: SafeArray<L::Item>)
            -> Result<Assignment<L::Item>,String> where L: Lattice, L::Item: Eq + Ord + Hash, {
        let SafeArray { product, lattice_hash, } = conditions;
        let double_sequence = Self::build_double_sequence_up(bbas.len() as u8);
//        let length_mid = u32::MAX.slx(); let length_max = u32::MAX.slx();
        for sequence in double_sequence.into_iter().rev() {
            let sequence_valid = sequence.into_iter()
                .map(|u| Self::join(lattice, u, bbas, &product))
                .filter_map(|(x,ow)| ow.map(|w|(x,w))).collect::<Vec<_>>();
            if !sequence_valid.is_empty() {
                // let mut unnormed = sequence_valid.into_iter()
                //     .map(|(x,w)|(Rc::new(x.as_ref().clone()),w))
                //     .collect::<Vec<_>>();
                let norm = sequence_valid.iter().map(|(_,w)|*w).sum::<f64slx>();
                let mut elements = OrdMap::new();
                for (x,w) in sequence_valid { elements.push(x, w / norm); }
                // for (_,w) in &mut unnormed { *w /= norm; }
                // let elements = unnormed.clone().into_iter().collect();
                // let ord_elements = unnormed.into_iter().map(|xw|OrdData(xw)).collect();
                // let elements = OrdMap { elements, ord_elements };
                return Ok(Assignment { elements: elements.elements, lattice_hash, })
            }
        }
        Err("no consensus found (full contradiction on all sources?)".to_string())
    }
}


pub mod experiment {
    use silx_types::IntoSlx;

    use crate::{
        structs::{Powerset, DiscountedFuser, PcrSharp, Assignment, }, 
        traits::{Lattice, DiscountedFusion, LatticeWithLeaves, }
    };

    /// Experimentation with the PCR# rule
    pub fn exp_pcr_sharp() -> Result<(),String> {
        println!("================= PCR# ================");
        let lattice = Powerset::new(3,1024)
                .expect("unexpected powwerset initialisation failure");
        let (mut m1, mut m2, mut m3) = (
            lattice.assignment(),
            lattice.assignment(),
            lattice.assignment(),
        );
        let (prop_a, m_a) = (lattice.leaf(0)?, 0.3.slx());
        let (prop_b, m_b) = (lattice.leaf(1)?, 0.4.slx());
        let (prop_c, m_c) = (lattice.leaf(2)?, 0.5.slx());
        let (prop_ab, m_ab) = (lattice.join(&prop_a,&prop_b)?,0.5.slx());
        let (prop_bc, m_bc) = (lattice.join(&prop_b,&prop_c)?,0.7.slx());
        let (prop_ca, m_ca) = (lattice.join(&prop_c,&prop_a)?,0.6.slx());
        m1.push(prop_a,m_a)?;
        m1.push(prop_bc,m_bc)?;
        m2.push(prop_b,m_b)?;
        m2.push(prop_ca,m_ca)?;
        m3.push(prop_c,m_c)?;
        m3.push(prop_ab,m_ab)?;
        let m1 = m1.into(); 
        let m2 = m2.into(); 
        let m3 = m3.into();
        let engine = DiscountedFuser::new(512..=1024);
        let referee = PcrSharp::new(16)?;
        let (fused,z) = engine.fuse(&lattice, &referee,&[&m1,&m2,&m3]).expect("unexpected fusion failure");
        let mut fused_theoretic = lattice.assignment();
        // A,B,C -> 1: A,B,C
        let weight = m_a * m_b * m_c / (m_a + m_b + m_c);
        fused_theoretic.push(prop_a,m_a * weight)?;
        fused_theoretic.push(prop_b,m_b * weight)?;
        fused_theoretic.push(prop_c,m_c * weight)?;
        // A,B,AB -> 2: A,B
        let weight = m_a * m_b * m_ab / (m_a * m_ab + m_b * m_ab);
        fused_theoretic.push(prop_a,m_a * m_ab * weight)?;
        fused_theoretic.push(prop_b,m_b * m_ab * weight)?;
        // A,CA,C -> 2: A,C
        let weight = m_a * m_ca * m_c / (m_a * m_ca + m_c * m_ca);
        fused_theoretic.push(prop_a,m_a * m_ca * weight)?;
        fused_theoretic.push(prop_c,m_c * m_ca * weight)?;
        // A,CA,AB -> 3: A
        fused_theoretic.push(prop_a,m_a*m_ab*m_ca)?;
        // BC,B,C -> 2: B,C
        let weight = m_bc * m_b * m_c / (m_bc * m_b + m_bc * m_c);
        fused_theoretic.push(prop_b,m_bc * m_b * weight)?;
        fused_theoretic.push(prop_c,m_bc * m_c * weight)?;
        // BC,B,AB -> 3: B
        fused_theoretic.push(prop_b,m_b*m_ab*m_bc)?;
        // BC,CA,C -> 3: C
        fused_theoretic.push(prop_c,m_c*m_bc*m_ca)?;
        // BC,CA,AB -> 2: A,B,C
        let weight = m_bc * m_ca * m_ab / (m_ab * m_bc + m_ca * m_bc + m_ab * m_ca);
        fused_theoretic.push(prop_b,m_ab * m_bc * weight)?;
        fused_theoretic.push(prop_c,m_ca * m_bc * weight)?;
        fused_theoretic.push(prop_a,m_ab * m_ca * weight)?;
        let fused_theoretic: Assignment<_> = fused_theoretic.into();
        println!("ms: {:?}",[&m1,&m2,&m3]);
        println!("fused: {:?}",fused);
        println!("z -> {z}");
        println!("fused_theoretic: {:?}",fused_theoretic);
        println!();
        Ok(())
    }
}
