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


use std::ops::RangeInclusive;

use hashed_type_def::HashedTypeDef;
#[cfg(feature = "rkyv")] use rkyv::{Archive, Serialize as RkyvSerialize, Deserialize as RkyvDeserialize, };
use crate::types::{ u32slx, SlxInto, IntoSlx, };
// #[cfg(not(feature = "silx-types"))] use crate::fake_slx::{ u32slx, FakeSlx };
// #[cfg(feature = "silx-types")] use silx_types::{u32slx, IntoSlx, SlxInto};

use crate::traits::DiscountedFusion;
#[cfg(feature = "serde")] use serde::{Serialize as SerdeSerialize, Deserialize as SerdeDeserialize};

#[derive(HashedTypeDef, Copy, Clone, Debug)]
#[cfg_attr(feature = "rkyv", derive(Archive,RkyvSerialize,RkyvDeserialize))]
/// Generic fusion engine based on exact computation, but with mass discounting when above a given range
/// * Mass discounting is performed by iteratively putting the mass of the weakest assigments on their disjunction 
pub struct DiscountedFuser {
    range_min: u32slx, range_max: u32slx,
}

// implementation of Serde serialization
#[cfg(feature = "serde")] mod serding {
    use crate::types::{ SlxInto, IntoSlx, };
    // #[cfg(not(feature = "silx-types"))] use crate::fake_slx::FakeSlx;
    // #[cfg(feature = "silx-types")] use silx_types::{IntoSlx, SlxInto};
    use super::{ 
        DiscountedFuser as SerdingDiscountedFuser, SerdeSerialize, SerdeDeserialize,
    };
    #[derive(SerdeSerialize,SerdeDeserialize)]
    pub struct DiscountedFuser {
        range_min: u32, range_max: u32,
    }

    impl<'de> SerdeDeserialize<'de> for SerdingDiscountedFuser {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de> {
            let DiscountedFuser { range_min, range_max } = DiscountedFuser::deserialize(deserializer)?;
            let range_min = range_min.slx();
            let range_max = range_max.slx();
            Ok(Self { range_min, range_max })
        }
    }
    impl SerdeSerialize for SerdingDiscountedFuser {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer {
            let Self { range_min, range_max  } = *self;
            let range_min = range_min.unslx();
            let range_max = range_max.unslx();
            let discounted_fuser = DiscountedFuser { range_min, range_max };
            discounted_fuser.serialize(serializer)
        }
    }
}

impl DiscountedFuser {
    /// Constructor of the fusion engine
    /// * `range: RangeInclusive<usize>` : range within which the fused assignment size will be reduced after discounting
    /// * Output: fusion engine
    pub fn new(range: RangeInclusive<usize>) -> Self {
        let range_min = (*range.start()) as u32;
        let range_max = (*range.end()) as u32;
        let range_min = range_min.slx();
        let range_max = range_max.slx();
        Self { range_min, range_max }
    }
}

impl DiscountedFusion for DiscountedFuser {
    fn size_range(&self) -> RangeInclusive<usize> {
        let Self { range_min, range_max } = *self;
        let range_min = range_min.unslx() as usize;
        let range_max = range_max.unslx() as usize;
        range_min..=range_max
    }
}