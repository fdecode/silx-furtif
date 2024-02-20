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


/// Definition of safe element and related tools
/// * Safe element combines the element's actual encoding (which generally gives no information about the original lattice) with the hash of its lattice
mod safe; pub use self::safe::{ SafeElement, SafeArray, };
/// Definition of assignment and related tools
mod assignment; pub use self::assignment::{ Assignment, AssignmentBuilder, exp_hidden, ASSIGNMENT_EPSILON, }; 
pub (crate) use self::assignment::{ hidden, one_f64slx, zero_f64slx, };
