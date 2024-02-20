// ======= IMPORTS
use std::{pin::Pin, time::Duration};

use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use nalgebra::SVector;

use silx_core::{ id_tools::IdBuilder, utils::{ 
    ServantBuilderParameters, ServantBuilder, SendToMaster, ProcessProducer, ProcessInstance, ArchSized,
    produce_data_future, produce_query, produce_emit,
}};
use silx_types::{ f64slx, nalgebra::{ArrayStorageSlx, DerefMatrixSlx}, ArchToDerefMut, IntoSlx, WakeSlx };

// ======= A SERVANT DEFINITION

#[derive(Serialize, Deserialize, Clone,)]
/// Structure defining the servant which enumerate the vector 
pub struct MyLooperBuilderVec { 
    /// Number of loops
    pub loops: u32,
    /// Name of the channel for querying power computation
    pub channel_power: String,
    /// Name of the channel where to broadcast values raised to power
    pub channel_adder: String,
    /// Names of the channels where to shutdown signal
    pub channels_shutdown: Vec<String>,
}
impl MyLooperBuilderVec {
    #[allow(dead_code)]
    /// full options constructor of the looper builder
    /// * `loops: u32` : number of loops 
    /// * `channel_power: String` : name of the channel for querying power computation
    /// * `channel_adder: String` : name of the channel where to broadcast values raised to power
    /// * `channels_shutdown: Vec<String>` : names of the channels where to shutdown signal
    /// * Output: the builder
    pub fn new(loops: u32, channel_power: String, channel_adder: String, channels_shutdown: Vec<String>,) -> Self { 
        Self { loops, channel_power, channel_adder, channels_shutdown, } 
    }

    #[allow(dead_code)]
    /// default constructor of the looper builder
    /// * `loops: u32` : number of loops 
    /// * `nb_shutdown: usize` : number of shutdown channel
    /// * Output: the builder
    ///    * Channels are named: Shutdown_1,  Shutdown_2, ...  Shutdown_N
    pub fn default_channels(loops: u32, nb_shutdown: usize) -> Self { 
        let channels_shutdown = match nb_shutdown {
            0 => Vec::new(),
            1 => vec![format!("Shutdown")],
            n => (0..n).into_iter().map(|i| format!("Shutdown_{}",i+1)).collect(),
        };
        Self { 
            loops, channel_power: format!("Power"), channel_adder: format!("Adder"), channels_shutdown,
        } 
    }
}

// This line is mandatory
#[typetag::serde] impl ServantBuilder for MyLooperBuilderVec { }

// 10s delay between servant awaits before forcing the servant to stop (this is much longer than necessary)
impl ServantBuilderParameters for MyLooperBuilderVec {
    // 0.1s delay between servant awaits before forcing the servant to stop
    fn max_cycle_time(&self) -> Duration { Duration::from_millis(100) }

    fn build_process(&self, _task_id: IdBuilder, send_to_master: SendToMaster,) -> ProcessInstance { 
        // this type is the slx translation of SVector<f64slx,3>
        type ArrSlx = ArrayStorageSlx<f64slx,3,1>;
        //  get the number of loops (to be moved to the future)
        let n =  self.loops;

        // get a process producer ; producer will handle part of the channels and future definitions
        let mut producer = ProcessProducer::new(&send_to_master);

        // ////////////////////////////
        // Building servant channels

        // Get the channels (send the query, receive the reply) for querying the computation of raising data to the power
        // query type is ArrSlx => ArrSlx (query is ArrSlx and reply is ArrSlx), name is self.channel_power and capacity is n
        let (in_query_send, out_query_recv) = produce_query!(producer,ArrSlx => ArrSlx,self.channel_power, Some(n as usize)).unwrap();
        //Get channel sender of type ArrSlx, of name self.channel_adder and capacity 1
        let in_emit_send = produce_emit!(producer,ArrSlx,self.channel_adder, Some(1)).unwrap();
        //Get channels senders of type WakeSlx, of names within self.channels_shutdow and capacity 1
        let mut shutdowns_emit_send = Vec::new();
        for sn in self.channels_shutdown.iter() { 
            shutdowns_emit_send.push(produce_emit!(producer,WakeSlx,sn, Some(1),).unwrap()); 
        }

        // ////////////////////////////
        // Building servant processes
    
        //this macro produce the future to be processed by the servant together with some initialization data (which are fake here)
        produce_data_future!(producer, "THIS IS FAKE DATA".to_string(), { // code definition of the future: 
            // precompute some data (to be moved to the future)
            let mut query_vec_slx = Vec::new(); // mutable variable for storing the list of queries for display (a list of row vectors)
            let zero_slx: ArrayStorageSlx<_,3,1> = SVector::from([0f64.slx();3]).slx();
            let step = SVector::from([1f64.slx(),2f64.slx(),3f64.slx()]);
            // in particular, some archives containers may be precomputed 
            let arch_wake = WakeSlx.arch_sized().expect("failed to serialize"); // archive to be send for shutdown
            let mut query_arch_vec = zero_slx.arch_sized().expect("failed to serialize vec"); // archive of the current query

            for i in 0..n { // iterate the loop 
                query_vec_slx.push({ // push a new query to the query vector for this iteration:
                    // access to a mutable reference of the current query archive
                    let mut write: Pin<DerefMatrixSlx<_,_,_,_>> = query_arch_vec.pinned().arch_deref_mut().expect("failed to get &mut archive"); 
                    // increment the query value and transpose it
                    *write += step; 
                    // transpose this value before pushing it
                    write.transpose()
                });
                // send the current query
                in_query_send.send(query_arch_vec.clone()).await.expect("failed to send in query bytes");
                // and wait 1s at start
                if i == 0 { sleep(Duration::from_millis(1000)).await; }
            }
            for i in 0..n { // iterate the loop again
                // print the looper step and the value of its query
                println!("---------------\nlooper: {} -> {}",i,query_vec_slx[i as usize]);
                // wait for the reply to be sent back
                let reply_arch_vec = out_query_recv.recv().await.expect("failed to receive replied bytes");
                // send the result (actually to the adder)
                in_emit_send.send(reply_arch_vec).await.expect("failed to send reply");
            }
            // wait a while before shutting down
            sleep(Duration::from_millis(100)).await;
            // send shutdown message to all clusters
            for sender in shutdowns_emit_send {
                sender.send(arch_wake.clone()).await.expect("failed to send waking message");
            }
        }) // a process instance is obtained at output of the macro
    }
}
