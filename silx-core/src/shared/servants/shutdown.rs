use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::shared::{
    id_tools::IdBuilder,
    utils::{ 
        ServantBuilderParameters, ServantBuilder, SendToMaster, ProcessProducer, ProcessInstance, MsgFromServant, 
        produce_future, produce_read,
    },
    types::WakeSlx, 
};


#[derive(Serialize, Deserialize, Clone,)]
/// Structure defining Shutdown servant
/// * This servant await a `WakeSlx` message and then shut down the cluster
pub struct ShutdownBuilder {
    channel_shutdown:String,
} 

impl ShutdownBuilder {
    /// Shutdown servant builder
    /// `channel_shutdown: String` : channel name for awakening this servant
    /// Output: shutdown servant
    pub fn new(channel_shutdown: String,) -> Self { Self { channel_shutdown, } }

    /// Default shutdown servant builder
    /// Output: shutdown servant
    pub fn default_channels() -> Self { Self { channel_shutdown: format!("Shutdown"), } }
}

// This line is mandatory
#[typetag::serde] impl ServantBuilder for ShutdownBuilder { }

impl ServantBuilderParameters for ShutdownBuilder {
    // 0.1s delay between servant awaits before forcing the servant to stop
    fn max_cycle_time(&self) -> Duration { Duration::from_millis(100) }


    fn build_process(&self, task_id: IdBuilder, send_to_master: SendToMaster,) -> ProcessInstance { 
        // get a process producer ; producer will handle part of the channels and future definitions
        let mut producer = ProcessProducer::new(&send_to_master);
        // build channel receiver of type `WakeSlx`, of name `self.channel_shutdown` and capacity `1`
        let read_recv = produce_read!(producer,WakeSlx,self.channel_shutdown, Some(1)).unwrap();
        // this macro produce the future to be processed by the servant
        produce_future!(producer, { // code definition of the future 
            let _ = read_recv.recv().await; // Just wait for a message (Ok or Err imply both shutdown)
            let tid = task_id.lock().await.generate(); // generate a new task identifier to be sent to the master of the cluster
            match MsgFromServant::Shutdown(tid).send(&send_to_master).await { // send shutdown message to the master of the cluster
                Err(e)       => eprintln!("Shutdown error: failed to send message to master:\n -> {}",e), // print error if any
                Ok(receiver) => { let _ = receiver.await; }, // or receive master acknowledgment
            }  
        })
    }
}
