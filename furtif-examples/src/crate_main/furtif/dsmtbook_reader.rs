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


use std::time::Duration;

use serde::{ Serialize, Deserialize, };
use tokio::fs::read_to_string;

use furtif_core::{structs::{Assignment, EnumLattice}, traits::Lattice};
use silx_core::{ id_tools::IdBuilder, utils::{ 
    ServantBuilderParameters, ServantBuilder, SendToMaster, ProcessProducer, ProcessInstance,
    produce_future, produce_emit,
}, types::IntoSlx, produce_read, };
use silx_types::{ u128slx, ArchSized, };

use super::SerLang;

// ======= A SERVANT DEFINITION

/// Servant builder for reader (DSmT book example)
#[derive(Serialize, Deserialize, Clone,)]
pub struct DsmtbookReaderBuilder {
    channel_reader: String,
    channel_lattice: String,
    serializer: SerLang,
    file: String,
}

impl DsmtbookReaderBuilder {
    #[allow(dead_code)]
    /// Constructor for DsmtbookReaderBuilder (DSmT book example)
    /// * `channel_reader: String` : channel to connect the reader
    /// * `channel_lattice: String` : channel for getting lattice definition
    /// * `serializer: SerLang` : language choice for deserializing 
    /// * `file: String` : file name to be read
    /// * Output : builder
    pub fn new(
        channel_reader: String, 
        channel_lattice: String,
        serializer: SerLang,
        file: String,
    ) -> Self { Self { channel_reader, channel_lattice, serializer, file } }
}

// This line is mandatory
#[typetag::serde] impl ServantBuilder for DsmtbookReaderBuilder { }

impl ServantBuilderParameters for DsmtbookReaderBuilder {
    // 10s delay between servant awaits before forcing the servant to stop (this is much longer than necessary)
    fn max_cycle_time(&self) -> Duration { Duration::from_millis(10000) }

    fn build_process(&self, _task_id: IdBuilder, send_to_master: SendToMaster,) -> ProcessInstance {
        // get a process producer ; producer will handle part of the channels and future definitions
        let mut producer = ProcessProducer::new(&send_to_master);

        // ////////////////////////////
        // Building servant channels

        // build channel receiver of type `EnumLattice`, of name `self.channel_lattice` and capacity `1`
        let lattice_recv = match produce_read!(
            producer,EnumLattice,self.channel_lattice, Some(1)
        ) {
            Ok(rr) => rr,
            Err(_) => { eprintln!("Reader:: failed to produce lattice"); panic!(); },
        };
        // build channel senders of type `Vec<Assignment<<EnumLattice as Lattice>::Item>>`, of name `self.channel_reader` and capacity `1`        
        let in_emit_send = match produce_emit!(
            producer, Vec<Assignment<<EnumLattice as Lattice>::Item>>, self.channel_reader, Some(1),
        ) {
            Ok(es) => es,
            Err(_) => { eprintln!("Reader:: failed to produce emit"); panic!(); },
        };
                
        // ////////////////////////////
        // Building servant processes
    
        // clone file name parameter in order to move it into future
        let file_name = self.file.clone();
        // get serializer choice copy in order to move it into future        
        let serializer = self.serializer;
        produce_future!(producer, { // code definition of the future 
            let lattice = match match lattice_recv.recv().await { // receive the archived lattice definition
                Ok(l) => l,
                Err(_) => { eprintln!("Reader:: failed to recv lattice"); panic!() },
            }.unarchive() { // and unarchive it
                // * unarchive is the most simple way but slighly costy because of the deserialization
                // * another way, generally more efficient but more complex, is to use zero copy reference to the archive content by means of data.archive_ref()
                Ok(u) => u,
                Err(_) => { eprintln!("Reader:: failed to unarchive lattice"); panic!(); },
            };
            // display lattice hash 
            println!("Reader::lattice_hash -> {}", lattice.lattice_hash());
            // read serialized data from file
            let ser = match read_to_string(file_name).await {
                Ok(s) => s,
                Err(_) => { eprintln!("Reader:: failed to read to string"); panic!(); },
            };
            // deserialize the input
            let input: Vec<Vec<(String,f64)>> = match serializer { // with respect to the serialization language
                SerLang::Json => match serde_json::from_str(&ser) {
                    Ok(vv) => vv,
                    Err(_) => { eprintln!("Reader:: failed to unserialize"); panic!(); },
                },
                SerLang::Yaml => match serde_yaml::from_str(&ser) {
                    Ok(vv) => vv,
                    Err(_) => { eprintln!("Reader:: failed to unserialize"); panic!(); },
                },
            };
            // build assignments with 128 mid and 256 max lengths 
            let length_mid = 128u32;
            let length_max = 256u32;
            let assignments: Vec<Assignment<u128slx>> = input.into_iter().map(|v| { // map the weighted string sequence
                let mut bba = lattice.prunable(length_mid,length_max);
                for (s,x) in v {
                    match bba.push(lattice.from_str(&s).unwrap(), x.slx()) { // into a weighted lattice sequence
                        Ok(b) => b,
                        Err(_) => { eprintln!("Reader:: failed to push bba"); panic!(); },
                    };
                }
                bba.into() // and transform it into an assignment
            }).collect();
            // archive assignments in order to send it
            let arch_assignments = match assignments.arch_sized() {
                Ok(ad) => ad,
                Err(_) => { eprintln!("Reader:: failed to arch_unsized"); panic!(); },
            };
            // send assignments
            match in_emit_send.send(arch_assignments).await{
                Ok(_) => (),
                Err(_) => { eprintln!("Reader:: failed to emit_send"); panic!(); },
            };
        })
    }
}