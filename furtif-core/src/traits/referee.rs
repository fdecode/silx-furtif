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

use crate::{ structs::{Assignment, SafeArray}, traits::Lattice, };

/// Trait defining Referee functions
pub trait Referee {
    /// Test if fusion is allowed
    /// * does not concern lattice coherence, which is tested by `from_conditions`
    /// * typically concerns the number of entries or the algebraic properties of lattice
    /// * `lattice: &L` : reference lattice
    /// * `bbas: &[&Assignment<L::Item>]` : sequence of bbas to be fused
    /// * Output: a boolean
    fn is_allowed<L>(&self, lattice: &L, bbas: &[&Assignment<L::Item>]) -> bool where L: Lattice, L::Item: Eq + Ord + Hash,;

    /// unsafe conditional referee decision
    /// * `lattice: &L` : reference lattice
    /// * `bbas: &[&Assignment<L::Item>]` : sequence of bbas to be fused 
    /// * `conditions: SafeArray<L::Item>` : conditionning safe elements array
    /// * Output: fused assigment or error
    unsafe fn unsafe_from_conditions<L>(&self, lattice: &L, bbas: &[&Assignment<L::Item>], conditions: SafeArray<L::Item>)
            -> Result<Assignment<L::Item>,String> where L: Lattice, L::Item: Eq + Ord + Hash,;

    /// Conditional referee decision
    /// * `lattice: &L` : reference lattice
    /// * `bbas: &[&Assignment<L::Item>]` : sequence of bbas to be fused 
    /// * `conditions: SafeArray<L::Item>` : conditionning safe elements array 
    /// * Output: fused assigment or error 
    fn from_conditions<L>(&self, lattice: &L, bbas: &[&Assignment<L::Item>], conditions: SafeArray<L::Item>)
            -> Result<Assignment<L::Item>,String> where L: Lattice, L::Item: Eq + Ord + Hash, {
        if bbas.len() != conditions.len() { return Err("Mismatching entries lengths".to_string()); }
        let lattice_hash = lattice.lattice_hash();
        if lattice_hash != conditions.lattice_hash { return Err("Conditions not within lattice".to_string()); }
        for (u,bba) in bbas.iter().enumerate() {
            if lattice_hash != bba.lattice_hash { return Err(format!("Bba with index {u} is not defined over lattice")); }
        }
        if self.is_allowed(lattice, bbas) {
            unsafe { self.unsafe_from_conditions(lattice, bbas, conditions) }
        } else { Err("Entries not allowed".to_string()) } 
    }
}