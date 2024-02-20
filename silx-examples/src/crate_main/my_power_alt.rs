// ======= IMPORTS

use std::time::Duration;

use tokio::time::sleep;
use serde::{Deserialize, Serialize};

use silx_core::{ id_tools::IdBuilder, utils::{ 
    ServantBuilderParameters, ServantBuilder, SendToMaster, ProcessProducer, ProcessInstance, ArchSized,
    produce_reply2,
}};
use silx_types::{ f64slx, ArchToDeref, Float, };



// ======= A SERVANT DEFINITION

#[derive(Serialize, Deserialize, Clone,)]
/// Alternative structure defining the servant that raises the scalar to power
pub struct MyPowerBuilderAlt {
    /// exponent of the power
    exponent: i32,
    /// Name of the channel for replying to power computation query
    channel_power: String,
    /// Ident of the servant (only for display)
    id: u32,
}

impl MyPowerBuilderAlt {
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
#[typetag::serde] impl ServantBuilder for MyPowerBuilderAlt { }

impl ServantBuilderParameters for MyPowerBuilderAlt {
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

        // this macro produce future for a servant replying to query (second kind)
        // Second kind means that the future process take an archived data reference and return an archived result
        // Parameters are:
        // * the mapping type: `f64slx => f64slx`
        //    -> take an archived `f64slx` and produce an archived `f64slx`
        // * the channel name of the query process: `self.channel_power`
        //    -> the macro produce automatically the code for receiving and sending the data
        // * the name of the data to be processed by the future: `data`
        produce_reply2!(producer, f64slx => f64slx, self.channel_power,data, {
            // get a reference to the data
            let f: &f64slx = data.arch_deref().expect("failed to get & archive");
            // sleep 2s
            sleep(Duration::from_millis(2000)).await;
            // print the data
            print!("power {}: {}^{} -> ",id, f,n);
            // raise the data to power
            let f = (*f).powi(n); // the result is not an archive!
            // print the result 
            println!("{}",f);
            // produce an archived result
            f.arch_sized().expect("failed to serialize")
        }).unwrap(); // the macro has no output except error repport
        // this last command then produces the process instance:
        producer.named_process()
    }
}