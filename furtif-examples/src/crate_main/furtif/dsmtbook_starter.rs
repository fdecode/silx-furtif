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


use std::{ collections::HashMap, net::SocketAddr, path::Path, time::Duration };

use furtif_core::structs::{EnumRule, EnumLattice};
use silx_core::{ 
    servants::shutdown::ShutdownBuilder, 
    utils::{ StarterProducer, FiledStarter, RecFiled, },
};

use super::{ DsmtbookReaderBuilder, DsmtbookFuserBuilder, DsmtbookWriterBuilder, SerLang, DsmtbookLatticeBuilder, };

// ======= BUILDING STARTER

#[allow(dead_code)]
/// build starters loaders for the DSmT Book 5 example (with 4 clusters)
/// * `lattice: EnumLattice` : lattice used by the network
/// * `referee: EnumRule` : referee function used by the network
/// * `main_addr: SocketAddr` : IP adresse of main servant
/// * `slave_reader1_addr: SocketAddr` : IP adresse of slave servant 1
/// * `slave_reader2_addr: SocketAddr` : IP adresse of slave servant 2
/// * `slave_reader3_addr: SocketAddr` : IP adresse of slave servant 3
/// * `main_starter_path: P` : path of starter file of main servant 
/// * `slave_reader1_starter_path: Q` : path of starter file of slave servant 1
/// * `slave_reader2_starter_path: R` : path of starter file of slave servant 2
/// * `slave_reader3_starter_path: S` : path of starter file of slave servant 3 
/// * `input_1: (SerLang,String)` : serialization langage and input file path of source 1
/// * `input_2: (SerLang,String)` : serialization langage and input file path of source 2
/// * `input_3: (SerLang,String)` : serialization langage and input file path of source 3
/// * `output: (SerLang,String)` : serialization langage and output file path of fused result
/// * `P: AsRef<Path>` : type of starter file path of main servant
/// * `Q: AsRef<Path>` : type of starter file path of slave servant 1
/// * `R: AsRef<Path>` : type of starter file path of slave servant 2
/// * `S: AsRef<Path>` : type of starter file path of slave servant 3
/// * Output: sequence of starters loaders indexed with their IP addresses 
pub fn build_dsmtbook_starter<P: AsRef<Path>,Q: AsRef<Path>,R: AsRef<Path>,S: AsRef<Path>> (
    lattice: EnumLattice,
    referee: EnumRule,
    main_addr: SocketAddr,
    slave_reader1_addr: SocketAddr, slave_reader2_addr: SocketAddr, slave_reader3_addr: SocketAddr,
    main_starter_path: P, 
    slave_reader1_starter_path: Q, slave_reader2_starter_path: R, slave_reader3_starter_path: S,
    input_1: (SerLang,String), input_2: (SerLang,String), input_3: (SerLang,String),
    output: (SerLang,String),
) -> Result<HashMap<SocketAddr,RecFiled<FiledStarter>>,String> {
    // get main starter directory 
    let parent_main = match main_starter_path.as_ref().parent() { 
        None => "./".to_string(),
        Some(parent) => match parent.to_str() {
            Some(p) => p.to_string(), None => return Err(format!("failed to detect main parent")),
        },
    };
    // get slave 1 starter directory 
    let parent_slave_reader1 = match slave_reader1_starter_path.as_ref().parent() { 
        None => "./".to_string(),
        Some(parent) => match parent.to_str() {
            Some(p) => p.to_string(), None => return Err(format!("failed to detect main parent")),
        },
    };
    // get slave 2 starter directory 
    let parent_slave_reader2 = match slave_reader2_starter_path.as_ref().parent() { 
        None => "./".to_string(),
        Some(parent) => match parent.to_str() {
            Some(p) => p.to_string(), None => return Err(format!("failed to detect main parent")),
        },
    };
    // get slave 3 starter directory 
    let parent_slave_reader3 = match slave_reader3_starter_path.as_ref().parent() { 
        None => "./".to_string(),
        Some(parent) => match parent.to_str() {
            Some(p) => p.to_string(), None => return Err(format!("failed to detect main parent")),
        },
    };
    // set builder paths for main, slave1, slave2, and slave3
    let main_builder_path = format!("{parent_main}/builders/main_builder.yaml");
    let slave_reader1_builder_path = format!("{parent_slave_reader1}/builders/slave_reader1_builder.yaml"); 
    let slave_reader2_builder_path = format!("{parent_slave_reader2}/builders/slave_reader2_builder.yaml");
    let slave_reader3_builder_path = format!("{parent_slave_reader3}/builders/slave_reader3_builder.yaml");
    // set control channel capacity
    let ctrl_ch_capacity = 16;
    // set networked channel capacity
    let net_size = Some(16);
    // set max ping for failure detection (1s ; this is much longer than necessary)
    let max_ping = Duration::from_millis(1000);

    // set the starters producer
    let start_prod = StarterProducer::new(
        // producer is initialized with main cluster parameters
        main_addr, main_starter_path, main_builder_path, net_size, ctrl_ch_capacity
    ).add_cluster(
        // adding slave1 cluster parameters
        slave_reader1_addr, slave_reader1_starter_path, slave_reader1_builder_path, net_size, ctrl_ch_capacity
    )?.add_cluster(
        // adding slave2 cluster parameters
        slave_reader2_addr, slave_reader2_starter_path, slave_reader2_builder_path, net_size, ctrl_ch_capacity
    )?.add_cluster(
        // adding slave3 cluster parameters
        slave_reader3_addr, slave_reader3_starter_path, slave_reader3_builder_path, net_size, ctrl_ch_capacity
    )?.done(); // then, the starters producer is completed

    // add proceses to the producer
    let start_prod = start_prod.add_process(
        &slave_reader1_addr, format!("reader_1"), // add process reader_1 to slave cluster 1
        format!("{parent_main}/servants/servant_reader1.yaml"), // process definition is serialized within servant_reader1.yaml 
        DsmtbookReaderBuilder::new( // call the constructor of this process
            "Reader_1".to_string(), "Lattice_1".to_string(), input_1.0, input_1.1,
        ),
    )?.add_process(
        &slave_reader2_addr, format!("reader_2"), // add process reader_2 to slave cluster 2 
        format!("{parent_main}/servants/servant_reader2.yaml"), // process definition is serialized within servant_reader2.yaml
        DsmtbookReaderBuilder::new( // call the constructor of this process
            "Reader_2".to_string(), "Lattice_2".to_string(), input_2.0, input_2.1,
        ),
    )?.add_process( 
        &slave_reader3_addr, format!("reader_3"), // add process reader_3 to slave cluster 3
        format!("{parent_main}/servants/servant_reader3.yaml"), // process definition is serialized within servant_reader3.yaml 
        DsmtbookReaderBuilder::new( // call the constructor of this process
            "Reader_3".to_string(), "Lattice_3".to_string(), input_3.0, input_3.1,
        ),
    )?.add_process(
        &main_addr, format!("lattice"), // add process lattice to main cluster 
        format!("{parent_main}/servants/servant_lattice.yaml"), // process definition is serialized within servant_lattice.yaml
        DsmtbookLatticeBuilder::new(vec![ // call the constructor of this process
            "Lattice_0".to_string(),
            "Lattice_1".to_string(),
            "Lattice_2".to_string(),
            "Lattice_3".to_string(),
        ], lattice,),
    )?.add_process(
        &main_addr, format!("writer"), // add process writer to main cluster
        format!("{parent_main}/servants/servant_writer.yaml"), // process definition is serialized within servant_writer.yaml
        DsmtbookWriterBuilder::new( // call the constructor of this process
            "Writer".to_string(), "Lattice_0".to_string(), output.0, output.1,
        ),
    )?.add_process(
        &main_addr, format!("shutdown_0"), // add process shutdown_0 to main cluster
        format!("{parent_main}/servants/servant_shutdown0.yaml"), // process definition is serialized within servant_shutdown0.yaml
        ShutdownBuilder::new(format!("Shutdown_0")), // call the constructor of this process
    )?.add_process(
        &slave_reader1_addr, format!("shutdown_1"), // add process shutdown_1 to slave cluster 1
        format!("{parent_main}/servants/servant_shutdown1.yaml"), // process definition is serialized within servant_shutdown1.yaml
        ShutdownBuilder::new(format!("Shutdown_1")), // call the constructor of this process
    )?.add_process(
        &slave_reader2_addr, format!("shutdown_2"), // add process shutdown_2 to slave cluster 2
        format!("{parent_main}/servants/servant_shutdown2.yaml"), // process definition is serialized within servant_shutdown2.yaml
        ShutdownBuilder::new(format!("Shutdown_2")), // call the constructor of this process
    )?.add_process(
        &slave_reader3_addr, format!("shutdown_3"), // add process shutdown_3 to slave cluster 3
        format!("{parent_main}/servants/servant_shutdown3.yaml"), // process definition is serialized within servant_shutdown3.yaml
        ShutdownBuilder::new(format!("Shutdown_3")), // call the constructor of this process
    )?.add_process(
        &main_addr, format!("fuser"), // add process fuser to main cluster
        format!("{parent_main}/servants/servant_fuser.yaml"), // process definition is serialized within servant_fuser.yaml
        DsmtbookFuserBuilder::new( // call the constructor of this process
            "Lattice_0".to_string(),
            referee, 
            vec!["Reader_1".to_string(),"Reader_2".to_string(),"Reader_3".to_string(),], 
            "Writer".to_string(), 
            vec![ "Shutdown_0".to_string(), 
                "Shutdown_1".to_string(),"Shutdown_2".to_string(),"Shutdown_3".to_string(),
            ]
        ),
    )?.done(); // then, the starters producer with processes is completed

    // add channels to the producer (all channels are saved within main directory in this example)
    let ok = Ok(start_prod.add_net_broadcast( // add broadcast channel between two clusters
        format!("{parent_main}/channels/channel_reader1.yaml"), // channel definition is serialized within channel_reader1.yaml
        format!("Reader_1"), // channel name
        slave_reader1_addr, [format!("reader_1"),], // input cluster and input servants
        main_addr, [format!("fuser"),], // output cluster and output servants
        max_ping, 16,
    )?.add_net_broadcast( // add broadcast channel between two clusters
        format!("{parent_main}/channels/channel_reader2.yaml"), // channel definition is serialized within channel_reader2.yaml
        format!("Reader_2"), // channel name 
        slave_reader2_addr, [format!("reader_2"),], // input cluster and input servants 
        main_addr, [format!("fuser"),], // output cluster and output servants 
        max_ping, 16,
    )?.add_net_broadcast( // add broadcast channel between two clusters
        format!("{parent_main}/channels/channel_reader3.yaml"), // channel definition is serialized within channel_reader3.yaml
        format!("Reader_3"), // channel name 
        slave_reader3_addr, [format!("reader_3"),], // input cluster and input servants 
        main_addr, [format!("fuser"),], // output cluster and output servants 
        max_ping, 16,
    )?.add_net_broadcast( // add broadcast channel between two clusters
        format!("{parent_main}/channels/channel_lattice1.yaml"), // channel definition is serialized within channel_lattice1.yaml
        format!("Lattice_1"), // channel name 
        main_addr, [format!("lattice"),], // input cluster and input servants 
        slave_reader1_addr, [format!("reader_1"),], // output cluster and output servants 
        max_ping, 16,
    )?.add_net_broadcast( // add broadcast channel between two clusters
        format!("{parent_main}/channels/channel_lattice2.yaml)"), // channel definition is serialized within channel_lattice2.yaml
        format!("Lattice_2"), // channel name 
        main_addr, [format!("lattice"),], // input cluster and input servants 
        slave_reader2_addr, [format!("reader_2"),], // output cluster and output servants 
        max_ping, 16,
    )?.add_net_broadcast( // add broadcast channel between two clusters
        format!("{parent_main}/channels/channel_lattice3.yaml"), // channel definition is serialized within channel_lattice3.yaml
        format!("Lattice_3"), // channel name 
        main_addr, [format!("lattice"),], // input cluster and input servants 
        slave_reader3_addr, [format!("reader_3"),], // output cluster and output servants 
        max_ping, 16,
    )?.add_broadcast( // add broadcast channel within the same cluster
        format!("{parent_main}/channels/channel_lattice0.yaml"), // channel definition is serialized within channel_lattice0.yaml
        format!("Lattice_0"), // channel name 
        main_addr, // cluster address
        [format!("lattice"),], // input servants 
        [format!("fuser"),format!("writer"),], // output servants  
        max_ping, 16,
    )?.add_broadcast( // add broadcast channel within the same cluster
        format!("{parent_main}/channels/channel_writer.yaml"), // channel definition is serialized within channel_writer.yaml
        format!("Writer"), // channel name 
        main_addr, // cluster address
        [format!("fuser"),], // input servants
        [format!("writer"),], // output servants
        max_ping, 16,
    )?.add_broadcast( // add broadcast channel within the same cluster
        format!("{parent_main}/channels/channel_shutdown0.yaml"), // channel definition is serialized within channel_shutdown0.yaml
        format!("Shutdown_0"), // channel name 
        main_addr, // cluster address
        [format!("fuser"),] , // input servants
        [format!("shutdown_0"),], // output servants
        max_ping, 16,
    )?.add_net_broadcast( // add broadcast channel between two clusters
        format!("{parent_main}/channels/channel_shutdown1.yaml"), // channel definition is serialized within channel_shutdown1.yaml
        format!("Shutdown_1"), // channel name 
        main_addr, [format!("fuser"),], // input cluster and input servants 
        slave_reader1_addr, [format!("shutdown_1"),], // output cluster and output servants 
        max_ping, 16,
    )?.add_net_broadcast( // add broadcast channel between two clusters
        format!("{parent_main}/channels/channel_shutdown2.yaml"), // channel definition is serialized within channel_shutdown2.yaml
        format!("Shutdown_2"), // channel name 
        main_addr, [format!("fuser"),], // input cluster and input servants 
        slave_reader2_addr, [format!("shutdown_2"),], // output cluster and output servants 
        max_ping, 16,
    )?.add_net_broadcast( // add broadcast channel between two clusters
        format!("{parent_main}/channels/channel_shutdown3.yaml"), // channel definition is serialized within channel_shutdown3.yaml
        format!("Shutdown_3"), // channel name 
        main_addr, [format!("fuser"),], // input cluster and input servants 
        slave_reader3_addr, [format!("shutdown_3"),], // output cluster and output servants 
        max_ping, 16,
    )?.done());
    ok
}



// ======= BUILDING STARTER

/// build starter loader for the DSmT Book 5 example (with 1 cluster)
/// * `lattice: EnumLattice` : lattice used by the network
/// * `referee: EnumRule` : referee function used by the network
/// * `main_addr: SocketAddr` : IP adresse of main servant
/// * `main_starter_path: P` : path of starter file of main servant 
/// * `input_1: (SerLang,String)` : serialization langage and input file path of source 1
/// * `input_2: (SerLang,String)` : serialization langage and input file path of source 2
/// * `input_3: (SerLang,String)` : serialization langage and input file path of source 3
/// * `output: (SerLang,String)` : serialization langage and output file path of fused result
/// * `P: AsRef<Path>` : type of starter file path of main servant
/// * Output: sequence of starters (actually only main) loaders indexed with their IP addresses 
#[allow(dead_code)]
pub fn build_dsmtbook_starter_mono<P: AsRef<Path>> (
    lattice: EnumLattice,
    referee: EnumRule,
    main_addr: SocketAddr,
    main_starter_path: P, 
    input_1: (SerLang,String), input_2: (SerLang,String), input_3: (SerLang,String),
    output: (SerLang,String),
) -> Result<HashMap<SocketAddr,RecFiled<FiledStarter>>,String> {
    // get main starter directory 
    let parent_main = match main_starter_path.as_ref().parent() { 
        None => "./".to_string(),
        Some(parent) => match parent.to_str() {
            Some(p) => p.to_string(), None => return Err(format!("failed to detect main parent")),
        },
    };
    // set builder paths for main
    let main_builder_path = format!("{parent_main}/builders/main_builder.yaml");
    // set control channel capacity
    let ctrl_ch_capacity = 16;
    // set networked channel capacity
    let net_size = Some(16);
    // set max ping for failure detection (1s ; this is much longer than necessary)
    let max_ping = Duration::from_millis(1000);
    
    // set the starters producer
    let start_prod = StarterProducer::new(
        // producer is initialized with main cluster parameters
        main_addr, main_starter_path, main_builder_path, net_size, ctrl_ch_capacity
    ).done();

    // add proceses to the producer
    let start_prod = start_prod.add_process(
        &main_addr, format!("reader_1"), // add process reader_1 to main cluster 
        format!("{parent_main}/servants/servant_reader1.yaml"), // process definition is serialized within servant_reader1.yaml 
        DsmtbookReaderBuilder::new( // call the constructor of this process 
            "Reader_1".to_string(), "Lattice".to_string(), input_1.0, input_1.1,
        ),
    )?.add_process(
        &main_addr, format!("reader_2"), // add process reader_2 to main cluster
        format!("{parent_main}/servants/servant_reader2.yaml"), // process definition is serialized within servant_reader2.yaml 
        DsmtbookReaderBuilder::new( // call the constructor of this process
            "Reader_2".to_string(), "Lattice".to_string(), input_2.0, input_2.1,
        ),
    )?.add_process(
        &main_addr, format!("reader_3"), // add process reader_3 to main cluster 
        format!("{parent_main}/servants/servant_reader3.yaml"), // process definition is serialized within servant_reader3.yaml 
        DsmtbookReaderBuilder::new( // call the constructor of this process
            "Reader_3".to_string(), "Lattice".to_string(), input_3.0, input_3.1,
        ),
    )?.add_process(
        &main_addr, format!("lattice"), // add process lattice to main cluster 
        format!("{parent_main}/servants/servant_lattice.yaml"), // process definition is serialized within servant_lattice.yaml 
        DsmtbookLatticeBuilder::new(vec!["Lattice".to_string(),], lattice,), // call the constructor of this process
    )?.add_process(
        &main_addr, format!("writer"), // add process writer to main cluster 
        format!("{parent_main}/servants/servant_writer.yaml"), // process definition is serialized within servant_writer.yaml 
        DsmtbookWriterBuilder::new( // call the constructor of this process
            "Writer".to_string(), "Lattice".to_string(), output.0, output.1,
        ),
    )?.add_process(
        &main_addr, format!("shutdown"), // add process shutdown to main cluster 
        format!("{parent_main}/servants/servant_shutdown.yaml"), // process definition is serialized within servant_shutdown.yaml 
        ShutdownBuilder::new(format!("Shutdown")), // call the constructor of this process
    )?.add_process(
        &main_addr, format!("fuser"), // add process fuser to main cluster 
        format!("{parent_main}/servants/servant_fuser.yaml"), // process definition is serialized within servant_fuser.yaml 
        DsmtbookFuserBuilder::new( // call the constructor of this process
            "Lattice".to_string(),
            referee, 
            vec!["Reader_1".to_string(),"Reader_2".to_string(),"Reader_3".to_string(),], 
            "Writer".to_string(), 
            vec![ "Shutdown".to_string(),]
        ),
    )?.done();

    // add channels to the producer (all channels are saved within main directory in this example)
    let ok = Ok(start_prod.add_broadcast( // add broadcast channel within the same cluster
        format!("{parent_main}/channels/channel_reader1.yaml"), // channel definition is serialized within channel_reader1.yaml
        format!("Reader_1"), // channel name 
        main_addr, // cluster address
        [format!("reader_1"),], // input servants
        [format!("fuser"),], // output servants
        max_ping, 16,
    )?.add_broadcast( // add broadcast channel within the same cluster
        format!("{parent_main}/channels/channel_reader2.yaml"), // channel definition is serialized within channel_reader2.yaml
        format!("Reader_2"), // channel name 
        main_addr, // cluster address
        [format!("reader_2"),], // input servants
        [format!("fuser"),], // output servants
        max_ping, 16,
    )?.add_broadcast( // add broadcast channel within the same cluster
        format!("{parent_main}/channels/channel_reader3.yaml"), // channel definition is serialized within channel_reader3.yaml
        format!("Reader_3"), // channel name 
        main_addr, // cluster address
        [format!("reader_3"),], // input servants
        [format!("fuser"),], // output servants
        max_ping, 16,
    )?.add_broadcast( // add broadcast channel within the same cluster
        format!("{parent_main}/channels/channel_lattice.yaml"), // channel definition is serialized within channel_lattice.yaml
        format!("Lattice"), // channel name 
        main_addr, // cluster address
        [format!("lattice"),], // input servants
        [ // output servants
            format!("fuser"),format!("writer"), format!("reader_1"), format!("reader_2"), format!("reader_3"),
        ], 
        max_ping, 16,
    )?.add_broadcast( // add broadcast channel within the same cluster
        format!("{parent_main}/channels/channel_writer.yaml"), // channel definition is serialized within channel_writer.yaml
        format!("Writer"), // channel name 
        main_addr, // cluster address
        [format!("fuser"),], // input servants
        [format!("writer"),], // output servants
        max_ping, 16,
    )?.add_broadcast( // add broadcast channel within the same cluster
        format!("{parent_main}/channels/channel_shutdown.yaml"), // channel definition is serialized within channel_shutdown.yaml
        format!("Shutdown"), // channel name 
        main_addr, // cluster address
        [format!("fuser"),] , // input servants
        [format!("shutdown"),], // output servants
        max_ping, 16,
    )?.done());
    ok
}