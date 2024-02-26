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

#[derive(HashedTypeDef, Clone, Debug)]
#[cfg_attr(feature = "rkyv", derive(Archive,RkyvSerialize,RkyvDeserialize))]
#[cfg_attr(feature = "serde", derive(SerdeSerialize, SerdeDeserialize))]
/// Enumeration of lattices implemented by default
pub enum CombiLattice {
    /// Powerset
    Powerset{ powerset: Powerset, },
    /// Taxonomy
    Taxonomy{ taxonomy: Taxonomy, },
}

impl Lattice for CombiLattice {
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
            CombiLattice::Powerset { powerset } => powerset.rand_element(rng),
            CombiLattice::Taxonomy { taxonomy } => taxonomy.rand_element(rng),
        }
    }

    fn rand_elements<R: rand::prelude::Rng,I>(&self, len: usize, rng: &mut R) -> I::Type<crate::structs::SafeElement<Self::Item>> where I: CollectionFamily1 {
        match self {
            CombiLattice::Powerset { powerset } => powerset.rand_elements::<R,I>(len, rng),
            CombiLattice::Taxonomy { taxonomy } => taxonomy.rand_elements::<R,I>(len, rng),
        }
    }


    fn ref_lattice_hash(&self) -> &u128slx {
        match self {
            CombiLattice::Powerset { powerset } => powerset.ref_lattice_hash(),
            CombiLattice::Taxonomy { taxonomy } => taxonomy.ref_lattice_hash(),
        }
    }

    fn contains(&self, element: &Self::Item) -> bool {
        match self {
            CombiLattice::Powerset { powerset } => powerset.contains(element),
            CombiLattice::Taxonomy { taxonomy } => taxonomy.contains(element),
        }
    }

    fn ref_bottom(&self) -> &crate::structs::SafeElement<Self::Item> {
        match self {
            CombiLattice::Powerset { powerset } => powerset.ref_bottom(),
            CombiLattice::Taxonomy { taxonomy } => taxonomy.ref_bottom(),
        }
    }

    fn ref_top(&self) -> &crate::structs::SafeElement<Self::Item> {
        match self {
            CombiLattice::Powerset { powerset } => powerset.ref_top(),
            CombiLattice::Taxonomy { taxonomy } => taxonomy.ref_top(),
        }
    }

    unsafe fn unsafe_meet(&self, element_left: &Self::Item, element_right: &Self::Item) -> Self::Item {
        match self {
            CombiLattice::Powerset { powerset } => powerset.unsafe_meet(element_left, element_right),
            CombiLattice::Taxonomy { taxonomy } => taxonomy.unsafe_meet(element_left, element_right),
        }
    }

    unsafe fn unsafe_join(&self, element_left: &Self::Item, element_right: &Self::Item) -> Self::Item {
        match self {
            CombiLattice::Powerset { powerset } => powerset.unsafe_join(element_left, element_right),
            CombiLattice::Taxonomy { taxonomy } => taxonomy.unsafe_join(element_left, element_right),
        }
    }

    fn from_str(&self, s: &str) -> Result<crate::structs::SafeElement<Self::Item>,String> {
        match self {
            CombiLattice::Powerset { powerset } => powerset.from_str(s),
            CombiLattice::Taxonomy { taxonomy } => taxonomy.from_str(s),
        }
    }

    fn to_string(&self, element: &crate::structs::SafeElement<Self::Item>) -> Result<String,String> {
        match self {
            CombiLattice::Powerset { powerset } => powerset.to_string(element),
            CombiLattice::Taxonomy { taxonomy } => taxonomy.to_string(element),
        }
    }
}

impl IterableLattice for CombiLattice {
    type IntoIterUp = vec::IntoIter<u128slx>;

    type IntoIterDown = vec::IntoIter<u128slx>;

    unsafe fn unsafe_bottom_to_top(&self) -> Result<Self::IntoIterUp,String> {
        match self {
            CombiLattice::Powerset { powerset } => powerset.unsafe_bottom_to_top(),
            CombiLattice::Taxonomy { taxonomy } => taxonomy.unsafe_bottom_to_top(),
        }
    }

    unsafe fn unsafe_top_to_bottom(&self) -> Result<Self::IntoIterDown,String> {
        match self {
            CombiLattice::Powerset { powerset } => powerset.unsafe_top_to_bottom(),
            CombiLattice::Taxonomy { taxonomy } => taxonomy.unsafe_top_to_bottom(),
        }
    }
}

impl LatticeWithLeaves for CombiLattice {
    type IntoIterLeaves = hash_map::IntoIter<Self::Item, f64slx>;

    unsafe fn unsafe_weighted_leaf(&self, u: usize) -> Result<(&Self::Item,&f64slx),String> {
        match self {
            CombiLattice::Powerset { powerset } => powerset.unsafe_weighted_leaf(u),
            CombiLattice::Taxonomy { taxonomy } => taxonomy.unsafe_weighted_leaf(u),
        }
    }

    unsafe fn unsafe_leaves(&self) -> Result<Self::IntoIterLeaves,String> {
        match self {
            CombiLattice::Powerset { powerset } => powerset.unsafe_leaves(),
            CombiLattice::Taxonomy { taxonomy } => taxonomy.unsafe_leaves(),
        }
    }
}
