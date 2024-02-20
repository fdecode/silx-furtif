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


/// Definitions for furtif examples
pub mod furtif; 

use std::path::PathBuf;
use silx_core::utils::{RecFiled, FiledStarter, Filable};

pub use self::furtif::{ exp_dsmtbook, exp_dsmtbook_mono, };

/// Example of method for loading a silx network and running it
/// * `starter_path: &str` : path of the starter file
/// * `save_path: &str` : path of saving directory
/// * Output : return nothing of a String error message
pub async fn exp_load_start(starter_path: &str, save_path: &str) -> Result<(), String> {
    // load serialized starter command
    let mut starter =  RecFiled::<FiledStarter>::new_unloaded(
        PathBuf::from(starter_path)
    );
    // load entire starter data
    starter.load(&save_path)?;
    // get the starter
    let starter_in = starter.unwrap()?;
    match starter_in.run().await { // run the starter 
        Ok(()) => Ok(()), 
        Err(e) => Err(format!("Failed (loaded) to run starter: {}",e)), 
    }
}