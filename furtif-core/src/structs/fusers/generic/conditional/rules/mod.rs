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


/// Definition of Dempster-Shafer rule
mod dempster_shafer; pub use self::dempster_shafer::{ DempsterShafer, experiment::exp_dempster_shafer, };
/// Definition of disjunctive rule
mod disjunctive; pub use self::disjunctive::{ Disjunctive, experiment::exp_disjunctive, };
/// Definition of conjunctive rule
mod conjunctive; pub use self::conjunctive::{ Conjunctive, experiment::exp_conjunctive, };
/// Definition of Dubois & Prade rule
mod dubois_prade; pub use self::dubois_prade::{ DuboisPrade2D, experiment::exp_dubois_prade_2d, };
/// Definition of PCR6 rule
mod pcr6; pub use self::pcr6::{ Pcr6, experiment::exp_pcr6, };
/// Definition of PCR# rule
mod pcr_sharp; pub use self::pcr_sharp::{ PcrSharp, experiment::exp_pcr_sharp, };
/// Enumeration of different rules
mod enum_rule; pub use self::enum_rule::EnumRule;