// ======= IMPORTS

use std::{pin::Pin, time::Duration};

use tokio::time::sleep;
use serde::{Deserialize, Serialize};

use silx_core::{ id_tools::IdBuilder, utils::{ 
    ServantBuilderParameters, ServantBuilder, SendToMaster, ProcessProducer, ProcessInstance,
    produce_reply1,
}};
use silx_types::{ f64slx, ArchToDerefMut, Float, };


// ======= A SERVANT DEFINITION

#[derive(Serialize, Deserialize, Clone,)]
/// Structure defining the servant that raises the scalar to power
pub struct MyPowerBuilder {
    /// Exponent of the power
    exponent: i32,
    /// Name of the channel for replying to power computation query
    channel_power: String,
    /// Ident of the servant (only for display)
    id: u32,
}

impl MyPowerBuilder {
    #[allow(dead_code)]
    /// full options constructor of the power builder
    /// * `exponent: i32` : exponent of the power 
    /// * `channel_power: String` : name of the channel for replying to power computation query
    /// * `id: u32` : ident of the servant (only for display)
    /// * Output: the builder
    pub fn new(exponent: i32, channel_power: String, id: u32,) -> Self { Self { exponent, channel_power, id, } }

    #[allow(dead_code)]
    /// default constructor of the power builder
    /// * `exponent: i32` : exponent of the power 
    /// * `id: u32` : ident of the servant (only for display)
    /// * Output: the builder
    pub fn default_channels(exponent: i32, id: u32,) -> Self { Self { exponent, channel_power: format!("Power"), id, } }
}

// This line is mandatory
#[typetag::serde] impl ServantBuilder for MyPowerBuilder { }

impl ServantBuilderParameters for MyPowerBuilder {
    // 0.1s delay between servant awaits before forcing the servant to stop
    fn max_cycle_time(&self) -> Duration { Duration::from_millis(100) }

    fn build_process(&self, _task_id: IdBuilder, send_to_master: SendToMaster,) -> ProcessInstance {
        // get a process producer ; producer will handle part of the channels and future definitions
        let mut producer = ProcessProducer::new(&send_to_master);

        // for servants replying to query, there is no need to get a query channel

        // ////////////////////////////
        // Building servant processes
    
        // get processing parameters (to be moved to the future)
        let n = self.exponent; let id = self.id;

        // this macro produce future for a servant replying to query (first kind)
        // First kind means that the future process on an archive pinned mutable reference and return nothing
        // This kind of future is particularly interesting if the queryer is in the same cluster
        // Parameters are:
        // * the archived type: `f64slx`
        // * the channel name of the query process: `self.channel_power`
        //    -> the macro produce automatically the code for receiving and sending the data
        // * the name of the data to be processed by the future: `data`
        produce_reply1!(producer, f64slx, self.channel_power, data, {
            // get a pinned mutable reference to the data
            let mut f: Pin<&mut f64slx> = data.arch_deref_mut().expect("failed to get &mut archive");
            // sleep 2s
            sleep(Duration::from_millis(2000)).await;
            // print the data
            print!("power {}: {}^{} -> ",id, f,n);
            // raise the data to power
            *f = (*f).powi(n);
            // print the result 
            println!("{}",f);
        } ).unwrap(); // the macro has no output except error repport
        // this last command then produces the process instance:
        producer.named_process()
    }
}