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


use std::{ iter::once, hash::Hash, collections::HashMap, };

use crate::{
    traits::{ Referee, Lattice, },
    structs::{Assignment, SafeArray, one_f64slx, },
};

use hashed_type_def::HashedTypeDef;
#[cfg(feature = "rkyv")] use rkyv::{Archive, Serialize as RkyvSerialize, Deserialize as RkyvDeserialize, };
#[cfg(feature = "serde")] use serde::{Serialize as SerdeSerialize, Deserialize as SerdeDeserialize};

#[derive(HashedTypeDef, Copy, Clone, Debug)]
#[cfg_attr(feature = "rkyv", derive(Archive,RkyvSerialize,RkyvDeserialize))]
#[cfg_attr(feature = "serde", derive(SerdeSerialize, SerdeDeserialize))]
/// Dempster-Shafer referee function
pub struct DempsterShafer;

impl Referee for DempsterShafer {
    fn is_allowed<L>(&self, _lattice: &L, _bbas: &[&Assignment<L::Item>]) -> bool 
                                                where L: Lattice, L::Item: Eq + Ord + Hash, {
        true // always defined
    }

    unsafe fn unsafe_from_conditions<L>(&self, 
        lattice: &L, _bbas: &[&Assignment<L::Item>], conditions: SafeArray<L::Item>
    ) -> Result<Assignment<L::Item>,String> where L: Lattice, L::Item: Eq + Ord + Hash, {
        let SafeArray { product, lattice_hash, } = conditions;
        let top = lattice.top().code;
        let meet = product.iter()
            .map(|e| *e).fold(top, 
                |acc,e| unsafe { lattice.unsafe_meet(&acc,e) }
            );
        if ! unsafe { lattice.unsafe_is_bottom(&meet) } {
            let elements = once((meet,*one_f64slx())).collect();
            Ok(Assignment { elements, lattice_hash, })
        } else {
            let elements = HashMap::new();
            Ok(Assignment { elements, lattice_hash, })
        }
    }
}

pub mod experiment {
    // #[cfg(not(feature = "silx-types"))] use crate::fake_slx::FakeSlx;
    // #[cfg(feature = "silx-types")] use silx_types::IntoSlx;

    use crate::{
        types::IntoSlx,
        structs::{Powerset, DiscountedFuser, DempsterShafer, one_f64slx, Assignment, }, 
        traits::{Lattice, DiscountedFusion, LatticeWithLeaves, }
    };

    /// Experimentation with the Dempster-Shafer rule
    pub fn exp_dempster_shafer() -> Result<(),String> {
        println!("================= Dempster Shafer======");
        let lattice = Powerset::new(4,1024)
                .expect("unexpected powwerset initialisation failure");
        let length_mid = 512u32;
        let length_max = 1024u32;
        let (mut m1, mut m2, mut m3) = (
            lattice.prunable_with_capacity(length_mid, length_max, 2),
            lattice.prunable_with_capacity(length_mid, length_max, 2),
            lattice.prunable_with_capacity(length_mid, length_max, 2),
        );
        let (prop_a, m_a) = (lattice.leaf(0)?, 0.3);
        let (prop_b, m_b) = (lattice.leaf(1)?, 0.4);
        let (prop_c, m_c) = (lattice.leaf(2)?, 0.5);
        let (prop_ab, m_ab) = (lattice.join(&prop_a,&prop_b)?,0.5);
        let (prop_bc, m_bc) = (lattice.join(&prop_b,&prop_c)?,0.7);
        let (prop_ca, m_ca) = (lattice.join(&prop_c,&prop_a)?,0.6);
        let (m_a, m_b, m_c, m_ab, m_bc, m_ca) = 
            (m_a.slx(), m_b.slx(), m_c.slx(), m_ab.slx(), m_bc.slx(), m_ca.slx());
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
        let referee = DempsterShafer;
        let (fused,z) = engine.fuse(&lattice, &referee,&[&m1,&m2,&m3]).expect("unexpected fusion failure");
        let mut fused_theoretic = lattice.prunable_with_capacity(length_mid, length_max, 4);
        let co_z_theoric = m_a*m_ab*m_ca + m_b*m_ab*m_bc + m_c*m_bc*m_ca;
        let z_theoric = *one_f64slx() - co_z_theoric;
        fused_theoretic.push(prop_a,(m_a*m_ab*m_ca) / co_z_theoric)?;
        fused_theoretic.push(prop_b,(m_b*m_ab*m_bc) / co_z_theoric)?;
        fused_theoretic.push(prop_c,(m_c*m_bc*m_ca) / co_z_theoric)?;
        let fused_theoretic: Assignment<_> = fused_theoretic.into();
        println!("ms: {:?}",[&m1,&m2,&m3]);
        println!("fused: {:?}",fused);
        println!("z -> {z}");
        println!("fused_theoretic: {:?}",fused_theoretic);
        println!("z_theoric -> {z_theoric}");
        println!();
        Ok(())
    }
}
