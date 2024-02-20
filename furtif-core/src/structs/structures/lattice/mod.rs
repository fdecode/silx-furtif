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


/// Powerset definitions
mod powerset; pub use self::powerset::Powerset;
/// Taxonomy definitions
mod taxonomy; pub use self::taxonomy::{
    Taxon, TaxonCoder, Taxons, Taxonomy, TaxonomyBuilder, 
    experiment::{ exp_taxonomy_1, exp_taxonomy_2, },
};
/// Enumeration of different lattice implementations
mod enum_lattice; pub use self::enum_lattice::CombiLattice;
