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


use std::{net::{SocketAddr, IpAddr, Ipv4Addr}, path::{ PathBuf, Path, }, time::Duration};
use furtif_core::structs::{EnumLattice, EnumRule, TaxonomyBuilder, Taxonomy, };
use serde::{Serialize, Deserialize};
use silx_core::utils::Filable;
use tokio::{ fs::{DirBuilder, write, }, spawn, time::sleep };

// get the network launcher
use crate::crate_main::exp_load_start;

// ======= SERVANTS DEFINITIONS FURTIF

/// definition of Reader servant of DSmbook 5 example
pub mod dsmtbook_reader; 
pub use self::dsmtbook_reader::DsmtbookReaderBuilder;

/// definition of Writer servant of DSmbook 5 example
pub mod dsmtbook_writer; 
pub use self::dsmtbook_writer::DsmtbookWriterBuilder;

/// definition of Fuser servant of DSmbook 5 example
pub mod dsmtbook_fuser; 
pub use self::dsmtbook_fuser::DsmtbookFuserBuilder;

/// definition of Lattice definition servant of DSmbook 5 example
pub mod dsmtbook_lattice; 
pub use self::dsmtbook_lattice::DsmtbookLatticeBuilder;

/// definition of starters of DSmbook 5 example
pub mod dsmtbook_starter; 
pub use self::dsmtbook_starter::{ build_dsmtbook_starter, build_dsmtbook_starter_mono, };

/// Serialization language selector
#[derive(Serialize, Deserialize, Clone, Copy,)]
pub enum SerLang{
    Json,
    Yaml
}

/// Example (from DSmT book 5) of three same sized BBA sequences defined on taxonomy
/// * Output: a taxonomy and three BBA sequences defined on it
pub fn taxonomy_bba() -> (Taxonomy,[Vec<Vec<(String,f64)>>;3]) {
    // build the taxon leaves, Car, Truck, Bike, Airplane, UAV, Ship, Boat, Hovercraft, with respective priors 0.2, 0.15, 0.15, 0.1, 0.1, 0.1, 0.15, 0.05
    let taxon_car = TaxonomyBuilder::new_leaf("Car".to_string(), 0.2);
    let taxon_truck = TaxonomyBuilder::new_leaf("Truck".to_string(), 0.15);
    let taxon_bike = TaxonomyBuilder::new_leaf("Bike".to_string(), 0.15);
    let taxon_airplane = TaxonomyBuilder::new_leaf("Airplane".to_string(), 0.1);
    let taxon_uav = TaxonomyBuilder::new_leaf("UAV".to_string(), 0.1);
    let taxon_ship = TaxonomyBuilder::new_leaf("Ship".to_string(), 0.1);
    let taxon_boat = TaxonomyBuilder::new_leaf("Boat".to_string(), 0.15);
    let taxon_hovercraft = TaxonomyBuilder::new_leaf("Hovercraft".to_string(), 0.05);
    // build the taxon nodes, Ground, Air, Water, Amphibian, Object (taxonomy root), and insert taxon respective childs 
    let taxon_ground = TaxonomyBuilder::new_node(
        "Ground".to_string(), vec![taxon_car,taxon_truck,taxon_bike],
    ).unwrap();
    let taxon_air = TaxonomyBuilder::new_node(
        "Air".to_string(), vec![taxon_airplane, taxon_uav],
    ).unwrap();
    let taxon_water = TaxonomyBuilder::new_node(
        "Water".to_string(), vec![taxon_ship,taxon_boat],
    ).unwrap();
    let taxon_amphibian = TaxonomyBuilder::new_node(
        "Amphibian".to_string(), vec![taxon_hovercraft],
    ).unwrap();
    let taxon_root = TaxonomyBuilder::new_node(
        "Object".to_string(), vec![taxon_ground,taxon_air,taxon_water, taxon_amphibian],
    ).unwrap();
    // build the taxonomy from the root
    let taxonomy = Taxonomy::new(&taxon_root).unwrap();
    // define three bba sequenc (three sources of information) in the form of sequences of weighted strings
    let bba1 = {
        let bba10 = vec![
            ("Object".to_string(), 0.2),("Air".to_string(), 0.3),("Truck".to_string(), 0.5),
        ];
        let bba11 = vec![
            ("Object".to_string(), 0.3), ("Ground".to_string(), 0.4), ("Hovercraft".to_string(), 0.3),
        ];
        vec![bba10,bba11]
    };
    let bba2 = {
        let bba20 = vec![
            ("Object".to_string(), 0.1), ("Amphibian".to_string(), 0.4), ("UAV".to_string(), 0.5),
        ];
        let bba21 = vec! [
            ("Object".to_string(), 0.4), ("Car".to_string(), 0.2), ("Water".to_string(), 0.3),
        ];
        vec![bba20,bba21]
    };
    let bba3 = {
        let bba30 = vec![
            ("Object".to_string(), 0.2), ("Ground".to_string(), 0.2), ("Bike".to_string(), 0.6),
        ];
        let bba31 = vec![
            ("Object".to_string(), 0.4), ("Air".to_string(), 0.4), ("Ship".to_string(), 0.2),
        ];
        vec![bba30,bba31]
    };
    (taxonomy,[bba1,bba2,bba3])
}

/// build and save bbas, and define save directory, lattice and referee to be defined for the DSmT book
/// * Output: either an error or a tuple with:
///    * save directory
///    * lattice
///    * referee function
///    * input files setting
///    * output file setting
pub async fn dsmtbook_setting() -> Result<(PathBuf, EnumLattice, EnumRule, [(SerLang,String);3], (SerLang,String)),String> {
    // ======= BUILDING DSmTbook 5 data

    // build the taxonomy and the bbas sequence for the DSmTbook 5 example
    let (taxonomy,bbas) = taxonomy_bba();
    // define data saving directory
    let save_path_str = "main_examples_data/saved_data/";
    let save_path = PathBuf::from(save_path_str);
    // define names for serialized input files and serialized output file
    let input_1 = (SerLang::Json,format!("{save_path_str}dsmtbook/data/input_1.json"));
    let input_2 = (SerLang::Yaml,format!("{save_path_str}dsmtbook/data/input_2.yaml"));
    let input_3 = (SerLang::Yaml,format!("{save_path_str}dsmtbook/data/input_3.yaml"));
    let output = (SerLang::Yaml,format!("{save_path_str}dsmtbook/data/output.yaml"));
    { // serialize the bbas and save the bbas sequences
        let [ // get the  three bbas sequences
            bba1,bba2,bba3
        ] = bbas;
        if let Some(path) = Path::parent(input_1.1.as_ref()) { // create file directory for input 1, if necessary
            match DirBuilder::new().recursive(true).create(path).await {
                Ok(_) => (), Err(_) => return Err("Failed to create directory".to_string()),
            };
        }
        if let Some(path) = Path::parent(input_2.1.as_ref()) { // create file directory for input 2, if necessary
            match DirBuilder::new().recursive(true).create(path).await {
                Ok(_) => (), Err(_) => return Err("Failed to create directory".to_string()),
            };
        }
        if let Some(path) = Path::parent(input_3.1.as_ref()) { // create file directory for input 3, if necessary
            match DirBuilder::new().recursive(true).create(path).await {
                Ok(_) => (), Err(_) => return Err("Failed to create directory".to_string()),
            };
        }
        if let Some(path) = Path::parent(output.1.as_ref()) { // create file directory for output, if necessary
            match DirBuilder::new().recursive(true).create(path).await {
                Ok(_) => (), Err(_) => return Err("Failed to create directory".to_string()),
            };
        }
        let data_str_1: String = match input_1.0 { // serialize bba1 with respect to the serialization language
            SerLang::Json => match serde_json::to_string_pretty(&bba1) {
                Ok(s) => s, Err(_) => return Err("Failed to serialize".to_string()),
            },
            SerLang::Yaml => match serde_yaml::to_string(&bba1) {
                Ok(s) => s, Err(_) => return Err("Failed to serialize".to_string()),
            },
        };
        let data_str_2: String = match input_2.0 { // serialize bba2 with respect to the serialization language
            SerLang::Json => match serde_json::to_string_pretty(&bba2) {
                Ok(s) => s, Err(_) => return Err("Failed to serialize".to_string()),
            },
            SerLang::Yaml => match serde_yaml::to_string(&bba2) {
                Ok(s) => s, Err(_) => return Err("Failed to serialize".to_string()),
            },
        };
        let data_str_3: String = match input_3.0 { // serialize bba3 with respect to the serialization language
            SerLang::Json => match serde_json::to_string_pretty(&bba3) {
                Ok(s) => s, Err(_) => return Err("Failed to serialize".to_string()),
            },
            SerLang::Yaml => match serde_yaml::to_string(&bba3) {
                Ok(s) => s, Err(_) => return Err("Failed to serialize".to_string()),
            },
        };
        match write(&input_1.1, &data_str_1).await { // write serialized data of bba1 into file of input 1
            Ok(_) => (), Err(_) => return Err("Failed to write file".to_string()),
        };
        match write(&input_2.1, &data_str_2).await { // write serialized data of bba2 into file of input 2
            Ok(_) => (), Err(_) => return Err("Failed to write file".to_string()),
        };
        match write(&input_3.1, &data_str_3).await { // write serialized data of bba3 into file of input 3
            Ok(_) => (), Err(_) => return Err("Failed to write file".to_string()),
        };
    }
    // define lattice
    let lattice = EnumLattice::Taxonomy{ taxonomy };
    // define referee
    let referee = EnumRule::Pcr6;
    Ok((save_path,lattice,referee,[input_1,input_2,input_3],output))
}

/// Experimentation for the DSmT book: the network is made up of 4 clusters operating on 4 different machine addresses, the main, and 3 slaves 
/// * Output: nothing or an error message
pub async fn exp_dsmtbook() -> Result<(),String> {
    // ======= BUILDING NETWORK FROM SCRATCH AND RUNNING IT
    println!("=================================================");
    println!("==                                             ==");
    println!("==   (a) Starters defined from scratch         ==");
    println!("==   (b) Starters definitions saved on disk    ==");
    println!("==                                             ==");
    println!("=================================================");
    println!();

    // define setting for DSmTbook example and get save path, lattice and refereee function
    let (
        save_path, lattice, referee,
        [input_1,input_2,input_3],output
    ) = dsmtbook_setting().await?;

    // ======= STARTERS DEFINITIONS

    // define net address of the 4 clusters: main, slave1, slave2, slave3
    let main_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8180);
    let slave_reader1_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8181);
    let slave_reader2_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8182);
    let slave_reader3_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8183);

    // define file path of the starters for main, slave1, slave2, slave3
    let main_starter_path = "dsmtbook/starters/main/main.yaml";
    let slave_reader1_starter_path = "dsmtbook/starters/slave1/slave_reader1.yaml";
    let slave_reader2_starter_path = "dsmtbook/starters/slave2/slave_reader2.yaml";
    let slave_reader3_starter_path = "dsmtbook/starters/slave3/slave_reader3.yaml";

    // construction of the starters loaders
    let mut starters = build_dsmtbook_starter(
        lattice, referee, main_addr, slave_reader1_addr, slave_reader2_addr, slave_reader3_addr,
        main_starter_path, 
        slave_reader1_starter_path, slave_reader2_starter_path, slave_reader3_starter_path,
        input_1, input_2, input_3, output,
    )?;

    // get back all starters loaders
    let mut main_starter = starters.remove(&main_addr).unwrap();
    let mut slave_reader1_starter = starters.remove(&slave_reader1_addr).unwrap();
    let mut slave_reader2_starter = starters.remove(&slave_reader2_addr).unwrap();
    let mut slave_reader3_starter = starters.remove(&slave_reader3_addr).unwrap();
    println!("Starters are defined");

    // save the starters loaders on disk
    let _slave_reader1_starter_in = slave_reader1_starter.unload(Some(&save_path))?;
    let _slave_reader2_starter_in = slave_reader2_starter.unload(Some(&save_path))?;
    let _slave_reader3_starter_in = slave_reader3_starter.unload(Some(&save_path))?;
    let _main_starter_in = main_starter.unload(Some(&save_path))?;
    println!("Starters are saved");
    println!();

    // ======= RUNNING NETWORK BY LOADING IT

    println!("=======================================================");
    println!("==                                                   ==");
    println!("==   (c) run based on definitions loaded from disk   ==");
    println!("==                                                   ==");
    println!("=======================================================");
    println!();
    // load from disk and run slave1 in a spawned process
    let handle_slave_reader1 = spawn({
        let save_path = save_path.to_str().unwrap().to_string();
        let slave_reader1_starter_path = slave_reader1_starter_path.to_string();
        async move {
            // NOTA: main starter should be launched FIRST, because it plays as server and TCP stream connection may be blocking
            sleep(Duration::from_millis(100)).await; // so, sleep awhile so that main starts first
            exp_load_start(&slave_reader1_starter_path, &save_path).await
        }
    });
    // load from disk and run slave2 in a spawned process
    let handle_slave_reader2 = spawn({
        let save_path = save_path.to_str().unwrap().to_string();
        let slave_reader2_starter_path = slave_reader2_starter_path.to_string();
        async move {
            // NOTA: main starter should be launched FIRST, because it plays as server and TCP stream connection may be blocking
            sleep(Duration::from_millis(100)).await; // so, sleep awhile so that main starts first
            exp_load_start(&slave_reader2_starter_path, &save_path).await
        }
    });
    // load from disk and run slave3 in a spawned process
    let handle_slave_reader3 = spawn({
        let save_path = save_path.to_str().unwrap().to_string();
        let slave_reader3_starter_path = slave_reader3_starter_path.to_string();
        async move {
            // NOTA: main starter should be launched FIRST, because it plays as server and TCP stream connection may be blocking
            sleep(Duration::from_millis(100)).await; // so, sleep awhile so that main starts first
            exp_load_start(&slave_reader3_starter_path, &save_path).await
        }
    });
    // load from disk, run main and wait for result; set message error (empty if no error)
    let result0 = match exp_load_start(&main_starter_path, save_path.to_str().unwrap()).await {
        Ok(()) => String::new(),
        Err(e) => format!("main_starter error: {e}"),
    };
    // wait for result of slave1; chain message erreor if any
    let result1 = match handle_slave_reader1.await { 
        Ok(Ok(())) => result0,
        Ok(Err(e)) => format!("{result0}\nslave_starter_reader1 error: {e}"),
        Err(e) => format!("{result0}\nslave_starter_reader1 handle error: {e}"),
    };
    // wait for result of slave2; chain message erreor if any
    let result2 = match handle_slave_reader2.await { 
        Ok(Ok(())) => result1,
        Ok(Err(e)) => format!("{result1}\nslave_starter_reader2 error: {e}"),
        Err(e) => format!("{result1}\nslave_starter_reader2 handle error: {e}"),
    };
    // wait for result of slave3; chain message erreor if any
    let result3 = match handle_slave_reader3.await { 
        Ok(Ok(())) => result2,
        Ok(Err(e)) => format!("{result2}\nslave_starter_reader3 error: {e}"),
        Err(e) => format!("{result2}\nslave_starter_reader3 handle error: {e}"),
    };
    // publish arrors if any
    if result3.is_empty() { Ok(()) } else { Err(result3) }
}


/// Experimentation for the DSmT book: the network is made up of 1 cluster operating on 1 machine address, the main
/// * Output: nothing or an error message
pub async fn exp_dsmtbook_mono() -> Result<(),String> {
    // ======= BUILDING NETWORK FROM SCRATCH AND RUNNING IT
    println!("=================================================");
    println!("==                                             ==");
    println!("==   (a) Starters defined from scratch         ==");
    println!("==   (b) Starters definitions saved on disk    ==");
    println!("==                                             ==");
    println!("=================================================");
    println!();

    // define setting for DSmTbook example and get svae path, lattice and refereee function
    let (
        save_path, lattice, referee,
        [input_1,input_2,input_3],output
    ) = dsmtbook_setting().await?;
    
    // ======= STARTERS DEFINITIONS

    // define net address of the 4 clusters: main, slave1, slave2, slave3
    let main_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8180);

    // define file path of the starters for main, slave1, slave2, slave3
    let main_starter_path = "dsmtbook-mono/starter/main.yaml";

    // construction of the starter loader
    let mut starters = build_dsmtbook_starter_mono(
        lattice, referee, main_addr, main_starter_path, 
        input_1, input_2, input_3, output,
    )?;

    // get back main starter loader
    let mut main_starter = starters.remove(&main_addr).unwrap();
    println!("Starters are defined");

    // save main starter loader on disk
    let _main_starter_in = main_starter.unload(Some(&save_path))?;
    println!("Starters are saved");
    println!();

    // ======= RUNNING NETWORK BY LOADING IT
    println!("=======================================================");
    println!("==                                                   ==");
    println!("==   (c) run based on definitions loaded from disk   ==");
    println!("==                                                   ==");
    println!("=======================================================");
    println!();
    // load from disk, run main and wait for result; set message error (empty if no error)
    match exp_load_start(&main_starter_path, save_path.to_str().unwrap()).await {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("main_starter error: {e}")),
    }
}
