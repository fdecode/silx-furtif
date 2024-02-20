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


/// Definition of fusion engines
mod engine; pub use self::engine::DiscountedFuser;
/// Definition of rules
mod rules; pub use self::rules::{ 
    Pcr6, PcrSharp, DuboisPrade2D, Disjunctive, Conjunctive, DempsterShafer, EnumRule,
    exp_pcr6, exp_conjunctive, exp_dempster_shafer, exp_disjunctive, exp_pcr_sharp, exp_dubois_prade_2d,
};