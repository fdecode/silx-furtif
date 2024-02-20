// ======= IMPORTS

use std::time::Duration;

use tokio::time::sleep;
use serde::{Deserialize, Serialize};

use silx_core::{ id_tools::IdBuilder, utils::{ 
    ServantBuilderParameters, ServantBuilder, SendToMaster, ProcessProducer, ProcessInstance, ArchSized,
    produce_reply2,
}};

use silx_types::{ f64slx, nalgebra::{ArrayStorageSlx, DerefMatrixSlx}, ArchToDeref, Float, IntoSlx };


// ======= A SERVANT DEFINITION

#[derive(Serialize, Deserialize, Clone,)]
/// Alternative structure defining the servant that raises the vector to power element by element
pub struct MyPowerBuilderAltVec {
    /// exponent of the power
    pub exponent: i32,
    /// Name of the channel for replying to power computation query
    pub channel_power: String,
    /// Ident of the servant (only for display)
    pub id: u32,
}

impl MyPowerBuilderAltVec {
    #[allow(dead_code)]
    /// full options constructor of the power builder
    /// * `exponent: i32` : exponent of the power 
    /// * `channel_power: String` : name of the channel for replying to power computation query
    /// * `id: u32` : ident of the servant (only for display)
    /// * Output: the builder
    pub fn new(exponent: i32, channel_power: String, id: u32,) -> Self { 
        Self { exponent, channel_power, id, } 
    }

    #[allow(dead_code)]
    /// default constructor of the power builder
    /// * `exponent: i32` : exponent of the power 
    /// * `id: u32` : ident of the servant (only for display)
    /// * Output: the builder
    pub fn default_channels(exponent: i32, id: u32,) -> Self { 
        Self { exponent, channel_power: format!("Power"), id, } 
    }
}

// This line is mandatory
#[typetag::serde] impl ServantBuilder for MyPowerBuilderAltVec { }

impl ServantBuilderParameters for MyPowerBuilderAltVec {
    // 0.1s delay between servant awaits before forcing the servant to stop
    fn max_cycle_time(&self) -> Duration { Duration::from_millis(100) }

    fn build_process(&self, _task_id: IdBuilder, send_to_master: SendToMaster,) -> ProcessInstance {
        type ArrSlx = ArrayStorageSlx<f64slx,3,1>;
        // get a process producer ; producer will handle part of the channels and future definitions
        let mut producer = ProcessProducer::new(&send_to_master);

        // for servants replying to query, there is no need to get a query channel

        // ////////////////////////////
        // Building servant processes
    
        // get processing parameters (to be moved to the future)
        let n = self.exponent; let id = self.id;

        // this macro produce future for a servant replying to query (second kind)
        // Second kind means that the future process take an archived data reference and return an archived result
        // Parameters are:
        // * the mapping type: `ArrSlx => ArrSlx`
        //    -> take an archived `ArrSlx` and produce an archived `ArrSlx`
        // * the channel name of the query process: `self.channel_power`
        //    -> the macro produce automatically the code for receiving and sending the data
        // * the name of the data to be processed by the future: `data`
        produce_reply2!(producer, ArrSlx => ArrSlx, self.channel_power,data, {
            // get a reference to the data
            let v: DerefMatrixSlx<_,_,_,_> = data.arch_deref().expect("failed to get & archive");
            // sleep 2s
            sleep(Duration::from_millis(2000)).await;
            // print the data
            print!("power {}: {}^{} -> ",id, v.transpose(),n);
            // raise the data to power (nalgebra vector is obtained)
            let v = v.map(|x| x.powi(n)); // the result is not an archive nor a silx data!
            // print the result 
            println!("{}",v.transpose());
            // get the silx translation of the nalgebra vector
            let vslx: ArrSlx = v.slx();
            // produce an archived result
            vslx.arch_sized().expect("failed to serialize")
        }).unwrap(); // the macro has no output except error repport
        // this last command then produces the process instance:
        producer.named_process()
    }
}