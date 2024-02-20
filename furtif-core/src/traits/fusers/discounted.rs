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


use std::{ hash::Hash, iter::once, ops::RangeInclusive, };

use silx_types::{ f64slx, IntoSlx, Float, };
use crate::{
    structs::{ Assignment, SafeArray, one_f64slx, zero_f64slx, },
    traits::{ Lattice, Referee, CollectionFamily1, },
};

/// For intern use: produce tensor product combination of the bbas
/// * `lattice: &'a L` : lattice of definition of the assignments
/// * `bbas: &'a[&'a Assignment<L::Item>]` : collection of assignments
/// * `L` : type of lattice
/// * Output: tensor product combination of the bbas or an error
fn product_bba<'a,L>(lattice: &'a L, bbas: &'a[&'a Assignment<L::Item>]) -> Result<Vec<(SafeArray<'a,L::Item>,f64slx)>, String> 
                                                                                                where L: Lattice, L::Item: Eq + Ord + Hash, {
    let lattice_hash = lattice.lattice_hash();
    // Compatibility tests:
    for (u,bba) in bbas.into_iter().enumerate() {
        if bba.lattice_hash != lattice_hash { return Err(format!("bbas of index {u} is not defined over lattice")); } 
    }
    let mut products = vec![(Vec::<&L::Item>::with_capacity(bbas.len()),1.0f64.slx())];
    for bba in bbas {
        products = products.iter().flat_map(|(left,left_w)| {
            bba.elements.iter().map(move |(right,right_w)|
                (left.iter().map(|l|*l).chain(once(right)).collect(), *left_w * *right_w)
            )
        }).collect::<Vec<_>>();
    }
    Ok(products.into_iter().map(|(product,weight)| 
        (SafeArray{ lattice_hash, product,}, weight)
    ).collect())
}


/// Trait defining generic discounted fusion processes
/// * Smallest assignments are reduced until assignment cardinal is below given range
pub trait DiscountedFusion {
    /// Range defining an hysteresis for assignment reduction
    /// * Principle:
    ///   1)  Reduction is started when above range max
    ///   2)  Reduction is done until below or equal to range min
    /// * Reduction strategy is defined by means of `AssignmentBuilder` mechanisms
    fn size_range(&self) -> RangeInclusive<usize>;

    /// Fusing bbas returning fused assignment and conflict
    /// * `lattice: &L` : lattice of definition of the assignments
    /// * `referee: &F` : referee function
    /// * `bbas: &[&Assignment<L::Item>]` : assignments sequence
    /// * `L` : type of the lattice
    /// * `F` : type of the referee function
    /// * Output: an error or a pair composed of:
    ///   * the fused assignment
    ///   * the conflict
    fn fuse<L,F>(&self, lattice: &L, referee: &F, bbas: &[&Assignment<L::Item>])
            -> Result<(Assignment<L::Item>,f64slx),String> where L: Lattice, L::Item: Eq + Ord + Hash, F: Referee {
        let (length_mid, length_max) = {
            let range = self.size_range();
            (*range.start() as u32,*range.end() as u32)
        };
        let mut bba = lattice.prunable(length_mid, length_max);
        let products = product_bba(lattice,bbas)?;
        for (conditions,weight)  in products {
            let output = referee.from_conditions(lattice, bbas, conditions)?;
            // optimizable ==>
            for (safe_element, sub_weight) in output {
                bba.push(safe_element, sub_weight * weight)?;
            } // <== 
        }
        bba.prune(|x,y| unsafe{ lattice.unsafe_meet(&x, &y) });
        let norm = bba.cumul_weight()?;
        let z = *one_f64slx() - norm;
        if &norm == zero_f64slx() {
            Err("Cumulative weight is zero, cannot be normalized".to_string())
        } else { 
            bba.scale(norm.recip())?;
            Ok((bba.into(),z))
        }
    }

    /// fusing bbas sequentially returning collected fused assignments and conflicts
    /// * `lattice: &L` : lattice of definition of the assignments
    /// * `referees: &[&F]` : collection of referee functions
    /// * `slice_bbas: &[&[&Assignment<L::Item>]]` : collection of assignments sequence 
    /// * `L` : type of the lattice
    /// * `F` : type of the referee function
    /// * `I` : type of the returned collection
    /// * Output: an error or a collection of pairs composed of:
    ///   * a fused assignment
    ///   * a conflict
    fn fuse_seq<L,F,I>(&self, lattice: &L, referees: &[&F], slice_bbas: &[&[&Assignment<L::Item>]]) 
            -> Result<I::Type<(Assignment<L::Item>,f64slx)>,String> 
                where L: Lattice, L::Item: Eq + Ord + Hash, F: Referee, I: CollectionFamily1 {
        let (referees_len, seq_bbas_len) = (referees.len(), slice_bbas.len());
        if referees_len != seq_bbas_len {
            return Err(format!("mismatching lengths {} vs {}", referees_len, seq_bbas_len));
        }
        let mut results = Vec::with_capacity(referees_len);
        for (referee,bbas) in referees.into_iter().zip(slice_bbas) {
            results.push(self.fuse(lattice,*referee, *bbas)?);
        }
        Ok(results.into_iter().collect())
    }
}