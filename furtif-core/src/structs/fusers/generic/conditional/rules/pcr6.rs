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


use std::{ hash::Hash, iter::once, };

use silx_types::f64slx;

use crate::{
    traits::{ Referee, Lattice, },
    structs::{Assignment, SafeArray, hidden::OrdMap, one_f64slx},
};

use hashed_type_def::HashedTypeDef;
use rkyv::{Archive, Serialize as RkyvSerialize, Deserialize as RkyvDeserialize, };
use serde::{Serialize as SerdeSerialize, Deserialize as SerdeDeserialize};

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, HashedTypeDef, SerdeSerialize, SerdeDeserialize, 
    Copy, Clone, Debug
)]
/// Pcr6 referee function
pub struct Pcr6;

impl Referee for Pcr6 {
    fn is_allowed<L>(&self, _lattice: &L, _bbas: &[&Assignment<L::Item>]) -> bool 
                                                where L: Lattice, L::Item: Eq + Ord + Hash, {
        true // always defined
    }

    unsafe fn unsafe_from_conditions<L>(&self, 
        lattice: &L, bbas: &[&Assignment<L::Item>], conditions: SafeArray<L::Item>
    ) -> Result<Assignment<L::Item>,String> where L: Lattice, L::Item: Eq + Ord + Hash, {
        let SafeArray { product, lattice_hash, } = conditions;
        let top = lattice.top().code;
        let meet = product.iter()
            .map(|e| *e).fold(top, 
                |acc,e| unsafe { lattice.unsafe_meet(&acc,e) }
            );
        if !unsafe { lattice.unsafe_is_bottom(&meet) } {
            let elements = once((meet.clone(),*one_f64slx())).collect();
            Ok(Assignment { elements, lattice_hash, })
        } else {
            let unnormed = product.iter().zip(bbas)
                .map(|(x,m)|(*x,m.elements[*x]))
                .collect::<Vec<_>>();
            let norm = unnormed.iter().map(|(_,w)|*w).sum::<f64slx>();
            let mut elements = OrdMap::new();
            for (x,w) in unnormed { elements.push(x.clone(), w / norm); }
            Ok(Assignment { elements: elements.elements, lattice_hash, })
        }
    }
}

pub mod experiment {
    use silx_types::IntoSlx;

    use crate::{
        structs::{Powerset, DiscountedFuser, Pcr6, Assignment, }, 
        traits::{Lattice, DiscountedFusion, LatticeWithLeaves, }
    };

    /// Experimentation with the PCR6 rule
    pub fn exp_pcr6() -> Result<(),String> {
        println!("================= PCR6 ================");
        let lattice = Powerset::new(3,1024)
                .expect("unexpected powwerset initialisation failure");
        let (mut m1, mut m2, mut m3) = (
            lattice.assignment_with_capacity(2),
            lattice.assignment_with_capacity(2),
            lattice.assignment_with_capacity(2),
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
        let referee = Pcr6;
        let (fused,z) = engine.fuse(&lattice, &referee,&[&m1,&m2,&m3]).expect("unexpected fusion failure");
        let mut fused_theoretic = lattice.assignment_with_capacity(6);
        // A,B,C -> resditributed
        let weight = m_a * m_b * m_c / (m_a + m_b + m_c);
        fused_theoretic.push(prop_a,m_a * weight)?;
        fused_theoretic.push(prop_b,m_b * weight)?;
        fused_theoretic.push(prop_c,m_c * weight)?;
        // A,B,AB -> resditributed
        let weight = m_a * m_b * m_ab / (m_a + m_b + m_ab);
        fused_theoretic.push(prop_a,m_a * weight)?;
        fused_theoretic.push(prop_b,m_b * weight)?;
        fused_theoretic.push(prop_ab,m_ab * weight)?;
        // A,CA,C -> resditributed
        let weight = m_a * m_ca * m_c / (m_a + m_ca + m_c);
        fused_theoretic.push(prop_a,m_a * weight)?;
        fused_theoretic.push(prop_ca,m_ca * weight)?;
        fused_theoretic.push(prop_c,m_c * weight)?;
        // A,CA,AB -> conjunctive
        fused_theoretic.push(prop_a,m_a*m_ab*m_ca)?;
        // BC,B,C -> resditributed
        let weight = m_bc * m_b * m_c / (m_bc + m_b + m_c);
        fused_theoretic.push(prop_bc,m_bc * weight)?;
        fused_theoretic.push(prop_b,m_b * weight)?;
        fused_theoretic.push(prop_c,m_c * weight)?;
        // BC,B,AB -> conjunctive
        fused_theoretic.push(prop_b,m_b*m_ab*m_bc)?;
        // BC,CA,C -> conjunctive
        fused_theoretic.push(prop_c,m_c*m_bc*m_ca)?;
        // BC,CA,AB -> resditributed
        let weight = m_bc * m_ca * m_ab / (m_bc + m_ca + m_ab);
        fused_theoretic.push(prop_bc,m_bc * weight)?;
        fused_theoretic.push(prop_ca,m_ca * weight)?;
        fused_theoretic.push(prop_ab,m_ab * weight)?;
        let fused_theoretic: Assignment<_> = fused_theoretic.into();
        println!("ms: {:?}",[&m1,&m2,&m3]);
        println!("fused: {:?}",fused);
        println!("z -> {z}");
        println!("fused_theoretic: {:?}",fused_theoretic);
        println!();
        Ok(())
    }
}
