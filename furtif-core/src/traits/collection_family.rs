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


use std::collections::{ VecDeque, LinkedList, };

/// Instrumental trait used for defining some collections transforms
pub trait CollectionFamily1 {
    type Type<T>: FromIterator<T> + IntoIterator<Item = T>;
}

impl<U> CollectionFamily1 for Vec<U> {
    type Type<T> = Vec<T>;
}

impl<U> CollectionFamily1 for VecDeque<U> {
    type Type<T> = VecDeque<T>;
}

impl<U> CollectionFamily1 for LinkedList<U> {
    type Type<T> = LinkedList<T>;
}
