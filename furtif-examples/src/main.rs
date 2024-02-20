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
pub mod crate_main; 

use std::env;

use self::crate_main::{ exp_dsmtbook, exp_dsmtbook_mono, exp_load_start, };

fn _main_exp_referee() {
    println!("{:?}",furtif_core::structs::exp_pcr6());
    println!("{:?}",furtif_core::structs::exp_conjunctive());
    println!("{:?}",furtif_core::structs::exp_dempster_shafer());
    println!("{:?}",furtif_core::structs::exp_disjunctive());
    println!("{:?}",furtif_core::structs::exp_pcr_sharp());
    println!("{:?}",furtif_core::structs::exp_dubois_prade_2d());
}

fn _main_exp_transform() {
    println!("{:?}",furtif_core::traits::exp_transform());
}

fn _main_exp_taxonomy() {
    println!("{:?}",furtif_core::structs::exp_taxonomy_1());
    println!("{:?}",furtif_core::structs::exp_taxonomy_2());
}

#[tokio::main]
/// Main method for the tests of the DSmT book 5 chapter. Options are available as command arguments
/// * exp_load_start STARTER_PATH COMPONENTS_DIR : execution of asynchroneous network loaded from file STARTER_PATH within directory COMPONENTS_DIR
///   - example: command `cargo run --bin furtif-examples --release -- exp_load_start dsmtbook-mono\starter\main.yaml .\main_examples_data\saved_data\` 
///     will load and run cluster defined in file `.\main_examples_data\saved_data\dsmtbook-mono\starter\main.yaml` 
/// * exp_dsmtbook : (default) execution of asynchroneous network example of DSmT book
/// * exp_dsmtbook_mono : execution of one-cluster asynchroneous network example of DSmT book
/// * exp_referee : some referee function examples
/// * exp_transform : some transform examples
/// * exp_taxonomy : some taxonomy examples
pub async fn main() {
    println!("Available paralelism -> {:?}",std::thread::available_parallelism());
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => exp_dsmtbook().await.unwrap(), 
        2 => {
            match args[1].as_str() {
                "exp_dsmtbook" => exp_dsmtbook().await.unwrap(),
                "exp_dsmtbook_mono" => exp_dsmtbook_mono().await.unwrap(),
                "exp_referee" => _main_exp_referee(),
                "exp_transform" => _main_exp_transform(),
                "exp_taxonomy" => _main_exp_taxonomy(),
                _ => panic!("bad argument"),
            }    
        }, 
        4 => {
            match (args[1].as_str(),args[2].as_str(),args[3].as_str()) {
                ("exp_load_start", starter_path, components_dir) => {
                    exp_load_start(starter_path, components_dir).await.unwrap();
                },
                _ => panic!("Bad arguments"),        
            }
        }   
        _ => panic!("Bad arguments")        
    }
}
