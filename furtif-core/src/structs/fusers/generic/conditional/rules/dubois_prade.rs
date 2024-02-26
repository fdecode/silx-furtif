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
    traits::{ Referee, Lattice,},
    structs::{SafeArray, Assignment, one_f64slx, },
};

use hashed_type_def::HashedTypeDef;
#[cfg(feature = "rkyv")] use rkyv::{Archive, Serialize as RkyvSerialize, Deserialize as RkyvDeserialize, };
#[cfg(feature = "serde")] use serde::{Serialize as SerdeSerialize, Deserialize as SerdeDeserialize};

#[derive(HashedTypeDef, Copy, Clone, Debug)]
#[cfg_attr(feature = "rkyv", derive(Archive,RkyvSerialize,RkyvDeserialize))]
#[cfg_attr(feature = "serde", derive(SerdeSerialize, SerdeDeserialize))]
/// Dubois & Prade referee function for two assignments
pub struct DuboisPrade2D;

impl Referee for DuboisPrade2D {
    fn is_allowed<L>(&self, _lattice: &L, _bbas: &[&Assignment<L::Item>]) -> bool 
                                                where L: Lattice, L::Item: Eq + Ord + Hash, {
        _bbas.len() == 2 // only defined for 2 bbas
    }

    unsafe fn unsafe_from_conditions<L>(&self, 
        lattice: &L, _bbas: &[&Assignment<L::Item>], conditions: SafeArray<L::Item>
    ) -> Result<Assignment<L::Item>,String> where L: Lattice, L::Item: Eq + Ord + Hash, {
        let SafeArray { product, lattice_hash, } = conditions;
        let meet = unsafe { lattice.unsafe_meet(&product[0],&product[1]) };
//        let length_mid = u32::MAX.slx(); let length_max = u32::MAX.slx();
        if ! unsafe { lattice.unsafe_is_bottom(&meet) } {
//            let x = meet;
            let elements = once((meet,*one_f64slx())).collect();
//            let ord_elements = once(OrdData((x,*one_f64slx()))).collect();
//            let elements = OrdMap { elements, ord_elements };
            Ok(Assignment { elements, lattice_hash, })
        } else {
            let join = unsafe { lattice.unsafe_join(&product[0],&product[1]) };
//            let x = join;
            let elements = once((join,*one_f64slx())).collect();
//            let ord_elements = once(OrdData((x,*one_f64slx()))).collect();
//            let elements = OrdMap { elements, ord_elements };
            Ok(Assignment { elements, lattice_hash, })
        }
    }
}

pub mod experiment {
    // #[cfg(not(feature = "silx-types"))] use crate::fake_slx::FakeSlx;
    // #[cfg(feature = "silx-types")] use silx_types::IntoSlx;

    use crate::{
        types::IntoSlx,
        structs::{Powerset, DiscountedFuser, DuboisPrade2D, Assignment, }, 
        traits::{Lattice, DiscountedFusion, LatticeWithLeaves, }
    };

    /// Experimentation with the Dubois & Prade rule (2 assignments)
    pub fn exp_dubois_prade_2d() -> Result<(),String> {
        println!("================= Dubois Prade 2d =====");
        let lattice = Powerset::new(3,1024)
                .expect("unexpected powwerset initialisation failure");
        let length_mid = 512;
        let length_max = 1024;
        let (mut m1, mut m2) = (
            lattice.prunable(length_mid, length_max),
            lattice.prunable(length_mid, length_max),
        );
        let (prop_a, m1_a,) = (lattice.leaf(0)?, 0.3);
        let (prop_b, m1_b, m2_b) = (lattice.leaf(1)?, 0.2, 0.1,);
        let (prop_c, m2_c) = (lattice.leaf(2)?, 0.5);
        let (prop_bc, m1_bc) = (lattice.join(&prop_b,&prop_c)?,0.1);
        let (prop_ca, m1_ca) = (lattice.join(&prop_c,&prop_a)?,0.4);
        let (prop_ab, m2_ab) = (lattice.join(&prop_a,&prop_b)?,0.3);
        let (prop_abc, m2_abc) = (lattice.join(&prop_bc,&prop_a)?,0.1);
        let (m1_a, m1_b, m2_b, m2_c, m1_bc, m1_ca, m2_ab, m2_abc) = 
            (m1_a.slx(), m1_b.slx(), m2_b.slx(), m2_c.slx(), m1_bc.slx(), m1_ca.slx(), m2_ab.slx(), m2_abc.slx());
        m1.push(prop_a,m1_a)?;
        m1.push(prop_b,m1_b)?;
        m1.push(prop_bc,m1_bc)?;
        m1.push(prop_ca,m1_ca)?;
        m2.push(prop_b,m2_b)?;
        m2.push(prop_c,m2_c)?;
        m2.push(prop_ab,m2_ab)?;
        m2.push(prop_abc,m2_abc)?;
        let m1 = m1.into(); 
        let m2 = m2.into(); 
        let engine = DiscountedFuser::new(512..=1024);
        let referee = DuboisPrade2D;
        let (fused,z) = engine.fuse(&lattice, &referee,&[&m1,&m2]).expect("unexpected fusion failure");
        let mut fused_theoretic = lattice.prunable(length_mid, length_max);
        // A,B -> AB
        fused_theoretic.push(prop_ab,m1_a * m2_b)?;
        // A,C -> AC
        fused_theoretic.push(prop_ca,m1_a * m2_c)?;
        // A,AB -> A
        fused_theoretic.push(prop_a,m1_a * m2_ab)?;
        // A,ABC -> A
        fused_theoretic.push(prop_a,m1_a * m2_abc)?;
        //
        // B,B -> B
        fused_theoretic.push(prop_b,m1_b * m2_b)?;
        // B,C -> BC
        fused_theoretic.push(prop_bc,m1_b * m2_c)?;
        // B,AB -> B
        fused_theoretic.push(prop_b,m1_b * m2_ab)?;
        // B,ABC -> B
        fused_theoretic.push(prop_b,m1_b * m2_abc)?;
        //
        // BC,B -> B
        fused_theoretic.push(prop_b,m1_bc * m2_b)?;
        // BC,C -> C
        fused_theoretic.push(prop_c,m1_bc * m2_c)?;
        // BC,AB -> B
        fused_theoretic.push(prop_b,m1_bc * m2_ab)?;
        // BC,ABC -> BC
        fused_theoretic.push(prop_bc,m1_bc * m2_abc)?;
        //
        // CA,B -> ABC
        fused_theoretic.push(prop_abc,m1_ca * m2_b)?;
        // CA,C -> C
        fused_theoretic.push(prop_c,m1_ca * m2_c)?;
        // CA,AB -> A
        fused_theoretic.push(prop_a,m1_ca * m2_ab)?;
        // CA,ABC -> CA
        fused_theoretic.push(prop_ca,m1_ca * m2_abc)?;
        let fused_theoretic: Assignment<_> = fused_theoretic.into();
        println!("ms: {:?}",[&m1,&m2]);
        println!("fused: {:?}",fused);
        println!("z -> {z}");
        println!("fused_theoretic: {:?}",fused_theoretic);
        println!();
        Ok(())
    }
}
