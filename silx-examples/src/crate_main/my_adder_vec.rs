// ======= IMPORTS
use std::time::Duration;

use serde::{Deserialize, Serialize};
use nalgebra::SVector;

use silx_core::{ id_tools::IdBuilder, utils::{ 
    ServantBuilderParameters, ServantBuilder, SendToMaster, ProcessProducer, ProcessInstance,
    produce_future, produce_read,
}};
use silx_types::{ f64slx, nalgebra::{ArrayStorageSlx, DerefMatrixSlx}, ArchToDeref, IntoSlx };


// ======= A SERVANT DEFINITION

#[derive(Serialize, Deserialize, Clone,)]
/// Structure defining the servant which add the vectors 
pub struct MyAdderBuilderVec {
    /// Name of the channel where to values raised to power are broadcasted
    pub channel_adder: String,
}

impl MyAdderBuilderVec {
    #[allow(dead_code)]
    /// full options constructor of the adder builder
    /// * `channel_adder: String` : name of the channel where to values raised to power are broadcasted
    /// * Output: the builder
    pub fn new(channel_adder: String,) -> Self { Self { channel_adder, } }

    #[allow(dead_code)]
    /// default constructor of the adder builder
    pub fn default_channels() -> Self { Self { channel_adder: format!("Adder"), } }
}

// This line is mandatory
#[typetag::serde] impl ServantBuilder for MyAdderBuilderVec { }

impl ServantBuilderParameters for MyAdderBuilderVec {
    // 0.1s delay between servant awaits before forcing the servant to stop
    fn max_cycle_time(&self) -> Duration { Duration::from_millis(100) }

    fn build_process(&self, _task_id: IdBuilder, send_to_master: SendToMaster,) -> ProcessInstance {
        // this type is the slx translation of SVector<f64slx,3>
        type ArrSlx = ArrayStorageSlx<f64slx,3,1>; 
        // get a process producer ; producer will handle part of the channels and future definitions
        let mut producer = ProcessProducer::new(&send_to_master);

        // ////////////////////////////
        // Building servant channels

        // build channel receiver of type `ArrSlx`, of name `self.channel_adder` and capacity `1`
        let read_recv = produce_read!(producer, ArrSlx, self.channel_adder, Some(1),).unwrap();

        // ////////////////////////////
        // Building servant processes
    
        // initialise the sum (to be moved to the future)
        let mut sum = SVector::from([0f64.slx();3]);

        // this macro produce the future to be processed by the servant
        produce_future!(producer, { // code definition of the future 
            let mut alive = true;
            while alive { // while the channel is alive
                match read_recv.recv().await { // receive data
                    Err(_) => alive = false, // error: the channel is closed
                    Ok(data) => {
                        // get a reference to the data; type `DerefMatrixSlx` will do that for nalgebra matrices 
                        let read_vec: DerefMatrixSlx<_,_,_,_> = data.arch_deref().expect("failed to get & archive");
                        // add the data to the sum
                        sum += &*read_vec;
                        // print the sum
                        println!("adder: {} -> {}",read_vec.transpose().to_string(),sum.transpose().to_string());        
                    }
                }
            }
        }) // a process instance is obtained at output of the macro
    }
}