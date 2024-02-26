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

use crate::{
    traits::{ Referee, Lattice, },
    structs::{
        Assignment, SafeArray, Conjunctive, DempsterShafer,
        Disjunctive, DuboisPrade2D, Pcr6, PcrSharp
    },
};

use hashed_type_def::HashedTypeDef;
#[cfg(feature = "rkyv")] use rkyv::{Archive, Serialize as RkyvSerialize, Deserialize as RkyvDeserialize, };
#[cfg(feature = "serde")] use serde::{Serialize as SerdeSerialize, Deserialize as SerdeDeserialize};

#[derive(HashedTypeDef, Copy, Clone, Debug)]
#[cfg_attr(feature = "rkyv", derive(Archive,RkyvSerialize,RkyvDeserialize))]
#[cfg_attr(feature = "serde", derive(SerdeSerialize, SerdeDeserialize))]
/// Enumeration of referee functions implemented by default
/// * This is useful for defining the choice of a rule within a single type
pub enum EnumRule {
    /// Conjunctive rule
    Conjunctive,
    /// Dempster-Shafer rule
    DempsterShafer,
    /// Dijunctive rule
    Disjunctive,
    /// Dubois & Prade rule (2 assignments)
    DuboisPrade2D,
    /// PCR6 rule
    Pcr6,
    /// PCR# rule
    PcrSharp(PcrSharp),
}

impl Referee for EnumRule {
    fn is_allowed<L>(&self, lattice: &L, bbas: &[&Assignment<L::Item>]) -> bool where L: Lattice, L::Item: Eq + Ord + Hash, {
        match self  {
            EnumRule::Conjunctive => Conjunctive.is_allowed(lattice, bbas),
            EnumRule::DempsterShafer => DempsterShafer.is_allowed(lattice, bbas),
            EnumRule::Disjunctive => Disjunctive.is_allowed(lattice, bbas),
            EnumRule::DuboisPrade2D => DuboisPrade2D.is_allowed(lattice, bbas),
            EnumRule::Pcr6 => Pcr6.is_allowed(lattice, bbas),
            EnumRule::PcrSharp(rule) => rule.is_allowed(lattice, bbas),
        }
    }

    unsafe fn unsafe_from_conditions<L>(&self, lattice: &L, bbas: &[&Assignment<L::Item>], conditions: SafeArray<L::Item>)
            -> Result<Assignment<L::Item>,String> where L: Lattice, L::Item: Eq + Ord + Hash, {
        match self  {
            EnumRule::Conjunctive => Conjunctive.unsafe_from_conditions(lattice, bbas, conditions),
            EnumRule::DempsterShafer => DempsterShafer.unsafe_from_conditions(lattice, bbas, conditions),
            EnumRule::Disjunctive => Disjunctive.unsafe_from_conditions(lattice, bbas, conditions),
            EnumRule::DuboisPrade2D => DuboisPrade2D.unsafe_from_conditions(lattice, bbas, conditions),
            EnumRule::Pcr6 => Pcr6.unsafe_from_conditions(lattice, bbas, conditions),
            EnumRule::PcrSharp(rule) => rule.unsafe_from_conditions(lattice, bbas, conditions),
        }
    }
}