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

use furtif_core::{structs::EnumLattice, traits::Lattice};
use silx_core::{ id_tools::IdBuilder, utils::{ 
    ServantBuilderParameters, ServantBuilder, SendToMaster, ProcessProducer, ProcessInstance,
    produce_future, produce_emit,
}};
use silx_types::ArchSized;

// ======= A SERVANT DEFINITION

/// Servant builder for lattice definition (DSmT book example)
#[derive(Serialize, Deserialize, Clone,)]
pub struct DsmtbookLatticeBuilder {
    channels_lattice: Vec<String>,
    lattice: EnumLattice,
}

impl DsmtbookLatticeBuilder {
    #[allow(dead_code)]
    /// Constructor for DsmtbookLatticeBuilder (DSmT book example)
    /// * `channels_lattice: Vec<String>` : channels where to send the lattice definition
    /// * `lattice: EnumLattice` : instance of the lattice
    /// * Output : builder
    pub fn new(
        channels_lattice: Vec<String>, 
        lattice: EnumLattice,
    ) -> Self { Self { channels_lattice, lattice, } }
}


// This line is mandatory
#[typetag::serde] impl ServantBuilder for DsmtbookLatticeBuilder { }

impl ServantBuilderParameters for DsmtbookLatticeBuilder {
    // 10s delay between servant awaits before forcing the servant to stop (this is much longer than necessary)
    fn max_cycle_time(&self) -> Duration { Duration::from_millis(10000) }

    fn build_process(&self, _task_id: IdBuilder, send_to_master: SendToMaster,) -> ProcessInstance {
        // get a process producer ; producer will handle part of the channels and future definitions
        let mut producer = ProcessProducer::new(&send_to_master);

        // ////////////////////////////
        // Building servant channels

        // build channel senders of type `EnumLattice`, of name within `self.channels_lattice` and capacity `1`
        let in_emit_send = self.channels_lattice.iter().map(|channel| match produce_emit!(
            producer, EnumLattice, channel, Some(1),
        ) {
            Ok(es) => es,
            Err(_) => { eprintln!("Lattice:: failed to produce emit"); panic!(); },
        }).collect::<Vec<_>>();

        // ////////////////////////////
        // Building servant processes

        // clone lattice parameter in order to move it into future
        let lattice = self.lattice.clone();
        produce_future!(producer, { // code definition of the future
            // display lattice hash 
            println!("Lattice::lattice_hash -> {}", lattice.lattice_hash());
            // archive lattice in order to send it
            let arch_lattice = match lattice.arch_sized() {
                Ok(ad) => ad,
                Err(_) => { eprintln!("Reader:: failed to arch_sized"); panic!(); },
            };
            for em in in_emit_send { // for each channel to lattice consumers,
                match em.send(arch_lattice.clone()).await{ // send a clone of the archived lattice
                    Ok(_) => (),
                    Err(_) => { eprintln!("Reader:: failed to emit_send"); panic!(); },
                };    
            }
        })
    }
}