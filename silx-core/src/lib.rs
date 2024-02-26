//! This is part of [**Silx**](https://github.com/fdecode/silx-furtif) project  
//!
//! `silx-core` contains core components for implementing silx application  
//!
//! # Purpose
//! Silx aims to enable:
//! * build an application as a network of asynchronous servants on one or more machines
//! * build these servants without worrying about the detailed implementation of exchange channels between servants 
//! * connect these servants using simple-to-parameterize exchange channels
//! * control the coherence of the channel data types thanks to type hash codes
//! * implement serialization with zero-copy deserialization (rkyv) on the exchange channels
//! * serialize the application's entire network definition in editable text format, then reload and execute it  
//!
//! Silx remains a project under development.   
//!
//! To start with, the following example provides a minimalist overview.
//! Other examples are also available on the project's github.
//!
//! # Minimalist example (Hello)
//! ## Cargo.toml
//! ```toml
//! [package]
//! name = "silx_hello"
//! version = "0.1.1"
//! edition = "2021"
//!
//! [dependencies]
//! tokio = "^1.36.0"
//! serde = "^1.0.197"
//! typetag = "^0.2.15"
//!
//! silx-core = "^0.1.1"
//! silx-types = "^0.1.1"
//! ```
//! ## main.rs
//! ```text
//! use std::{ net::{IpAddr, Ipv4Addr, SocketAddr}, path::PathBuf, time::Duration };
//! use serde::{Deserialize, Serialize};
//! use tokio::{spawn, time::sleep};
//!
//! use silx_core::{ 
//!     id_tools::IdBuilder, servants::shutdown::ShutdownBuilder, 
//!     utils::{ 
//!         produce_emit, produce_future, produce_query, produce_read, produce_reply2, 
//!         Filable, MsgFromServant, ProcessInstance, ProcessProducer, SendToMaster, 
//!         ServantBuilder, ServantBuilderParameters, Starter, StarterProducer
//!     },
//! };
//! use silx_types::{ ArchSized, WakeSlx };
//!
//! // ///////////////////////////
//! // Servants implementations
//!
//! /// Servant replying greetings by completing queryied full name with Hello
//! #[derive(Serialize, Deserialize, Clone,)]
//! struct Hello(String);
//! #[typetag::serde] impl ServantBuilder for Hello { }
//! impl ServantBuilderParameters for Hello {
//!     fn max_cycle_time(&self) -> Duration { Duration::from_millis(100) }
//!     fn build_process(&self, _task_id: IdBuilder, send_to_master: SendToMaster,) -> ProcessInstance {
//!         let mut producer = ProcessProducer::new(&send_to_master);         
//!         let hello = self.0.clone();
//!         let query_channel = "QueryHello".to_string();
//!         // build reply process
//!         produce_reply2!([hello], producer, String => String, query_channel, data, {
//!             // get full name
//!             let full_name: &str = data.archive_ref().unwrap();
//!             // build an return greeting
//!             let greeting = format!("{hello} {full_name}");
//!             greeting.arch_sized().unwrap()
//!         }).unwrap();
//!         producer.named_process()
//!     }
//! }
//!
//! /// Servant sending first name 
//! #[derive(Serialize, Deserialize, Clone,)]
//! struct FirstName(String);
//! #[typetag::serde] impl ServantBuilder for FirstName { }
//! impl ServantBuilderParameters for FirstName {
//!     fn max_cycle_time(&self) -> Duration { Duration::from_millis(100) }
//!     fn build_process(&self, _task_id: IdBuilder, send_to_master: SendToMaster,) -> ProcessInstance {
//!         let mut producer = ProcessProducer::new(&send_to_master);
//!         let first_name = self.0.clone();
//!         // build channels
//!         let emit_channel = "FirstName".to_string();
//!         let sender = produce_emit!(producer, String, emit_channel, None,).unwrap();
//!         // build process
//!         produce_future!(producer, {
//!             sleep(Duration::from_millis(100)).await; // Wait a little bit for receiver to be ready
//!             sender.send(first_name.arch_sized().unwrap()).await.unwrap();
//!         })
//!     }
//! }
//!
//! /// Servant doing:
//! /// * receive first name
//! /// * build full name
//! /// * query for greeting
//! /// * print greeting
//! /// * shutdown
//! #[derive(Serialize, Deserialize, Clone,)]
//! struct LastName(String);
//! #[typetag::serde] impl ServantBuilder for LastName { }
//! impl ServantBuilderParameters for LastName {
//!     fn max_cycle_time(&self) -> Duration { Duration::from_millis(100) }
//!     fn build_process(&self, task_id: IdBuilder, send_to_master: SendToMaster,) -> ProcessInstance {
//!         let mut producer = ProcessProducer::new(&send_to_master);
//!         let last_name = self.0.clone();
//!         // build channels
//!         let recv_channel = "FirstName".to_string();
//!         let receiver = produce_read!(producer,String,recv_channel,None,).unwrap();
//!         let query_channel = "QueryHello".to_string();
//!         let (query_sender,reply_receiver) = produce_query!(producer,String => String,query_channel, None).unwrap();
//!         let emit_death = "Shutdown".to_string();
//!         let death_sender = produce_emit!(producer, WakeSlx, emit_death, None,).unwrap();
//!         // build process
//!         produce_future!(producer, {
//!             // receive first name
//!             let arc_first_name = receiver.recv().await.unwrap();
//!             // build full name
//!             let full_name = format!("{} {last_name}", arc_first_name.archive_ref().unwrap());
//!             // query for greeting
//!             let arc_full_name = full_name.arch_sized().unwrap();
//!             query_sender.send(arc_full_name).await.unwrap();
//!             let reply = reply_receiver.recv().await.unwrap();
//!             // print greeting
//!             println!("{}",reply.archive_ref().unwrap());
//!             // shutdown            
//!             death_sender.send(WakeSlx.arch_sized().unwrap()).await.unwrap();
//!             let tid = task_id.lock().await.generate();
//!             MsgFromServant::Shutdown(tid).send(&send_to_master).await.unwrap();
//!         })
//!     }
//! }
//!
//! // ///////////////////////////
//! // Network implementation
//!
//! /// Given main and slave socket addresses, build main and slave starters
//! /// * main cluster implements servants `last_name` and `hello`
//! /// * slave cluster implements servants `first_name` and `shutdown` (which will shutdown the slave)
//! /// * actions of `last_name`: 
//! ///   * receive first name from `first_name`
//! ///   * build full name and query greeting from `hello`
//! ///   * print greeting
//! ///   * send shutdown signal to `shutdown` and shutdown main cluster
//! /// * `main_addr: SocketAddr` : main socket address
//! /// * `slave_addr: SocketAddr` : slave socket address
//! /// * `save_dir: &PathBuf` : directory where to save the network
//! /// * Output: main and slave starters
//! pub fn build_network (main_addr: SocketAddr, slave_addr: SocketAddr, save_dir: &PathBuf) -> (Starter,Starter) {
//!     let max_ping = Duration::from_millis(100);
//!     // set two clusters within the network
//!     let start_prod = StarterProducer::new(
//!         main_addr, "starter=main.yaml", "builder=main.yaml", None, 16
//!     ).add_cluster(
//!         slave_addr, "starter=slave.yaml", "builder=slave.yaml", None, 16
//!     ).unwrap().done();
//!     // add named servants
//!     let start_prod = start_prod.add_process(
//!         &main_addr, "last_name".to_string(), "servant=last_name.yaml", LastName("Doe".to_string())
//!     ).unwrap().add_process(
//!         &main_addr, "hello".to_string(), "servant=hello.yaml", Hello("Welcome".to_string())
//!     ).unwrap().add_process(
//!         &slave_addr, "first_name".to_string(),"servant=first_name.yaml", FirstName("John".to_string())
//!     ).unwrap().add_process(
//!         &slave_addr, "shutdown".to_string(),"servant=shutdown.yaml", ShutdownBuilder::new("Shutdown".to_string())
//!     ).unwrap().done();
//!     // add channels connecting the servants and produce the starter for each cluster
//!     let mut starters = start_prod.add_query(
//!         "channel=QueryHello.yaml", "QueryHello".to_string(), main_addr, ["last_name".to_string()], ["hello".to_string()], max_ping, None
//!     ).unwrap().add_net_broadcast(
//!         "channel=FirstName.yaml", "FirstName".to_string(), slave_addr, [format!("first_name"),], main_addr, [format!("last_name"),], max_ping, 16
//!     ).unwrap().add_net_broadcast(
//!         "channel=Shutdown.yaml", "Shutdown".to_string(), main_addr, ["last_name".to_string()], slave_addr, ["shutdown".to_string()], max_ping, 16,
//!     ).unwrap().done();
//!     // save, get and return starters of the clusters
//!     let main_starter = starters.remove(&main_addr).unwrap().unload(Some(save_dir)).unwrap();
//!     let slave_starter = starters.remove(&slave_addr).unwrap().unload(Some(save_dir)).unwrap();
//!     (main_starter,slave_starter)
//! }
//!
//! // //////////////////////////
//! // Run the network
//!
//! /// Main performs:
//! /// * build network and save it in files
//! /// * network execution
//! /// * network loading from files
//! /// * execute the loaded network
//! #[tokio::main]
//! pub async fn main() {
//!     // build network and save it in files
//!     let main_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8180);
//!     let slave_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8181);
//!     let save_dir = PathBuf::from("./saved");
//!     let (main_starter,slave_starter) = build_network(main_addr, slave_addr, &save_dir);
//!     // network execution
//!     println!("==== first run -------------\n");
//!     let handle_slave = spawn(async move { 
//!         // NOTA: main starter should be launched FIRST
//!         sleep(Duration::from_millis(100)).await; // So wait a little bit
//!         slave_starter.run().await.unwrap();
//!     });
//!     main_starter.run().await.unwrap();
//!     handle_slave.await.unwrap();
//!     sleep(Duration::from_millis(300)).await;
//!     // network loading from files
//!     println!("\n==== second run (loadind network) -------------\n");
//!     let main_starter = Starter::load("starter=main.yaml", &save_dir).unwrap();
//!     let slave_starter = Starter::load("starter=slave.yaml", &save_dir).unwrap();
//!     // execute the loaded network
//!     let handle_slave = spawn(async move { 
//!         sleep(Duration::from_millis(100)).await;
//!         slave_starter.run().await.unwrap();
//!     });
//!     main_starter.run().await.unwrap();
//!     handle_slave.await.unwrap();
//! }
//! ```
//! ## Typical output
//! ```txt
//! ==== first run -------------
//!
//! 127.0.0.1:8181: try to connect 127.0.0.1:8180
//! 127.0.0.1:8181: Listening connection established
//! cluster 127.0.0.1:8181 has been built
//! cluster 127.0.0.1:8180 has been built
//! Welcome John Doe
//! cluster 127.0.0.1:8181 is ended
//! cluster 127.0.0.1:8180 is ended
//!
//! ==== second run (loadind network) -------------
//!
//! 127.0.0.1:8181: try to connect 127.0.0.1:8180
//! 127.0.0.1:8181: Listening connection established
//! cluster 127.0.0.1:8181 has been built
//! cluster 127.0.0.1:8180 has been built
//! Welcome John Doe
//! cluster 127.0.0.1:8180 is ended
//! cluster 127.0.0.1:8181 is ended
//! ```
//! ## Servant definition
//! Servants are built by implementing the `ServantBuilderParameters` trait and the `ServantBuilder` trait with the macro `#[typetag::serde]`.
//! The macro `#[typetag::serde]` is required to serialize the `ServantBuilder` implementers, and are therefore necessary to describe the network by means of configuration files (see below).
//! Below, we take a detailed look at the construction of the `LastName` and `Hello` servants: 
//! * `LastName` corresponds to the main type of servant, including incoming and outgoing channels and a processing code
//! * `Hello` is a reply-to-query servant, taking the form of a simple function
//! Servant construction is the only stage where a Rust implementation is strictly necessary. 
//! Otherwise, all that is required to build the computing network is the definition of configuration files.
//! ### Servant `LastName`
//! All servants must implement the `ServantBuilderParameters` trait and the `ServantBuilder` trait.
//! The `ServantBuilder` implementation is empty but mandatory.
//! Implementing `ServantBuilderParameters` requires defining the `max_cycle_time` and `build_process` methods.
//! The `max_cycle_time` method specifies the maximum time allowed to respond to a request from the cluster master.
//! After this time, the servant is considered inoperative and is killed, so this feature is of little importance:
//! ```txt
//! #[derive(Serialize, Deserialize, Clone,)]
//! struct LastName(String);
//! #[typetag::serde] impl ServantBuilder for LastName { }
//! impl ServantBuilderParameters for LastName {
//!     fn max_cycle_time(&self) -> Duration { Duration::from_millis(100) }
//!     fn build_process(&self, task_id: IdBuilder, send_to_master: SendToMaster,) -> ProcessInstance {
//!         [...]
//!     }
//! }
//! ```
//! In contrast, implementation of the `build_process` method concerns the essential aspects of the servant's functional behavior
//! #### Initializing the producer and retrieving servant data
//! Firstly, a new producer must be initialized with the send channel to the master, and secondly, the servant data can be cloned (this task is not necessary for copyable data).
//! The producer will be an essential helper in the construction of all servant components:
//! ```txt
//! fn build_process(&self, task_id: IdBuilder, send_to_master: SendToMaster,) -> ProcessInstance {
//!     let mut producer = ProcessProducer::new(&send_to_master);
//!     let last_name = self.0.clone();
//!     [...]
//! }
//! ```
//! #### Setting up channels connecting the servant
//! Channels are built by means of macros `produce_read`, `QueryHello` and `Shutdown`.
//! These macros are working on the producer:
//! ```txt
//! fn build_process(&self, task_id: IdBuilder, send_to_master: SendToMaster,) -> ProcessInstance {
//!     [...]
//!     let recv_channel = "FirstName".to_string();
//!     let receiver = produce_read!(producer,String,recv_channel,None,).unwrap();
//!     let query_channel = "QueryHello".to_string();
//!     let (query_sender,reply_receiver) = produce_query!(producer,String => String,query_channel, None).unwrap();
//!     let emit_death = "Shutdown".to_string();
//!     let death_sender = produce_emit!(producer, WakeSlx, emit_death, None,).unwrap();
//!     [...]
//! }
//! ```
//! In this code, we successively define connections to `FirstName`, `QueryHello` and `Shutdown` channels:
//! * Macro `produce_read` registers the servant as reader of channel `FirstName`.
//! Receiver `receiver` is generated to access the channel output
//! * Macro `produce_query` registers the servant as a queryer on channel `QueryHello`.
//! Sender `query_sender` and receiver `reply_receiver` are generated to send a query and receive a reply
//! * Macro `produce_emit` registers the servant as emitter on channel `Shutdown`.
//! Sender `death_sender` is generated to access the channel input
//! #### Building servant processes
//! The servant performs the following operations in succession:
//! * receives the first name and builds the full name
//! * request for greeting message and print greeting message
//! * shutdown  
//! The process is defined by means of macro:
//! ```txt
//! produce_future!(producer, { ... })
//! ```
//! ##### Receives the first name and builds the full name
//! The servant awaits a message from `FirstName` channel.
//! This message is archived and can be accessed as a reference (zero-copy deserialization) using the `archive_ref` method.
//! The full name is then constructed using the `format` macro:
//! ```txt
//! fn build_process(&self, task_id: IdBuilder, send_to_master: SendToMaster,) -> ProcessInstance {
//!     [...]
//!     produce_future!(producer, {
//!         let arc_first_name = receiver.recv().await.unwrap();
//!         let full_name = format!("{} {last_name}", arc_first_name.archive_ref().unwrap());
//!         [...]
//!     })
//! }
//! ```
//! **Note 1:** there are two methods for referencing from an archive, `archive_ref` and `arch_deref`.
//! The `archive_ref` method references the rkyv archive, while `arch_deref` offers greater flexibility.
//! However, `arch_deref` is less frequently implemented.  
//! **Note 2:** `archive_mut` and `arch_deref_mut` are the pinned mutable counterparts of `archive_ref` and `arch_deref`.
//! ##### Request for greeting message and print greeting message
//! The servant build an archive from the full name by means of method `arch_sized`, send it as a query, await for a reply, and print this reply:
//! ```txt
//! fn build_process(&self, task_id: IdBuilder, send_to_master: SendToMaster,) -> ProcessInstance {
//!     [...]
//!     produce_future!(producer, {
//!         [...]
//!         let arc_full_name = full_name.arch_sized().unwrap();
//!         query_sender.send(arc_full_name).await.unwrap();
//!         let reply = reply_receiver.recv().await.unwrap();
//!         println!("{}",reply.archive_ref().unwrap());
//!         [...]
//!     })
//! }
//! ```
//! ##### Shutdown
//! The servant shuts down the network by sending a wake-up message to the `shutdown` servant of the other cluster and sending a Shutdown task to its master:
//! ```txt
//! fn build_process(&self, task_id: IdBuilder, send_to_master: SendToMaster,) -> ProcessInstance {
//!     [...]
//!     produce_future!(producer, {
//!         [...]
//!         death_sender.send(WakeSlx.arch_sized().unwrap()).await.unwrap();
//!         let tid = task_id.lock().await.generate();
//!         MsgFromServant::Shutdown(tid).send(&send_to_master).await.unwrap();
//!     })
//! }
//! ```
//! ### Servant `Hello`
//! This servant is a replier, so the definition of `build_process` is different.
//! First at all, a new producer is initialized with the send channel to the master, the servant data are cloned and the query channel name is defined:
//! ```txt
//! [...]
//! impl ServantBuilderParameters for Hello {
//!     [...]
//!     fn build_process(&self, _task_id: IdBuilder, send_to_master: SendToMaster,) -> ProcessInstance {
//!         let mut producer = ProcessProducer::new(&send_to_master);         
//!         let hello = self.0.clone();
//!         let query_channel = "QueryHello".to_string();
//!         [...]
//!     }
//! }
//! ```
//! Then, the replying code is registered to the producer by means of macro:
//! ```txt
//! produce_reply2!([hello], producer, String => String, query_channel, data, { ... })
//! ```
//! * `[hello]` informs the macro that the non-copyable variable `hello` will be moved to the closure
//! * `String => String` informs that the query is of type `String` and the reply is of type `String`
//! * `query_channel` is the name of the query channel
//! * `data` is the name of the variable containing the query  
//!
//! In its process, the servant retrieves the reference to the full name from archive `data`, then prefixes it with the greeting message and finally returns an archive of the result: 
//! ```txt
//! [...]
//! impl ServantBuilderParameters for Hello {
//!     [...]
//!     fn build_process(&self, _task_id: IdBuilder, send_to_master: SendToMaster,) -> ProcessInstance {
//!         [...]
//!         produce_reply2!([hello], producer, String => String, query_channel, data, {
//!             let full_name: &str = data.archive_ref().unwrap();
//!             let greeting = format!("{hello} {full_name}");
//!             greeting.arch_sized().unwrap()
//!         }).unwrap();
//!         [...]
//!     }
//! }
//! ```
//! At last, the process instance is recovered from `producer` and returned:
//! ```txt
//! [...]
//! impl ServantBuilderParameters for Hello {
//!     [...]
//!     fn build_process(&self, _task_id: IdBuilder, send_to_master: SendToMaster,) -> ProcessInstance {
//!         [...]
//!         producer.named_process()
//!     }
//! }
//! ```
//! ## Network definition
//! The network can be built using the `StarterProducer` and its derivatives.
//! Another way is to edit configuration files, which are used to build the network's cluster starters by deserialization.
//! These configuration files can be generated automatically using `StarterProducer` as shown in the `build_network` method in the example.
//! The example proceeds as follows:
//!   * initialize a producer with the characteristics of the main and slave clusters. 
//!   It emerges that the serialization file for each starter and each builder (one per cluster of the two) is supplied.
//!   The clusters are also identified by their socket addresses:
//! ```txt
//! let start_prod = StarterProducer::new(main_addr, "starter=main.yaml", "builder=main.yaml", None, 16)
//!     .add_cluster(slave_addr, "starter=slave.yaml", "builder=slave.yaml", None, 16).unwrap().done()
//! ```
//!   * add servants to the clusters. Here servants `last_name` and `hello` are added to main cluster while servants `first_name`  and `shutdown` are added to slave cluster. The name, serialization file and value of each servant is supplied:
//! ```txt
//! let start_prod = start_prod
//!     .add_process(&main_addr, "last_name".to_string(), "servant=last_name.yaml", LastName("Doe".to_string())).unwrap()
//!     .add_process(&main_addr, "hello".to_string(), "servant=hello.yaml", Hello("Welcome".to_string())).unwrap()
//!     .add_process(&slave_addr, "first_name".to_string(),"servant=first_name.yaml", FirstName("John".to_string())).unwrap()
//!     .add_process(&slave_addr, "shutdown".to_string(),"servant=shutdown.yaml", ShutdownBuilder::new("Shutdown".to_string())).unwrap().done();
//! ```
//!   * add the channels to the clusters and retrieve the serializable starters. 
//!   The serialization file, name and input servants followed by output servants are provided.
//!   Indeed, the channels may connect several servants to several servants.
//!   Moreover, the cluster address is provided in case of channels within a cluster, and the input cluster address followed by output cluster address are provided in case of channels betweens two clusters.
//!   The nature of the channel is determined by the used method, here `add_net_broadcast` and `add_query`:
//! ```txt
//! let mut starters = start_prod.add_query(
//!     "channel=QueryHello.yaml", "QueryHello".to_string(), main_addr, ["last_name".to_string()], ["hello".to_string()], max_ping, None
//! ).unwrap().add_net_broadcast(
//!     "channel=FirstName.yaml", "FirstName".to_string(), slave_addr, [format!("first_name"),], main_addr, [format!("last_name"),], max_ping, 16
//! ).unwrap().add_net_broadcast(
//!     "channel=Shutdown.yaml", "Shutdown".to_string(), main_addr, ["last_name".to_string()], slave_addr, ["shutdown".to_string()], max_ping, 16,
//! ).unwrap().done();
//! ```
//!   * At this stage, the starters are serializable, but not executable. We can generate serialized files and retrieve the executable starter for a cluster using the `unload` command. 
//!   At this stage, we have serializable, but not executable, starters. 
//!   We can generate serialized files and retrieve the executable starter for a cluster using the `unload` command:
//! ```txt
//! let main_starter = starters.remove(&main_addr).unwrap().unload(Some(save_dir)).unwrap();
//! let slave_starter = starters.remove(&slave_addr).unwrap().unload(Some(save_dir)).unwrap();
//! (main_starter,slave_starter)
//! ```  
//! Note that the `unload` command can be used without producing any serialization, by supplying `None` as the serialization directory; you can also use the `unwrap` command, which achieves the same result.
//! ## Cluster loading and execution
//! A starter can be loaded using the `load` method, which is a shortcut to a sequence of more elementary commands.
//! Executing a starter is simply done using the `run` method.
//! ```txt
//!     let save_dir = PathBuf::from("./saved");
//!     [...]
//!     let main_starter = Starter::load("starter=main.yaml", &save_dir).unwrap();
//!     main_starter.run().await.unwrap();
//! ```
//! ## Saved files from the network serialization
//! After a run, 11 files are generated from the network serialization in directory `saved` of the project.
//! ```txt
//! │   Cargo.toml
//! │
//! ├───saved
//! │   ├───main
//! │   │       builder=main.yaml
//! │   │       builder=slave.yaml
//! │   │       channel=FirstName.yaml
//! │   │       channel=QueryHello.yaml
//! │   │       channel=Shutdown.yaml
//! │   │       servant=first_name.yaml
//! │   │       servant=hello.yaml
//! │   │       servant=last_name.yaml
//! │   │       servant=shutdown.yaml
//! │   │       starter=main.yaml
//! │   │
//! │   └───slave
//! │           starter=slave.yaml
//! │
//! └───src
//!         main.rs
//! ```
//! Directory `saved/main` contains the full definition of the main starter, while directory `saved/slave` contains the full definition of the slave starter.  
//! **An important point is that all aspects of the network architecture are parameterized by these editable files.**
//! The only thing that cannot be parameterized and needs to be implemented in **Rust** is the definition of servants, by implementing the traits `ServantBuilder` and `ServantBuilderParameters`.
//! ### Slave starter saved file
//! Directory `saved/slave` contains the only file, `starter=slave.yaml`.
//! #### Slave starter file `starter=slave.yaml`
//! ```yaml
//! !Listener
//! main: 127.0.0.1:8180
//! this: 127.0.0.1:8181
//! ```
//! The file explains it all: 
//! * slave is a `!Listener`
//! * its socket address is `this: 127.0.0.1:8181`
//! * it waits for all its directives and definitions from the main socket address `main: 127.0.0.1:8180`
//! ### Main starter saved files
//! Directory `saved/main` contains all the other files, including , `builder=slave.yaml`. 
//! #### Main starter file `starter=main.yaml`
//! ```yaml
//! !Main
//! builders:
//!   127.0.0.1:8180: !unloaded
//!     path: builder=main.yaml
//!   127.0.0.1:8181: !unloaded
//!     path: builder=slave.yaml
//! flow:
//!   FirstName: !unloaded
//!     path: channel=FirstName.yaml
//!   QueryHello: !unloaded
//!     path: channel=QueryHello.yaml
//!   Shutdown: !unloaded
//!     path: channel=Shutdown.yaml
//! main: 127.0.0.1:8180
//! ```
//! The file contains all the structure of the network: 
//! * main is a `!Main`
//! * its socket address is `main: 127.0.0.1:8180`
//! * it controls two clusters, including itself, whose builders are listed after  `builders:`
//!   * Cluster at address `127.0.0.1:8180` is defined within file `builder=main.yaml`
//!   * Cluster at address `127.0.0.1:8181` is defined within file `builder=slave.yaml`
//! * it holds the definition of all channels, which are listed after `flow:`
//!   * Channels `FirstName`, `QueryHello` and `Shutdown` are defined respectively within `channel=FirstName.yaml`, `channel=QueryHello.yaml` and `channel=Shutdown.yaml`  
//! ### Builders
//! #### Builder file `builder=main.yaml`
//! ```yaml
//! net_size: null
//! named_servants:
//!   hello: !unloaded
//!     path: servant=hello.yaml
//!   last_name: !unloaded
//!     path: servant=last_name.yaml
//! ctrl_ch_capacity: 16
//! ```
//! This file informs that main cluster contains the servants `hello` and `last_name` which are respectively defined within files `servant=hello.yaml` and `servant=last_name.yaml`
//! #### Builder file `builder=slave.yaml`
//! ```yaml
//! net_size: null
//! named_servants:
//!   first_name: !unloaded
//!     path: servant=first_name.yaml
//!   shutdown: !unloaded
//!     path: servant=shutdown.yaml
//! ctrl_ch_capacity: 16
//! ```
//! This file informs that slave cluster contains the servants `first_name` and `shutdown` which are respectively defined within files `servant=first_name.yaml` and `servant=shutdown.yaml`
//! ### Servants and Channels files
//! The Servant file is inherited from the type definition directly by serialization:
//! #### Servant file `servant=hello.yaml`
//! ```yaml
//! servant: Hello
//! value: Welcome
//! ```
//! Channels are serialized in the same way, but contain channel type, cluster addresses, data type hash codes and input/output servants lists:
//! #### Channel file `channel=QueryHello.yaml`
//! ```yaml
//! !Query
//! cluster: 127.0.0.1:8180
//! max_ping:
//!   secs: 0
//!   nanos: 100000000
//! query_type: 31758449-bc37-9d2d-7a6d-5463554081ac
//! reply_type: 31758449-bc37-9d2d-7a6d-5463554081ac
//! size: null
//! input:
//! - last_name
//! output:
//! - hello
//! ```
//! #### Channel file `channel=FirstName.yaml`
//! ```yaml
//! !NetBroadcast
//! max_ping:
//!   secs: 0
//!   nanos: 100000000
//! data_type: 31758449-bc37-9d2d-7a6d-5463554081ac
//! size: 16
//! input:
//! - 127.0.0.1:8181
//! - - first_name
//! output:
//! - 127.0.0.1:8180
//! - - last_name
//! ```


/// Shared structures, traits and macros; reexport
mod shared;
pub use self::shared::{ types, servants, utils, channels, id_tools, };

/// silx structures
pub (crate) mod structs;
/// silx traits
pub (crate) mod traits;
/// silx builder
pub (crate) mod builder;
/// silx network
pub (crate) mod net;


const NALLOC: usize = 5;

type QueryIdType = u64;
type ChannelIdType = u64;
type ServantIdType = u32; 



#[doc(hidden)]
/// Probes for testing features activation
pub mod probes;