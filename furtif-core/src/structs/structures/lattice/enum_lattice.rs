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


use std::{collections::hash_map, vec};

use crate::{
    types::{ u128slx, f64slx, },
    structs::{ Powerset, Taxonomy, }, 
    traits::{CollectionFamily1, IterableLattice, Lattice, LatticeWithLeaves}
};

use hashed_type_def::HashedTypeDef;
#[cfg(feature = "rkyv")] use rkyv::{Archive, Serialize as RkyvSerialize, Deserialize as RkyvDeserialize, };
#[cfg(feature = "serde")] use serde::{Serialize as SerdeSerialize, Deserialize as SerdeDeserialize};
// #[cfg(feature = "silx-types")] use silx_types::{f64slx, u128slx};
// #[cfg(not(feature = "silx-types"))] use crate::fake_slx::{f64slx, u128slx};

#[deprecated(since="0.1.2", note="please use `EnumLattice`")]
/// Enumeration of lattices implemented by default
pub type CombiLattice = EnumLattice;

#[derive(HashedTypeDef, Clone, Debug)]
#[cfg_attr(feature = "rkyv", derive(Archive,RkyvSerialize,RkyvDeserialize))]
#[cfg_attr(feature = "serde", derive(SerdeSerialize, SerdeDeserialize))]
/// Enumeration of lattices implemented by default
pub enum EnumLattice {
    /// Powerset
    Powerset{ powerset: Powerset, },
    /// Taxonomy
    Taxonomy{ taxonomy: Taxonomy, },
}

impl Lattice for EnumLattice {
    type Item = u128slx;

    fn rand_lattice<R: rand::prelude::Rng>(rng: &mut R) -> Self {
        if rng.gen::<bool>() {
            Self::Powerset { powerset: Powerset::rand_lattice(rng) }
        } else {
            Self::Taxonomy { taxonomy: Taxonomy::rand_lattice(rng) }
        }
    }

    fn rand_element<R: rand::prelude::Rng>(&self, rng: &mut R) -> crate::structs::SafeElement<Self::Item> {
        match self {
            EnumLattice::Powerset { powerset } => powerset.rand_element(rng),
            EnumLattice::Taxonomy { taxonomy } => taxonomy.rand_element(rng),
        }
    }

    fn rand_elements<R: rand::prelude::Rng,I>(&self, len: usize, rng: &mut R) -> I::Type<crate::structs::SafeElement<Self::Item>> where I: CollectionFamily1 {
        match self {
            EnumLattice::Powerset { powerset } => powerset.rand_elements::<R,I>(len, rng),
            EnumLattice::Taxonomy { taxonomy } => taxonomy.rand_elements::<R,I>(len, rng),
        }
    }


    fn ref_lattice_hash(&self) -> &u128slx {
        match self {
            EnumLattice::Powerset { powerset } => powerset.ref_lattice_hash(),
            EnumLattice::Taxonomy { taxonomy } => taxonomy.ref_lattice_hash(),
        }
    }

    fn contains(&self, element: &Self::Item) -> bool {
        match self {
            EnumLattice::Powerset { powerset } => powerset.contains(element),
            EnumLattice::Taxonomy { taxonomy } => taxonomy.contains(element),
        }
    }

    fn ref_bottom(&self) -> &crate::structs::SafeElement<Self::Item> {
        match self {
            EnumLattice::Powerset { powerset } => powerset.ref_bottom(),
            EnumLattice::Taxonomy { taxonomy } => taxonomy.ref_bottom(),
        }
    }

    fn ref_top(&self) -> &crate::structs::SafeElement<Self::Item> {
        match self {
            EnumLattice::Powerset { powerset } => powerset.ref_top(),
            EnumLattice::Taxonomy { taxonomy } => taxonomy.ref_top(),
        }
    }

    unsafe fn unsafe_meet(&self, element_left: &Self::Item, element_right: &Self::Item) -> Self::Item {
        match self {
            EnumLattice::Powerset { powerset } => powerset.unsafe_meet(element_left, element_right),
            EnumLattice::Taxonomy { taxonomy } => taxonomy.unsafe_meet(element_left, element_right),
        }
    }

    unsafe fn unsafe_join(&self, element_left: &Self::Item, element_right: &Self::Item) -> Self::Item {
        match self {
            EnumLattice::Powerset { powerset } => powerset.unsafe_join(element_left, element_right),
            EnumLattice::Taxonomy { taxonomy } => taxonomy.unsafe_join(element_left, element_right),
        }
    }

    fn from_str(&self, s: &str) -> Result<crate::structs::SafeElement<Self::Item>,String> {
        match self {
            EnumLattice::Powerset { powerset } => powerset.from_str(s),
            EnumLattice::Taxonomy { taxonomy } => taxonomy.from_str(s),
        }
    }

    fn to_string(&self, element: &crate::structs::SafeElement<Self::Item>) -> Result<String,String> {
        match self {
            EnumLattice::Powerset { powerset } => powerset.to_string(element),
            EnumLattice::Taxonomy { taxonomy } => taxonomy.to_string(element),
        }
    }
}

impl IterableLattice for EnumLattice {
    type IntoIterUp = vec::IntoIter<u128slx>;

    type IntoIterDown = vec::IntoIter<u128slx>;

    unsafe fn unsafe_bottom_to_top(&self) -> Result<Self::IntoIterUp,String> {
        match self {
            EnumLattice::Powerset { powerset } => powerset.unsafe_bottom_to_top(),
            EnumLattice::Taxonomy { taxonomy } => taxonomy.unsafe_bottom_to_top(),
        }
    }

    unsafe fn unsafe_top_to_bottom(&self) -> Result<Self::IntoIterDown,String> {
        match self {
            EnumLattice::Powerset { powerset } => powerset.unsafe_top_to_bottom(),
            EnumLattice::Taxonomy { taxonomy } => taxonomy.unsafe_top_to_bottom(),
        }
    }
}

impl LatticeWithLeaves for EnumLattice {
    type IntoIterLeaves = hash_map::IntoIter<Self::Item, f64slx>;

    unsafe fn unsafe_weighted_leaf(&self, u: usize) -> Result<(&Self::Item,&f64slx),String> {
        match self {
            EnumLattice::Powerset { powerset } => powerset.unsafe_weighted_leaf(u),
            EnumLattice::Taxonomy { taxonomy } => taxonomy.unsafe_weighted_leaf(u),
        }
    }

    unsafe fn unsafe_leaves(&self) -> Result<Self::IntoIterLeaves,String> {
        match self {
            EnumLattice::Powerset { powerset } => powerset.unsafe_leaves(),
            EnumLattice::Taxonomy { taxonomy } => taxonomy.unsafe_leaves(),
        }
    }
}
