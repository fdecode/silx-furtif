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


use std::fmt::Display;

use hashed_type_def::HashedTypeDef;
// #[cfg(not(feature = "silx-types"))] use crate::fake_slx::u128slx;
// #[cfg(feature = "silx-types")] use silx_types::u128slx;
use crate::types::u128slx;

#[cfg(feature = "serde")] use serde::{ Serialize as SerdeSerialize, Deserialize as SerdeDeserialize, };
#[cfg(feature = "rkyv")] use rkyv::{ Archive, Serialize as RkyvSerialize, Deserialize as RkyvDeserialize, };

#[derive(HashedTypeDef, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord,)]
#[cfg_attr(feature = "rkyv", derive(Archive,RkyvSerialize,RkyvDeserialize))]
/// Definition of safe element
/// * Safe element combines the element's actual encoding (which generally gives no information about the original lattice) with the hash of its lattice
/// * `X` : type of the encoding
pub struct SafeElement<X> {
    pub (crate) code: X,
    pub (crate) lattice_hash: u128slx,
}

impl<X> Display for SafeElement<X> where X: Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.code.fmt(f)
    }
}

// implementation of Serde serialization
#[cfg(feature = "serde")]
mod serding {
    // #[cfg(not(feature = "silx-types"))] use crate::fake_slx::FakeSlx;
    // #[cfg(feature = "silx-types")] use silx_types::{IntoSlx, SlxInto};
    use crate::types::{ SlxInto, IntoSlx, };
    use super::{ 
        SafeElement as SerdingSafeElement, SerdeSerialize, SerdeDeserialize,
    };
    #[derive(SerdeSerialize,SerdeDeserialize)]
    pub struct SafeElement<X> {
        element: X, lattice_hash: u128,
    }
    impl<'de, X> SerdeDeserialize<'de> for SerdingSafeElement<X> where X: SerdeDeserialize<'de>, {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de> {
            let SafeElement { element, lattice_hash} = SafeElement::<X>::deserialize(deserializer)?;
            let lattice_hash = lattice_hash.slx();
            Ok(Self { code: element, lattice_hash, })
        }
    }
    impl<X> SerdeSerialize for SerdingSafeElement<X>  where X: Clone + SerdeSerialize, {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer {
            let SerdingSafeElement { code: element, lattice_hash } = self;
            let element = element.clone();
            let lattice_hash = lattice_hash.unslx();
            let safe_element = SafeElement { element, lattice_hash, };
            safe_element.serialize(serializer)
        }
    }
}

/// Structure for safe array (for internal use)
/// * A safe array contains a lattice hash and a sequence of encoded elements from this lattice  
/// * `X` : type of the element encoding
#[derive(HashedTypeDef, Debug,)]
pub struct SafeArray<'a,X> {
    pub (crate) product: Vec<&'a X>,
    pub (crate) lattice_hash: u128slx,
}

impl<X> SafeElement<X> {
    /// Unsafe constructor for safe element
    /// * the constructor is unsafe, as there is no consistency check that the encoded element comes from the lattice 
    /// * `element: X` : encoded element
    /// * `lattice_hash: u128slx` : lattice hash
    /// * Output: safe element
    pub unsafe fn unsafe_new(element: X, lattice_hash: u128slx,) -> Self {
        Self { code: element, lattice_hash }
    }

    /// Get encoded element
    /// * Output: encoded element
    pub fn encoded(&self) -> X where X: Clone { self.code.clone() }

    /// Get lattice hash
    /// * Output: lattice hash
    pub fn lattice_hash(&self) -> u128slx { self.lattice_hash }
}

impl<'a,X> SafeArray<'a,X> {
    pub fn len(&self) -> usize { self.product.len() }

    /// Get lattice hash
    /// * Output: lattice hash
    pub fn lattice_hash(&self) -> u128slx { self.lattice_hash }
}