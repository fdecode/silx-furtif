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
use tokio::time::sleep;

use furtif_core::{structs::{EnumRule, Assignment, DiscountedFuser, EnumLattice, }, traits::{Lattice, DiscountedFusion}};
use silx_core::{ id_tools::IdBuilder, utils::{ 
    ServantBuilderParameters, ServantBuilder, SendToMaster, ProcessProducer, ProcessInstance, ArchSized,
    produce_emit,
}, produce_read, produce_future};
use silx_types::WakeSlx;

// ======= A SERVANT DEFINITION

#[derive(Serialize, Deserialize, Clone,)]

/// Servant builder for fuser (DSmT book example)
pub struct DsmtbookFuserBuilder { 
    referee: EnumRule,
    channel_lattice: String,
    channels_reader: Vec<String>,
    channel_writer: String,
    channels_shutdown: Vec<String>,
}

impl DsmtbookFuserBuilder {
    #[allow(dead_code)]
    /// Constructor for DsmtbookFuserBuilder (DSmT book example)
    /// * `channel_lattice: String` : channel for getting lattice definition
    /// * `referee: EnumRule` : referee function characterizing the fusion
    /// * `channels_reader: Vec<String>` : channels for connecting to the readers
    /// * `channel_writer: String` : channel for connecting to the writer
    /// * `channels_shutdown: Vec<String>` : channels to send shutdown signal
    /// * Output : builder
    pub fn new(
        channel_lattice: String,
        referee: EnumRule,
        channels_reader: Vec<String>,
        channel_writer: String,
        channels_shutdown: Vec<String>,
    ) -> Self { 
        Self {
            channel_lattice, referee, channels_reader, channel_writer, channels_shutdown,
        } 
    }
}

// This line is mandatory
#[typetag::serde] impl ServantBuilder for DsmtbookFuserBuilder { }

impl ServantBuilderParameters for DsmtbookFuserBuilder {
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
            Err(_) => { eprintln!("Fuser:: failed to produce lattice"); panic!(); },
        };
        // build channel receivers of type `Vec<Assignment<<EnumLattice as Lattice>::Item>>`, of name within `self.channels_reader` and capacity `1`
        let mut readers_recv = Vec::new();
        for rn in self.channels_reader.iter() { 
            readers_recv.push(match produce_read!(
                producer,Vec<Assignment<<EnumLattice as Lattice>::Item>>,rn, Some(1),
            ) {
                Ok(rs) => rs,
                Err(_) => { eprintln!("Fuser:: failed to produce read"); panic!(); },
            }); 
        }
        // build channel sender of type `Vec<Assignment<<EnumLattice as Lattice>::Item>>`, of name `self.channel_writer` and capacity `1`
        let writer_send = match produce_emit!(
            producer,Vec<Assignment<<EnumLattice as Lattice>::Item>>,self.channel_writer, Some(1)
        ) {
            Ok(pe) => pe,
            Err(_) => { eprintln!("Fuser:: failed to produce emit"); panic!(); },
        };
        // build channel senders of type `WakeSlx` (waker), of name within `self.channels_shutdown` and capacity `1`
        let mut shutdowns_send = Vec::new();
        for sn in self.channels_shutdown.iter() { 
            shutdowns_send.push(match produce_emit!(producer,WakeSlx,sn, Some(1),) {
                Ok(pe) => pe,
                Err(_) => { eprintln!("Fuser:: failed to produce emits"); panic!(); },
            }); 
        }

        // ////////////////////////////
        // Building servant processes

        // build a fusion engine with a maximum discernment frame size of 1024 
        let engine = DiscountedFuser::new(512..=1024);
        // get referee function copy in order to move it into future
        let referee = self.referee;
        // create the future related to the servant
        produce_future!(producer, { // code definition of the future
            let lattice = match match lattice_recv.recv().await { // receive the archived lattice definition
                Ok(l) => l,
                Err(_) => { eprintln!("Fuser:: failed to recv lattice"); panic!() },
            }.unarchive() { // and unarchive it
                // * unarchive is the most simple way but slighly costy because of the deserialization
                // * another way, generally more efficient but more complex, is to use zero copy reference to the archive content by means of data.archive_ref()
                Ok(u) => u,
                Err(_) => { eprintln!("Fuser:: failed to unarchive lattice"); panic!(); },
            };
            // display lattice hash 
            println!("Fuser::lattice_hash -> {}", lattice.lattice_hash());
            let len_i = readers_recv.len(); // get the number of readers connected to fuser
            let mut input = Vec::with_capacity(len_i); // prepare for receiving assignment sequence from the readers
            for reader in readers_recv { // for each reader
                let data = match reader.recv().await { // receive archive data 
                    Ok(d) => d,
                    Err(_) => { eprintln!("Fuser:: failed to recv reader"); panic!() },
                };
                // unarchive data and store it:
                // * unarchive is the most simple way but slighly costy because of the deserialization
                // * another way, generally more efficient but more complex, is to use zero copy reference to the archive content by means of data.archive_ref()
                input.push(match data.unarchive() {
                    Ok(ar) => ar,
                    Err(_) => { eprintln!("Fuser:: failed to unarchive"); panic!(); },                    
                });
            }
            // mux the data
            let len_i = input.len();
            let len_j = input[0].len();
            let mux: Vec::<Vec<_>> = (0..len_j).map(|j| (0..len_i).into_iter().map(|i| &input[i][j]).collect()).collect();
            // compute fused bbas sequence
            let fused = mux.iter().map(|bbas| match engine.fuse(&lattice, &referee, bbas) {
                Ok(a) => a, Err(_) => { eprintln!("Fuser:: failed to fuse"); panic!(); },
            }.0).collect::<Vec<_>>();
            // archive this result
            let arch_fused = match fused.arch_sized() {
                Ok(ad) => ad,
                Err(_) => { eprintln!("Fuser:: failed to arch sized"); panic!(); },                            
            };
            // and send it to writer
            match writer_send.send(arch_fused).await {
                Ok(_) => (),
                Err(_) => { eprintln!("Fuser:: failed to emit send"); panic!(); },                            
            }
            // It is now time to shutdown the networks
            sleep(Duration::from_millis(10)).await; // wait 10ms
            let arch_wake = match WakeSlx.arch_sized() { // create an archived waker to be sent to network shutdown
                Ok(aw) => aw,
                Err(_) => { eprintln!("Fuser:: failed to wake arch_sized"); panic!() },
            };
            for sender in shutdowns_send { // send waker to each network shutdown
                match sender.send(arch_wake.clone()).await { // in order to shutdown all networks
                    Ok(_) => (),
                    Err(_) => { eprintln!("Fuser:: failed to sender send"); panic!(); },                            
                }
            }
        })
    }
}
