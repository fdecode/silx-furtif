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


/// Assignments tools
mod assignment_tools; pub use self::assignment_tools::{ 
    SafeElement, SafeArray, AssignmentBuilder, Assignment, exp_hidden, ASSIGNMENT_EPSILON, 
}; 
pub (crate) use self::assignment_tools::{ hidden, zero_f64slx, one_f64slx, };
/// Definition of fusion rules and fusion engines
mod fusers; pub use self::fusers::{ 
    Conjunctive, Disjunctive, DiscountedFuser, DuboisPrade2D, Pcr6, PcrSharp, DempsterShafer, EnumRule,
    exp_pcr6, exp_conjunctive, exp_dempster_shafer, exp_disjunctive, exp_pcr_sharp, exp_dubois_prade_2d,
};
/// Definitions of lattices structures
mod structures; pub use self::structures::{ 
    Powerset, Taxon, TaxonCoder, Taxonomy, Taxons, TaxonomyBuilder, CombiLattice,
    exp_taxonomy_1, exp_taxonomy_2,
}; 
/// Definitions of metrics
mod metrics; // not implemented at this time