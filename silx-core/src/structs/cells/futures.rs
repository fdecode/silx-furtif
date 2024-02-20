use crate::{
    shared::id_tools::AcknowledgeId,
    ChannelIdType,
    traits::FullId,
    structs::{ 
        cells::{
            components::{ Channelling, Flag, Ping, MembraneType, Mapper, Assert, },
            ctrl_message::ReplyToMaster,
        },
        archmod::archannel::{
            SerializedDataQuerySender as BinQySender, SerializedDataQueryReceiver as BinQyReceiver, 
            RootSerializedDataBroadcastSender as BinBcSender, RootSerializedDataBroadcastReceiver as BinBcReceiver, 
            SerializedDataSignalSender as BinSgSender, SerializedDataSignalReceiver as BinSgReceiver,         
            SerializedDataDispatchSender as BinDpSender, SerializedDataDispatchReceiver as BinDpReceiver,
        },
    },
};
use std::{ sync::Arc, time::Duration, future::Future, };
use async_scoped::TokioScope;
use tokio::{ spawn, sync::RwLock, };

// Channel setting

/// Future builder replying undefined status to the master
pub  (crate) enum FutureUndefined {}

/// Future builder implementing sending query cycle to a servant of another cluster  
pub  (crate) enum FutureNetQuerySender {}

/// Future builder implementing receving query cycle from a servant of another cluster  
pub  (crate) enum FutureNetQueryReceiver {}

/// Future builder implementing broadcast sending cycle to a servant of another cluster  
pub  (crate) enum FutureNetBroadCastSender {}

/// Future builder implementing broadcast receving cycle from a servant of another cluster  
pub  (crate) enum FutureNetBroadCastReceiver {}


/// Future builder implementing sending query cycle to a servant of same cluster  
pub  (crate) enum FutureQuerySender {}

/// Future builder implementing receving query cycle from a servant of same cluster  
pub  (crate) enum FutureQueryReceiver {}

/// Future builder implementing broadcast sending cycle to a servant of same cluster  
pub  (crate) enum FutureBroadCastSender {}

/// Future builder implementing broadcast receving cycle from a servant of same cluster  
pub  (crate) enum FutureBroadCastReceiver {}

/// Future builder implementing signal (a reference to archived data) sending cycle to a servant of same cluster  
pub  (crate) enum FutureSignalSender {}
/// Future builder implementing signal (a reference to archived data) receving cycle from a servant of same cluster  
pub  (crate) enum FutureSignalReceiver {}

// other controls
/// Future builder implementing channel activation
pub  (crate) enum FutureTurnOnChannel {}

/// Future builder implementing channel desactivation
pub  (crate) enum FutureTurnOffChannel {}

/// Future builder implementing channel pinging
pub  (crate) enum FuturePingChannel {}

/// Future builder implementing channel killing
pub  (crate) enum FutureKillChannel {}

impl FutureUndefined {
    /// Build future replying undefined status to the master
    /// * `acknowledge_id: AcknowledgeId` : id of the task
    /// * Output: future replying to master
    pub  (crate) fn new(acknowledge_id: AcknowledgeId,) -> impl Future<Output = ReplyToMaster> {
        async move { ReplyToMaster::Undefined(acknowledge_id) }
    }

}


impl FutureNetQuerySender {
    /// Build future implementing sending query cycle to a servant of another cluster  
    /// * `acknowledge_id: AcknowledgeId` : id of the task
    /// * `membrane: MembraneType` : cell membrane (contains channels informations and status)
    /// * `channel: ChannelIdType` : channel identifier
    /// * `in_chan: BinDpReceiver` : channel intern receiver
    /// * `in_type: FullId` : identifier of the data type of intern receiver
    /// * `out_chan: BinDpSender` : channel intern sender
    /// * `out_type: FullId` : identifier of the data type of intern sender
    /// * `max_ping: Duration` : max duration when pinging the channel
    /// * `query_type: FullId` : identifier of the data type of query
    /// * `reply_type: FullId` :  identifier of the data type of reply
    /// * `sender: BinQySender` : Query sender
    /// * Output: future replying to master
    pub  (crate) fn new(acknowledge_id: AcknowledgeId, membrane: MembraneType, channel: ChannelIdType,
        in_chan: BinDpReceiver, in_type: FullId, out_chan: BinDpSender, out_type: FullId,
        max_ping: Duration, query_type: FullId, reply_type: FullId, sender: BinQySender,
    ) -> impl Future<Output = ReplyToMaster> {
        async move { 
            if in_type == query_type && out_type == reply_type {
                let channelling = Channelling::NetQuerySend(sender.clone());
                let flag = Arc::new(Flag::new());
                let ping = Arc::new(Ping::new(max_ping));
                let handle =  spawn( { let flag = flag.clone(); let ping = ping.clone(); async move { 
                    let mut alive = true;
                    while alive {
                        flag.go().await; // block process until activation
                        ping.unping(); //unping pingger
                        match in_chan.recv().await {
                            Ok(bytes)   => {
                                match bytes.send(&sender).await { 
                                    Err(_)        => { alive = false; },
                                    Ok(oreceiver) => {
                                        match oreceiver.await {
                                            Ok(bytes) => {
                                                if out_chan.send(bytes).await.is_err() { alive = false; }
                                            },
                                            Err(_)    => alive = false, // channel is closed : stop the future
                                        }
                                    },
                                }
                            },
                            Err(_)      => alive = false, // channel is closed : stop the future
                        }
                    } 
                } } );
                membrane.write().await.insert(channel,(channelling,flag,ping,Arc::new(RwLock::new(handle))));
                ReplyToMaster::Ok(acknowledge_id)
            } else { ReplyToMaster::WrongType(acknowledge_id) }
        }
    }
}

impl FutureNetQueryReceiver {
    /// Build future implementing receving query cycle from a servant of another cluster  
    /// * `acknowledge_id: AcknowledgeId` : id of the task
    /// * `membrane: MembraneType` : cell membrane (contains channels informations and status)
    /// * `channel: ChannelIdType` : channel identifier
    /// * `in_type: FullId` : identifier of the data type of intern recceiver
    /// * `out_type: FullId` : identifier of the data type of intern sender
    /// * `mapper: Mapper` : query to answer mapper
    /// * `max_ping: Duration` : max duration when pinging the channel
    /// * `query_type: FullId` : identifier of the data type of query
    /// * `reply_type: FullId` :  identifier of the data type of reply
    /// * `receiver: BinQyReceiver` : Query receiver
    /// * Output: future replying to master
    pub  (crate) fn new(acknowledge_id: AcknowledgeId, membrane: MembraneType, channel: ChannelIdType,
        in_type: FullId, out_type: FullId, mapper: Mapper, max_ping: Duration, query_type: FullId, reply_type: FullId, receiver: BinQyReceiver,
    ) -> impl Future<Output = ReplyToMaster> {
        async move { 
            let aknowledge_id = acknowledge_id;
            if in_type == query_type && out_type == reply_type {
                let channelling = Channelling::NetQueryRecv(receiver.clone());
                let flag = Arc::new(Flag::new());
                let ping = Arc::new(Ping::new(max_ping));
                let handle =  spawn( { let flag = flag.clone(); let ping = ping.clone(); async move { 
                    let mut alive = true;
                    while alive {
                        flag.go().await; // block process until activation
                        ping.unping(); //unping pingger
                        match receiver.recv().await {
                            Ok((mut bytes,resender)) => {
                                match unsafe {
                                    TokioScope::scope_and_collect(|scope| {
                                        scope.spawn(mapper(std::mem::transmute(&mut bytes)));
                                    })
                                }.await.1[0] {
                                    Ok(_) => (),
                                    Err(_) => {
                                        alive = false;
                                        eprintln!("spawning failure for task {}",acknowledge_id);
                                    },
                                };
                                if resender.send(bytes).is_err() { // resend error does not end the process
                                    eprintln!("Failed to resend data");
                                }
                            },
                            Err(_)    => alive = false, // channel is closed : stop the future
                        };
                    } 
                } } );
                membrane.write().await.insert(channel,(channelling,flag,ping,Arc::new(RwLock::new(handle))));
                ReplyToMaster::Ok(aknowledge_id)
            } else { ReplyToMaster::WrongType(aknowledge_id) }
        }
    }
}

impl FutureNetBroadCastSender {
    /// Build future implementing broadcast sending cycle to a servant of another cluster  
    /// * `acknowledge_id: AcknowledgeId` : id of the task
    /// * `membrane: MembraneType` : cell membrane (contains channels informations and status)
    /// * `channel: ChannelIdType` : channel identifier
    /// * `in_chan: BinDpReceiver` : channel intern receiver
    /// * `in_type: FullId` : identifier of the data type of intern receiver
    /// * `max_ping: Duration` : max duration when pinging the channel
    /// * `data_type: FullId` : identifier of broadcasted data type
    /// * `sender: BinBcSender` : Broadcast sender
    /// * Output: future replying to master
    pub  (crate) fn new(acknowledge_id: AcknowledgeId, membrane: MembraneType, channel: ChannelIdType,
        in_chan: BinDpReceiver, in_type: FullId,  max_ping: Duration, data_type: FullId, sender: BinBcSender,
    ) -> impl Future<Output = ReplyToMaster> {
        async move { 
            if in_type == data_type {
                let asender = sender.instance();
                let channelling = Channelling::NetBroadcastSend(sender);
                let flag = Arc::new(Flag::new());
                let ping = Arc::new(Ping::new(max_ping));
                let handle =  spawn( { let flag = flag.clone(); let ping = ping.clone(); async move { 
                    let mut alive = true;
                    while alive {
                        flag.go().await; // block process until activation
                        ping.unping(); //unping pingger
                        match in_chan.recv().await {
                            Ok(bytes) => if asender.send(bytes).is_err() { alive = false; },
                            Err(_)    => alive = false, // channel is closed : stop the future
                        };
                    } 
                } } );
                membrane.write().await.insert(channel,(channelling,flag,ping,Arc::new(RwLock::new(handle))));
                ReplyToMaster::Ok(acknowledge_id)
            } else { ReplyToMaster::WrongType(acknowledge_id) }
        }
    }
}

impl FutureNetBroadCastReceiver {
    /// Build future implementing broadcast receving cycle from a servant of another cluster  
    /// * `acknowledge_id: AcknowledgeId` : id of the task
    /// * `membrane: MembraneType` : cell membrane (contains channels informations and status)
    /// * `channel: ChannelIdType` : channel identifier
    /// * `out_chan: BinDpSender` : channel intern sender
    /// * `out_type: FullId` : identifier of the data type of intern sender
    /// * `max_ping: Duration` : max duration when pinging the channel
    /// * `data_type: FullId` : identifier of broadcasted data type
    /// * `receiver: BinBcReceiver` : Broadcast receiver
    /// * Output: future replying to master
    pub  (crate) fn new(acknowledge_id: AcknowledgeId, membrane: MembraneType, channel: ChannelIdType,
        out_chan: BinDpSender, out_type: FullId, max_ping: Duration, data_type: FullId, receiver: BinBcReceiver,
    ) -> impl Future<Output = ReplyToMaster> {
        async move { 
            if out_type == data_type {
                let mut areceiver = receiver.instance();
                let channelling = Channelling::NetBroadcastRecv(receiver);
                let flag = Arc::new(Flag::new());
                let ping = Arc::new(Ping::new(max_ping));
                let handle =  spawn( { let flag = flag.clone(); let ping = ping.clone(); async move { 
                    let mut alive = true;
                    while alive {
                        flag.go().await; // block process until activation
                        ping.unping(); //unping pingger
                        match areceiver.recv().await {
                            Ok(bytes) =>  if out_chan.send(bytes).await.is_err() { alive = false; },
                            Err(_)    => alive = false, // channel is closed : stop the future
                        };
                    } 
                } } );
                membrane.write().await.insert(channel,(channelling,flag,ping,Arc::new(RwLock::new(handle))));
                ReplyToMaster::Ok(acknowledge_id)
            } else { ReplyToMaster::WrongType(acknowledge_id) }
        }
    }
}


impl FutureQuerySender {
    pub  (crate) fn new(acknowledge_id: AcknowledgeId, membrane: MembraneType, channel: ChannelIdType,
        in_chan: BinDpReceiver, in_type: FullId, out_chan: BinDpSender, out_type: FullId,
        max_ping: Duration, query_type: FullId, reply_type: FullId, sender: BinQySender,
    ) -> impl Future<Output = ReplyToMaster> {
        async move { 
            if in_type == query_type && out_type == reply_type {
                let channelling = Channelling::QuerySend(sender.clone());
                let flag = Arc::new(Flag::new());
                let ping = Arc::new(Ping::new(max_ping));
                let handle =  spawn( { let flag = flag.clone(); let ping = ping.clone(); async move { 
                    let mut alive = true;
                    while alive {
                        flag.go().await; // block process until activation
                        ping.unping(); //unping pingger
                        match in_chan.recv().await {
                            Ok(bytes)   => {
                                match bytes.send(&sender).await { 
                                    Err(_)        => { alive = false; },
                                    Ok(oreceiver) => {
                                        match oreceiver.await {
                                            Ok(bytes) => {
                                                if out_chan.send(bytes).await.is_err() { alive = false; }
                                            },
                                            Err(_)    => alive = false, // channel is closed : stop the future
                                        }
                                    },
                                }
                            },
                            Err(_)      => alive = false, // channel is closed : stop the future
                        }
                    } 
                } } );
                membrane.write().await.insert(channel,(channelling,flag,ping,Arc::new(RwLock::new(handle))));
                ReplyToMaster::Ok(acknowledge_id)
            } else { ReplyToMaster::WrongType(acknowledge_id) }
        }
    }
}

impl FutureQueryReceiver {
    pub  (crate) fn new(acknowledge_id: AcknowledgeId, membrane: MembraneType, channel: ChannelIdType,
        in_type: FullId, out_type: FullId, mapper: Mapper, max_ping: Duration, query_type: FullId, reply_type: FullId, receiver: BinQyReceiver,
    ) -> impl Future<Output = ReplyToMaster> {
        async move { 
            let aknowledge_id = acknowledge_id;
            if in_type == query_type && out_type == reply_type {
                let channelling = Channelling::QueryRecv(receiver.clone());
                let flag = Arc::new(Flag::new());
                let ping = Arc::new(Ping::new(max_ping));
                let handle =  spawn( { let flag = flag.clone(); let ping = ping.clone(); async move { 
                    let mut alive = true;
                    while alive {
                        flag.go().await; // block process until activation
                        ping.unping(); //unping pingger
                        match receiver.recv().await {
                            Ok((mut bytes,resender)) => {
                                match unsafe {
                                    TokioScope::scope_and_collect(|scope| {
                                        scope.spawn(mapper(std::mem::transmute(&mut bytes)));
                                    })
                                }.await.1[0] {
                                    Ok(_) => (),
                                    Err(_) => {
                                        alive = false;
                                        eprintln!("spawning failure for task {}", acknowledge_id);
                                    },
                                };
                                if resender.send(bytes).is_err() { // resend error does not end the process
                                    eprintln!("Failed to resend data");
                                }
                            },
                            Err(_)    => alive = false, // channel is closed : stop the future
                        };
                    } 
                } } );
                membrane.write().await.insert(channel,(channelling,flag,ping,Arc::new(RwLock::new(handle))));
                ReplyToMaster::Ok(aknowledge_id)
            } else { ReplyToMaster::WrongType(aknowledge_id) }
        }
    }
}

impl FutureBroadCastSender {
    pub  (crate) fn new(acknowledge_id: AcknowledgeId, membrane: MembraneType, channel: ChannelIdType,
        in_chan: BinDpReceiver, in_type: FullId,  max_ping: Duration, data_type: FullId, sender: BinBcSender,
    ) -> impl Future<Output = ReplyToMaster> {
        async move { 
            if in_type == data_type {
                let asender = sender.instance();
                let channelling = Channelling::BroadcastSend(sender);
                let flag = Arc::new(Flag::new());
                let ping = Arc::new(Ping::new(max_ping));
                let handle =  spawn( { let flag = flag.clone(); let ping = ping.clone(); async move { 
                    let mut alive = true;
                    while alive {
                        flag.go().await; // block process until activation
                        ping.unping(); //unping pingger
                        match in_chan.recv().await {
                            Ok(bytes) => if asender.send(bytes).is_err() { alive = false; },
                            Err(_)    => alive = false, // channel is closed : stop the future
                        };
                    } 
                } } );
                membrane.write().await.insert(channel,(channelling,flag,ping,Arc::new(RwLock::new(handle))));
                ReplyToMaster::Ok(acknowledge_id)
            } else { ReplyToMaster::WrongType(acknowledge_id) }
        }
    }
}

impl FutureBroadCastReceiver {
    pub  (crate) fn new(acknowledge_id: AcknowledgeId, membrane: MembraneType, channel: ChannelIdType,
        out_chan: BinDpSender, out_type: FullId, max_ping: Duration, data_type: FullId, receiver: BinBcReceiver,
    ) -> impl Future<Output = ReplyToMaster> {
        async move { 
            if out_type == data_type {
                let mut areceiver = receiver.instance();
                let channelling = Channelling::BroadcastRecv(receiver);
                let flag = Arc::new(Flag::new());
                let ping = Arc::new(Ping::new(max_ping));
                let handle =  spawn( { let flag = flag.clone(); let ping = ping.clone(); async move { 
                    let mut alive = true;
                    while alive {
                        flag.go().await; // block process until activation
                        ping.unping(); //unping pingger
                        match areceiver.recv().await {
                            Ok(bytes) =>  if out_chan.send(bytes).await.is_err() { alive = false; },
                            Err(_)    => alive = false, // channel is closed : stop the future
                        };
                    } 
                } } );
                membrane.write().await.insert(channel,(channelling,flag,ping,Arc::new(RwLock::new(handle))));
                ReplyToMaster::Ok(acknowledge_id)
            } else { ReplyToMaster::WrongType(acknowledge_id) }
        }
    }
}

impl FutureSignalSender {
    pub  (crate) fn new(acknowledge_id: AcknowledgeId, membrane: MembraneType, channel: ChannelIdType,
        in_chan: BinDpReceiver, in_type: FullId, max_ping: Duration, data_type: FullId, sender: BinSgSender,
    ) -> impl Future<Output = ReplyToMaster> {
        async move { 
            if in_type == data_type {
                let channelling = Channelling::SignalSend(sender.clone());
                let flag = Arc::new(Flag::new());
                let ping = Arc::new(Ping::new(max_ping));
                let handle =  spawn( { let flag = flag.clone(); let ping = ping.clone(); async move { 
                    let mut alive = true;
                    while alive {
                        flag.go().await; // block process until activation
                        ping.unping(); //unping pingger
                        match in_chan.recv().await {
                            Ok(bytes) => if sender.send(bytes).is_err() { alive = false; }, // Nota: the economic send_replace is not used for the moment
                            Err(_)    => alive = false, // channel is closed : stop the future
                        };
                    } 
                } } );
                membrane.write().await.insert(channel,(channelling,flag,ping,Arc::new(RwLock::new(handle))));
                ReplyToMaster::Ok(acknowledge_id)
            } else { ReplyToMaster::WrongType(acknowledge_id) }
        }
    }
}

impl FutureSignalReceiver {
    pub  (crate) fn new(acknowledge_id: AcknowledgeId, membrane: MembraneType, channel: ChannelIdType,
        assert: Assert, out_type: FullId, max_ping: Duration, data_type: FullId, mut receiver: BinSgReceiver,
    ) -> impl Future<Output = ReplyToMaster> {
        async move { 
            let aknowledge_id = acknowledge_id;
            if out_type == data_type {
                let channelling = Channelling::SignalRecv(receiver.clone());
                receiver.borrow_and_update(); // update at beginning, so as to skip  undefined data
                let flag = Arc::new(Flag::new());
                let ping = Arc::new(Ping::new(max_ping));
                let handle =  spawn( { let flag = flag.clone(); let ping = ping.clone(); async move { 
                    let mut alive = true;
                    while alive {
                        flag.go().await; // block process until activation
                        ping.unping(); //unping pingger
                        match receiver.changed().await {
                            Ok(_)   => {
                                if let Some(bor) = receiver.borrow_and_update() { 
                                    match unsafe {
                                        TokioScope::scope_and_collect(|scope| {
                                            scope.spawn(assert(std::mem::transmute(&bor)));
                                        })
                                    }.await.1[0] {
                                        Ok(_) => (),
                                        Err(_) => {
                                            alive = false;
                                            eprintln!("spawning failure for task {}", acknowledge_id);
                                        },
                                    };
                                } else { panic!("Unexpected: signal is not inited") }
                            },
                            Err(_)  => alive = false, // channel is closed : stop the future
                        };
                    } 
                } } );
                membrane.write().await.insert(channel,(channelling,flag,ping,Arc::new(RwLock::new(handle))));
                ReplyToMaster::Ok(aknowledge_id)
            } else { ReplyToMaster::WrongType(aknowledge_id) }
        }
    }
}

impl FutureTurnOnChannel {
    pub  (crate) fn new(acknowledge_id: AcknowledgeId, membrane: MembraneType, channel: ChannelIdType,) -> impl Future<Output = ReplyToMaster> {
            async move { 
                let the_membrane = membrane.read().await.get(&channel).cloned();
                if let Some((_, ref activ, _, _)) = the_membrane {
                    activ.activate().await; ReplyToMaster::Ok(acknowledge_id,)
                } else { ReplyToMaster::Undefined(acknowledge_id,) }
            }
    }
}

impl FutureTurnOffChannel {
    pub  (crate) fn new(acknowledge_id: AcknowledgeId, membrane: MembraneType, channel: ChannelIdType,) -> impl Future<Output = ReplyToMaster> {
        async move { 
            let the_membrane = membrane.read().await.get(&channel).cloned();
            if let Some((_, ref activ, _, _)) = the_membrane {
                activ.desactivate().await; ReplyToMaster::Ok(acknowledge_id,)
            } else { ReplyToMaster::Undefined(acknowledge_id,) }
        }
    }
}

impl FuturePingChannel {
    pub  (crate) fn new(acknowledge_id: AcknowledgeId, membrane: MembraneType, channel: ChannelIdType,) -> impl Future<Output = ReplyToMaster> {
        async move { 
            let the_membrane = membrane.read().await.get(&channel).cloned();
            if let Some((ref channelling, _, ref ping, _)) = the_membrane {
                let ch_ok = match channelling {
                    Channelling::NetQuerySend(amsender) => !amsender.is_closed(),
                    Channelling::NetQueryRecv(amreceiv) => !amreceiv.is_closed(),
                    Channelling::NetBroadcastSend(_)    => true,
                    Channelling::NetBroadcastRecv(_)    => true,
                    Channelling::QuerySend(amsender)    => !amsender.is_closed(),
                    Channelling::QueryRecv(amreceiv)    => !amreceiv.is_closed(),
                    Channelling::BroadcastSend(_)       => true,
                    Channelling::BroadcastRecv(_)       => true,
                    Channelling::SignalSend(amsender)   => !amsender.is_closed(),
                    Channelling::SignalRecv(_)          => true,
                };
                let th_ok = ping.ping().await;
                if ch_ok { if th_ok { 
                    ReplyToMaster::Ok(acknowledge_id,) 
                } else { 
                    ReplyToMaster::PingFail(acknowledge_id,) 
                } } else { 
                    ReplyToMaster::Failure(acknowledge_id,) 
                }
            } else { ReplyToMaster::Undefined(acknowledge_id,) }    
        }
    }
}

impl FutureKillChannel {
    pub  (crate) fn new(acknowledge_id: AcknowledgeId, membrane: MembraneType, channel: ChannelIdType,) -> impl Future<Output = ReplyToMaster> {
        async move { 
            let the_membrane = membrane.read().await.get(&channel).cloned();
            if let Some((_,_,_,handle,)) = the_membrane {
                handle.read().await.abort();
                let _ = (&mut *handle.write().await).await;
            } ReplyToMaster::Ok(acknowledge_id,)
        }
    }
}
