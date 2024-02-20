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


use std::{ iter::once, hash::Hash, };

use crate::{
    traits::{ Referee, Lattice, },
    structs::{Assignment, SafeArray, one_f64slx, },
};

use hashed_type_def::HashedTypeDef;
use rkyv::{Archive, Serialize as RkyvSerialize, Deserialize as RkyvDeserialize, };
use serde::{Serialize as SerdeSerialize, Deserialize as SerdeDeserialize};

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, HashedTypeDef, SerdeSerialize, SerdeDeserialize, 
    Copy, Clone, Debug
)]
/// Disjunctive referee function
pub struct Disjunctive;

impl Referee for Disjunctive {
    fn is_allowed<L>(&self, _lattice: &L, _bbas: &[&Assignment<L::Item>]) -> bool 
                                                where L: Lattice, L::Item: Eq + Ord + Hash, {
        true // always defined
    }

    unsafe fn unsafe_from_conditions<L>(&self, 
        lattice: &L, _bbas: &[&Assignment<L::Item>], conditions: SafeArray<L::Item>
    ) -> Result<Assignment<L::Item>,String> where L: Lattice, L::Item: Eq + Ord + Hash, {
        let SafeArray { product, lattice_hash, } = conditions;
        let bottom = lattice.bottom().code;
        let join = product.iter()
            .map(|e| *e).fold(bottom, 
                |acc,e| unsafe { lattice.unsafe_join(&acc,e) }
            );
//        let length_mid = u32::MAX.slx(); let length_max = u32::MAX.slx();
//        let x = join;
        let elements = once((join,*one_f64slx())).collect();
//        let ord_elements = once(OrdData((x,*one_f64slx()))).collect();
//        let elements = OrdMap { elements, ord_elements };
        Ok(Assignment { elements, lattice_hash, })
    }
}

pub mod experiment {
    use silx_types::IntoSlx;

    use crate::{
        structs::{Powerset, DiscountedFuser, Disjunctive, Assignment, }, 
        traits::{Lattice, DiscountedFusion, LatticeWithLeaves, }
    };

    /// Experimentation with the disjunctive rule
    pub fn exp_disjunctive() -> Result<(),String> {
        println!("================= Disjunctive =========");
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
        let prop_abc = lattice.join(&prop_a,&prop_bc)?;
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
        let referee = Disjunctive;
        let (fused,z) = engine.fuse(&lattice, &referee,&[&m1,&m2,&m3]).expect("unexpected fusion failure");
        let mut fused_theoretic = lattice.assignment();
        // A,B,C -> ABC
        fused_theoretic.push(prop_abc,m_a * m_b * m_c)?;
        // A,B,AB -> AB
        fused_theoretic.push(prop_ab,m_a * m_b * m_ab)?;
        // A,CA,C -> CA
        fused_theoretic.push(prop_ca,m_a * m_ca * m_c)?;
        // A,CA,AB -> ABC
        fused_theoretic.push(prop_abc,m_a*m_ab*m_ca)?;
        // BC,B,C -> BC
        fused_theoretic.push(prop_bc,m_bc * m_b * m_c)?;
        // BC,B,AB -> ABC
        fused_theoretic.push(prop_abc,m_b*m_ab*m_bc)?;
        // BC,CA,C -> ABC
        fused_theoretic.push(prop_abc,m_c*m_bc*m_ca)?;
        // BC,CA,AB -> ABC
        fused_theoretic.push(prop_abc,m_bc * m_ca * m_ab)?;
        let fused_theoretic: Assignment<_> = fused_theoretic.into();
        println!("ms: {:?}",[&m1,&m2,&m3]);
        println!("fused: {:?}",fused);
        println!("z -> {z}");
        println!("fused_theoretic: {:?}",fused_theoretic);
        println!();
        Ok(())
    }
}
