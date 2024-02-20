use crate::{
    shared::id_tools::{TaskId, IdBuilder}, structs::{ 
        archmod::archannel::ChannelServer,
        cells::ctrl_message::{ 
            CtrlCell, MsgFromMaster, MsgFromServant, RecvFromServant, ReplyToMaster, ReplyToServant, SendToServant, SetChannel as sch 
        },
    }, ChannelIdType, ServantIdType
};

use std::{ sync::Arc, collections::VecDeque, fmt::Debug, fmt::{ Formatter, Error, }, net::SocketAddr, };

use tokio::{ spawn, sync::RwLock, };
use fnv::FnvHashMap;

const VERBOSE: bool = false;

/// doc to be defined
pub struct Master {
    pub (crate) cluster_id: SocketAddr,
    #[allow(dead_code)]
    server: ChannelServer,
    send_to_servants: FnvHashMap<ServantIdType, SendToServant>, 
    recv_from_servants: FnvHashMap<ServantIdType, RecvFromServant>, 
    channels: FnvHashMap<ServantIdType, Vec<(ChannelIdType, sch)>>,
    full_alive: Arc<RwLock<bool>>, // false during shutdown process; useful for muting errors then
    alive: Arc<RwLock<bool>>,
    task_id: IdBuilder,
}

impl Debug for Master {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Servant {{ .. }}")
    }
}
impl Master {
    // Nota: builders are responsible of building ctrl channels, not master
    pub  (crate) fn new (cluster_id: SocketAddr,
            server: ChannelServer, // server is stored within master, so as to be active until cluster is dropped
            send_to_servants: FnvHashMap<ServantIdType, SendToServant>, recv_from_servants: FnvHashMap<ServantIdType, RecvFromServant>, 
            channels: FnvHashMap<ServantIdType, Vec<(ChannelIdType, sch)>>, task_id: IdBuilder,) -> Self {
        Self { 
            cluster_id,
            server,
            send_to_servants,
            recv_from_servants,
            channels,
            full_alive: Arc::new(RwLock::new(true)),
            alive: Arc::new(RwLock::new(true)),
            task_id,
        }
    }

    pub (crate) async fn run(self, ) {
        use MsgFromMaster::Ctrl as Ctrl;
        use MsgFromServant::{ FailureChl, StaledChl, Shutdown, };
        let Self { cluster_id, send_to_servants, recv_from_servants, channels, alive, full_alive, task_id, .. } = self; 
        let nb_channels = channels.iter().map(|(_,sc)| sc.len()).sum::<usize>();
        //
        // INITIALIZATION
        //
        let handle_recv = { 
            let alive = alive.clone(); 
            let full_alive = full_alive.clone(); 
            let task_id = task_id.clone(); 
            let channels = channels.clone(); 
            let send_to_servants = send_to_servants.clone();
            spawn(async move {
                if VERBOSE { println!("CLUSTER {} INITIALIZATION: SETTING {} CHANNELS", cluster_id, nb_channels); }
                let mut receivers = VecDeque::new();
                for (servant_id, servant_channels) in channels {
                    for (ch_i,s_ch) in servant_channels {
                        match send_to_servants.get(&servant_id) {
                            None => {
                                *alive.write().await = false;
                                if *full_alive.read().await { eprintln!("Killing master! Custer channel is undefined!"); } // mute during shutdown
                            },
                            Some(msender) => {
                                let tid = task_id.lock().await.generate();
                                if VERBOSE { 
                                    println!("CLUSTER {} INITIALIZATION: TASK {} -> SET CHANNEL {} FOR SERVANT {}", cluster_id, tid.acknowledge_id(), ch_i, servant_id,); 
                                }
                                match Ctrl(tid, CtrlCell::SetChl(ch_i,s_ch)).send(msender).await {
                                    Err(_) => {
                                        *alive.write().await = false; 
                                        if *full_alive.read().await { eprintln!("Killing master! Custer channel is closed!"); } // mute during shutdown
                                    },
                                    Ok(receiver) => receivers.push_front(receiver),
                                }
                            }
                        }
                    }    
                }
                let mut handles = VecDeque::new();
                while let Some(receiver) = receivers.pop_back() {
                    let alive = alive.clone();
                    let full_alive = full_alive.clone(); 
                    let task_id = task_id.clone();
                    handles.push_front(spawn(async move {
                        match receiver.await {
                            Ok(reply) => {
                                let tid = match reply {
                                    ReplyToMaster::Ok(tid) => { 
                                        if VERBOSE { println!("CLUSTER {} INITIALIZATION: TASK {} IS DONE", cluster_id, tid); }
                                        tid 
                                    },
                                    ReplyToMaster::Undefined(tid,) => {
                                        *alive.write().await = false; 
                                        eprintln!("Reply from servant: [31mUNDEFINED CHANNEL FOR TASK {}[0m",tid); tid
                                    },
                                    ReplyToMaster::PingFail(tid,) => {
                                        eprintln!("Reply from servant: ping fail for task {}",tid); tid
                                    },
                                    ReplyToMaster::Failure(tid,) => {
                                        eprintln!("Reply from servant: failure for task {}",tid); tid
                                    },
                                    ReplyToMaster::WrongType(tid,) => {
                                        *alive.write().await = false; 
                                        eprintln!("Reply from servant: [31mTYPE MISMATCH FOR TASK {}[0m",tid); tid
                                    },
                                    ReplyToMaster::OutOfTime(tid,) => {
                                        eprintln!("Reply from servant: task {} is out of time",tid); tid
                                    },
                                };
                                match task_id.lock().await.delete(TaskId::new(tid)) { 
                                    Ok(()) => (), Err(msg) => { eprintln!("Reply from servant -> {}",msg); }, 
                                }
                            },
                            Err(_)    => {
                                *alive.write().await = false; 
                                if *full_alive.read().await { eprintln!("Killing master (servant restart not implemented for now)! Custer channel is closed!"); } // mute during shutdown
                            }
                        } 
                    }));
                }
                if !*alive.read().await {
                    panic!("Initialization failed!");
                }
                for handle in handles { let _: Result<_,_> = handle.await; }
                if VERBOSE { println!("END: CLUSTER {} INITIALIZATION", cluster_id); }
            })
        };
        //
        // LISTEN TO SERVANT REQUESTS
        //
        let mut handles = VecDeque::new();
        for (servant_id, mut mreceiver) in recv_from_servants {
            let alive = alive.clone();
            let full_alive = full_alive.clone(); 
            let task_id = task_id.clone();
            let channels = channels.clone(); 
            let send_to_servants = send_to_servants.clone();
            handles.push_front(spawn(async move {
                while *alive.read().await {
                    match mreceiver.recv().await {
                        None => { 
                            *alive.write().await = false; 
                            if *full_alive.read().await { eprintln!("Killing! Custer_recv channel is closed for servant {}!", servant_id); } // mute during shutdown
                        },
                        Some((FailureChl(tid,chan),cluster_reply)) => {
                            eprintln!("Failure on channel {} for task {}!", chan, tid.acknowledge_id(),);
                            match cluster_reply.send(ReplyToServant::Aknowledged(tid.acknowledge_id())) {
                                Err(_) =>  eprintln!("Killing! Failed to reply to servant {} on task {}!", servant_id, tid.acknowledge_id()),
                                Ok(_)  => (),
                            }
                        }
                        Some((StaledChl(tid,chan),cluster_reply)) => {
                            eprintln!("Staled channel {} for task {} and servant {}!", chan, tid.acknowledge_id(), servant_id,);
                            match cluster_reply.send(ReplyToServant::Aknowledged(tid.acknowledge_id())) {
                                Err(_) =>  eprintln!("Killing! Failed to reply to servant {} on task {}!", servant_id, tid.acknowledge_id()),
                                Ok(_)  => (),
                            }
                        }
                        Some((Shutdown(_tid,),_cluster_reply)) => {
                            *full_alive.write().await = false; // muting error messages 
                            if VERBOSE { println!("CLUSTER {} : TURN OFF {} CHANNELS", cluster_id, nb_channels); }
                            let mut receivers = VecDeque::new();
                            for (&servant_id, servant_channels) in &channels {
                                for &(ch_i,_) in servant_channels {
                                    match send_to_servants.get(&servant_id) {
                                        None => {
                                            *alive.write().await = false;
                                            if *full_alive.read().await { eprintln!("Killing master! Custer channel is undefined!"); } // mute during shutdown                                             
                                        },
                                        Some(msender) => {
                                            let tid = task_id.lock().await.generate();
                                            if VERBOSE { 
                                                println!("CLUSTER {} : TASK {} -> TURN OFF CHANNEL {} FOR SERVANT {}", cluster_id, tid.acknowledge_id(), ch_i, servant_id,); 
                                            }
                                            match Ctrl(tid, CtrlCell::TurnOffChl(ch_i,)).send(msender).await {
                                                Err(_) => {
                                                    *alive.write().await = false;
                                                    if *full_alive.read().await { eprintln!("Killing master! Custer channel is closed!"); } // mute during shutdown
                                                },
                                                Ok(receiver) => receivers.push_front(receiver),
                                            }                
                                        }
                                    }
                                }    
                            }
                            let mut handles = VecDeque::new();
                            while let Some(receiver) = receivers.pop_back() {
                                let alive = alive.clone();
                                let full_alive = full_alive.clone(); 
                                let task_id = task_id.clone();
                                handles.push_front(spawn(async move {
                                    match receiver.await {
                                        Ok(reply) => {
                                            let tid = match reply {
                                                ReplyToMaster::Ok(tid) => { 
                                                    if VERBOSE { println!("CLUSTER {} : TASK {} IS DONE", cluster_id, tid); }
                                                    tid 
                                                },
                                                ReplyToMaster::Undefined(tid,) => {
                                                    *alive.write().await = false; 
                                                    eprintln!("Reply from servant: [31mUNDEFINED CHANNEL FOR TASK {}[0m",tid); tid
                                                },
                                                ReplyToMaster::PingFail(tid,) => {
                                                    eprintln!("Reply from servant: ping fail for task {}",tid); tid
                                                },
                                                ReplyToMaster::Failure(tid,) => {
                                                    eprintln!("Reply from servant: failure for task {}",tid); tid
                                                },
                                                ReplyToMaster::WrongType(tid,) => {
                                                    *alive.write().await = false; 
                                                    eprintln!("Reply from servant: [31mTYPE MISMATCH FOR TASK {}[0m",tid); tid
                                                },
                                                ReplyToMaster::OutOfTime(tid,) => {
                                                    eprintln!("Reply from servant: task {} is out of time",tid); tid
                                                },
                                            };
                                            match task_id.lock().await.delete(TaskId::new(tid)) { 
                                                Ok(()) => (), Err(msg) => { eprintln!("Reply from servant -> {}",msg); }, 
                                            }
                                        },
                                        Err(_)    => {
                                            *alive.write().await = false; 
                                            if *full_alive.read().await { eprintln!("Killing master (servant restart not implemented for now)! Custer channel is closed!");  } // mute during shutdown
                                        }
                                    } 
                                }));
                            }
                            for handle in handles { let _: Result<_,_> = handle.await; }
                            if VERBOSE { 
                                println!("END: CLUSTER {} -> TURN OFF", cluster_id);

                                println!("CLUSTER {} : KILL {} SERVANTS", cluster_id, send_to_servants.len());
                            }
                            let mut receivers = VecDeque::new();
                            for (servid, msender) in &send_to_servants {
                                let tid = task_id.lock().await.generate();
                                if VERBOSE { println!("CLUSTER {}: TASK {} -> KILL SERVANT {}", cluster_id, tid.acknowledge_id(), servid,); }
                                match Ctrl(tid, CtrlCell::Kill).send(msender).await {
                                    Err(_) => {
                                        *alive.write().await = false; 
                                        if *full_alive.read().await { eprintln!("Killing master! Custer channel is closed!"); } // mute during shutdown
                                    },
                                    Ok(receiver) => receivers.push_front(receiver),
                                }
                            };
                            let mut handles = VecDeque::new();
                            while let Some(receiver) = receivers.pop_back() {
                                let alive = alive.clone();
                                let full_alive = full_alive.clone(); 
                                let task_id = task_id.clone();
                                handles.push_front(spawn(async move {
                                    match receiver.await {
                                        Ok(reply) => {
                                            let tid = match reply {
                                                ReplyToMaster::Ok(tid) => { 
                                                    if VERBOSE { println!("CLUSTER {} : TASK {} IS DONE", cluster_id, tid); }
                                                    tid 
                                                },
                                                ReplyToMaster::Undefined(tid,) => {
                                                   *alive.write().await = false; 
                                                    eprintln!("Reply from servant: [31mUNDEFINED CHANNEL FOR TASK {}[0m",tid); tid
                                                },
                                                ReplyToMaster::PingFail(tid,) => {
                                                    eprintln!("Reply from servant: ping fail for task {}",tid); tid
                                                },
                                                ReplyToMaster::Failure(tid,) => {
                                                    eprintln!("Reply from servant: failure for task {}",tid); tid
                                                },
                                                ReplyToMaster::WrongType(tid,) => {
                                                    *alive.write().await = false; 
                                                    eprintln!("Reply from servant: [31mTYPE MISMATCH FOR TASK {}[0m",tid); tid
                                                },
                                                ReplyToMaster::OutOfTime(tid,) => {
                                                    eprintln!("Reply from servant: task {} is out of time",tid); tid
                                                },
                                            };
                                            match task_id.lock().await.delete(TaskId::new(tid)) { 
                                                Ok(()) => (), Err(msg) => { eprintln!("Reply from servant -> {}",msg); }, 
                                            }
                                        },
                                        Err(_)    => {
                                            *alive.write().await = false; 
                                            if *full_alive.read().await { eprintln!("Killing master (servant restart not implemented for now)! Custer channel is closed!"); } // mute during shutdown                                             
                                        }
                                    } 
                                }));
                            }
                            for handle in handles { let _: Result<_,_> = handle.await; }
                            *alive.write().await = false;
                            if VERBOSE { println!("END: CLUSTER {} -> KILL", cluster_id); }

                        }
                    }
                }    
            }));
        }
        //
        // WAIT FOR INITIALIZATION TO BE DONE
        let _ = handle_recv.await.is_err();
        if *alive.read().await { 
            //
            // AND THEN START ALL
            //
            let handle_recv = { 
                let alive = alive.clone(); 
                let full_alive = full_alive.clone(); 
                let task_id = task_id.clone(); 
                let channels = channels.clone(); 
                let send_to_servants = send_to_servants.clone();
                spawn(async move {
                    if VERBOSE { println!("CLUSTER {} : TURN ON {} CHANNELS", cluster_id, nb_channels); }
                    let mut receivers = VecDeque::new();
                    for (servant_id, servant_channels) in channels {
                        for (ch_i,_) in servant_channels {
                            match send_to_servants.get(&servant_id) {
                                None => {
                                    *alive.write().await = false; 
                                    if *full_alive.read().await { eprintln!("Killing master! Custer channel is undefined!"); } // mute during shutdown 
                                },
                                Some(msender) => {
                                    let tid = task_id.lock().await.generate();
                                    if VERBOSE { 
                                        println!("CLUSTER {} : TASK {} -> TURN ON CHANNEL {} FOR SERVANT {}", cluster_id, tid.acknowledge_id(), ch_i, servant_id,); 
                                    }
                                    match Ctrl(tid, CtrlCell::TurnOnChl(ch_i,)).send(msender).await {
                                        Err(_) => {
                                            *alive.write().await = false; 
                                            if *full_alive.read().await { eprintln!("Killing master! Custer channel is closed!"); } // mute during shutdown 
                                        },
                                        Ok(receiver) => receivers.push_front(receiver),
                                    }      
                                }
                            }
                        }    
                    }
                    let mut handles = VecDeque::new();
                    while let Some(receiver) = receivers.pop_back() {
                        let alive = alive.clone();
                        let full_alive = full_alive.clone(); 
                        let task_id = task_id.clone();
                        handles.push_front(spawn(async move {
                            match receiver.await {
                                Ok(reply) => {
                                    let tid = match reply {
                                        ReplyToMaster::Ok(tid) => { 
                                            if VERBOSE { println!("CLUSTER {} : TASK {} IS DONE", cluster_id, tid); }
                                            tid 
                                        },
                                        ReplyToMaster::Undefined(tid,) => {
                                            *alive.write().await = false; 
                                            eprintln!("Reply from servant: [31mUNDEFINED CHANNEL FOR TASK {}[0m",tid); tid
                                        },
                                        ReplyToMaster::PingFail(tid,) => {
                                            eprintln!("Reply from servant: ping fail for task {}",tid); tid
                                        },
                                        ReplyToMaster::Failure(tid,) => {
                                            eprintln!("Reply from servant: failure for task {}",tid); tid
                                        },
                                        ReplyToMaster::WrongType(tid,) => {
                                            *alive.write().await = false; 
                                            eprintln!("Reply from servant: [31mTYPE MISMATCH FOR TASK {}[0m",tid); tid
                                        },
                                        ReplyToMaster::OutOfTime(tid,) => {
                                            eprintln!("Reply from servant: task {} is out of time",tid); tid
                                        },
                                    };
                                    match task_id.lock().await.delete(TaskId::new(tid)) { 
                                        Ok(()) => (), Err(msg) => { eprintln!("Reply from servant -> {}",msg); }, 
                                    }
                                },
                                Err(_)    => {
                                    *alive.write().await = false; 
                                    if *full_alive.read().await { eprintln!("Killing master (servant restart not implemented for now)! Custer channel is closed!"); } // mute during shutdown                                  
                                }
                            } 
                        }));
                    }
                    for handle in handles { let _: Result<_,_> = handle.await; }
                    if VERBOSE { println!("END: CLUSTER {} -> TURN ON", cluster_id); }
                })
            };
            let _: Result<_,_> = handle_recv.await;
            for handle in handles { let _: Result<_,_> = handle.await;  }            
        }

//        println!("\nMASTER {} IS DONE", cluster_id);
    }
}
