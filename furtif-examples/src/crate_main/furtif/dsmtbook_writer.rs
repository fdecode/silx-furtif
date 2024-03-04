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
use tokio::fs::write;

use furtif_core::{ structs::{ Assignment, EnumLattice, }, traits::Lattice, };
use silx_core::{ id_tools::IdBuilder, utils::{ 
    ServantBuilderParameters, ServantBuilder, SendToMaster, ProcessProducer, ProcessInstance, 
    produce_future, produce_read,
}, types::SlxInto, };
use super::SerLang;

/// Servant builder for writer (DSmT book example)
#[derive(Serialize, Deserialize, Clone,)]
pub struct DsmtbookWriterBuilder {
    channel_writer: String,
    channel_lattice: String,
    serializer: SerLang,
    file: String,
}
 
impl DsmtbookWriterBuilder {
    #[allow(dead_code)]
    /// Constructor for DsmtbookWriterBuilder (DSmT book example)
    /// * `channel_writer: String` : channel to connect the writer
    /// * `channel_lattice: String` : channel for getting lattice definition
    /// * `serializer: SerLang` : language choice for serializing  
    /// * `file: String` : file name to be written
    /// * Output : builder
    pub fn new(
        channel_writer: String, 
        channel_lattice: String,
        serializer: SerLang,
        file: String,
    ) -> Self { Self { channel_writer, serializer, channel_lattice, file } }
}

// This line is mandatory
#[typetag::serde] impl ServantBuilder for DsmtbookWriterBuilder { }

impl ServantBuilderParameters for DsmtbookWriterBuilder {
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
            Err(_) => { eprintln!("Writer:: failed to produce lattice"); panic!(); },
        };
        // build channel receiver of type `Vec<Assignment<<EnumLattice as Lattice>::Item>>`, of name `self.channel_writer` and capacity `1`
        let read_recv = match produce_read!(
            producer,Vec<Assignment<<EnumLattice as Lattice>::Item>>,self.channel_writer, Some(1)
        ) {
            Ok(rr) => rr,
            Err(_) => { eprintln!("Writer:: failed to produce read"); panic!(); },
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
                Err(_) => { eprintln!("Writer:: failed to recv lattice"); panic!() },
            }.unarchive() { // and unarchive it
                // * unarchive is the most simple way but slighly costy because of the deserialization
                // * another way, generally more efficient but more complex, is to use zero copy reference to the archive content by means of data.archive_ref()
                Ok(u) => u,
                Err(_) => { eprintln!("Writer:: failed to unarchive lattice"); panic!(); },
            };
            // display lattice hash 
            println!("Writer::lattice_hash -> {}", lattice.lattice_hash());
            // receive fused data,
            let fused = match match read_recv.recv().await {
                Ok(v) => v,
                Err(_) => { eprintln!("Writer:: failed to read_recv"); panic!(); },
            }.unarchive() { // unarchive it,
                Ok(u) => u,
                Err(_) => { eprintln!("Writer:: failed to unarchive"); panic!(); },
            }.into_iter().map(|a| { // and map each fused assignment
                // into a friendly representation (name of proposition with its weight)
                a.into_iter().map(|(se,w)| (match lattice.to_string(&se) {
                    Ok(s) => s,
                    Err(_) => { eprintln!("Writer:: to_string failed"); panic!(); },
                },w.unslx())).collect::<Vec<_>>()
            }).collect::<Vec<_>>(); // collect all
            /* 
            // ////////////////////////////
            // Alternative implementation

            // a variant without unarchiving, which uses .archive_ref() for non-copy reference to the data
            let fused = match match read_recv.recv().await {
                Ok(v) => v,
                Err(_) => { eprintln!("Writer:: failed to read_recv"); panic!(); },
            }.archive_ref() {
                Ok(u) => u,
                Err(_) => { eprintln!("Writer:: failed to archive_ref"); panic!(); },
            }.into_iter().map(|a| {
                let lattice_hash = a.lattice_hash;
                a.elements.iter().map(|(element,w)|(
                    match lattice.to_string(& unsafe { SafeElement::unsafe_new(*element, lattice_hash) }) {
                        Ok(s) => s,
                        Err(_) => { eprintln!("Writer:: failed to to_string"); panic!(); },
                    }, (*w).unslx()
                )).collect::<Vec<_>>()
            }).collect::<Vec<_>>();
            */
            // serialize fused result in accordance with format
            let fused_str: String = match serializer { 
                SerLang::Json => match serde_json::to_string(&fused) {
                    Ok(s) => s,
                    Err(_) => { eprintln!("Writer:: failed to serialize (json)"); panic!(); },
                },
                SerLang::Yaml => match serde_yaml::to_string(&fused) {
                    Ok(s) => s,
                    Err(_) => { eprintln!("Writer:: failed to serialize (yaml)"); panic!(); },
                },
            };
            // write serialized data to file
            match write(file_name, fused_str).await { Ok(_) => (), Err(_) => { eprintln!("Writer:: failed to write file"); panic!(); }, };
        })
    }
}
