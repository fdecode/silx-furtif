use crate::{
    shared::id_tools::TaskId,
    ServantIdType,
    traits::procell::{ ProcessType as pty, HasProcess, },
    structs::cells::{
        components::MembraneType,
        futures::{ 
            FutureUndefined, FutureQuerySender, FutureQueryReceiver, FutureBroadCastSender, FutureBroadCastReceiver, FutureSignalSender, FutureSignalReceiver,
            FutureNetQuerySender, FutureNetQueryReceiver, FutureNetBroadCastSender, FutureNetBroadCastReceiver,
            FutureTurnOnChannel, FutureTurnOffChannel, FuturePingChannel, FutureKillChannel,
        },
        ctrl_message::{ CtrlCell, MsgFromMaster, SetChannel as sch, ReplyToMaster, RecvFromMaster, SenderToMaster, },
    },
};

use std::{ sync::Arc, time::Duration, future::Future, fmt::Debug, fmt::{ Formatter, Error, }, };

use tokio::{ spawn, time::timeout, sync::{ Mutex, RwLock, }, };
use fnv::FnvHashMap;

const VERBOSE: bool = false;

/// doc to be defined
pub struct Servant {
    servant_id: ServantIdType,
    servant_name: String,
    membrane: MembraneType,
    processes: Mutex<Box<dyn HasProcess + Send>>,
    cluster_recv: RecvFromMaster,
    alive: Arc<RwLock<bool>>,
    max_cycle_time: Duration,
}
impl Debug for Servant {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Servant {{ servant_id: {}, servant_name: {}, max_cycle_time: {:?}, .. }}", self.servant_id, self.servant_name, self.max_cycle_time)
    }
}
impl Servant {
    pub (crate) fn new(servant_id: ServantIdType, servant_name: String, processes: Box<dyn HasProcess + Send>, 
                                                  max_cycle_time: Duration, cluster_recv: RecvFromMaster,) -> Self {
        let processes = Mutex::new(processes);
        Self { 
            servant_id, servant_name,
            processes, cluster_recv, max_cycle_time, 
            membrane: Arc::new(RwLock::new(FnvHashMap::default())), 
            alive: Arc::new(RwLock::new(true)), 
        }
    }

    async fn launch_task<T>(&self, tid: TaskId, task: T, reply: SenderToMaster)
                                                        where T: Future<Output = ReplyToMaster> + Send + 'static, {
        let max_cycle_time = self.max_cycle_time;
        let alive = self.alive.clone();
        spawn(async move {
            match timeout(max_cycle_time, task,).await {
                Ok(rep) => {
                    if reply.send(rep).is_err() {
                        *alive.write().await = false; eprintln!("Killing servant! Failed to reply to cluster!");
                    }
                },
                Err(_) => { 
                    if reply.send(ReplyToMaster::OutOfTime(tid.acknowledge_id())).is_err() {
                        *alive.write().await = false; eprintln!("Killing servant! Failed to reply to cluster!");
                    }
                },
            }    
        });
    }

    async fn kill(&mut self) {
        let handles: Vec<_> = self.membrane.write().await.drain().map(|(_,(_,_,_,h))| h).collect();
        for rhandle in &handles { rhandle.read().await.abort(); }
        for handle in handles { let _ = (&mut *handle.write().await).await; }
    }

    pub (crate) async fn run(mut self) {
        if VERBOSE { println!("SERVANT {} : RUNNING START", self.servant_id); }
        use MsgFromMaster::Ctrl as Ctrl;
        let alive = self.alive.clone();
        while *alive.read().await {
            if VERBOSE { println!("SERVANT {} : WAITING FOR TASK", self.servant_id,); }
            match self.cluster_recv.recv().await {
                None => { *self.alive.write().await = false; eprintln!("Killing servant! Cluster channel is closed!"); },
                Some((Ctrl(tid,ctrl),cluster_reply)) => {
                    let acknowledge_id =  tid.acknowledge_id();
                    if VERBOSE { println!("SERVANT {} : START TASK {}", self.servant_id, acknowledge_id,); }
                    let alive = alive.clone();
                    match ctrl {
                        CtrlCell::SetChl(channel, set_channel) => {
                            let process = self.processes.lock().await.process(channel);
                            match (process, set_channel) {
                                (None,_)  => self.launch_task(tid, FutureUndefined::new(acknowledge_id,),cluster_reply).await,

                                (Some(pty::Query{in_chan,in_type,out_chan,out_type}),sch::NetQuerySender{max_ping,query_type,reply_type,sender,},) => {
                                    let membrane = self.membrane.clone();
                                    self.launch_task(tid, FutureNetQuerySender::new(acknowledge_id, membrane, channel,
                                        in_chan, in_type, out_chan, out_type, max_ping, query_type, reply_type, sender,
                                    ),cluster_reply).await;
                                },
                                (Some(pty::Reply{in_type,out_type,mapper,}),sch::NetQueryReceiver{max_ping,query_type,reply_type,receiver,},) => {
                                    let membrane = self.membrane.clone();
                                    self.launch_task(tid, FutureNetQueryReceiver::new(acknowledge_id, membrane, channel,
                                            in_type, out_type, mapper, max_ping, query_type, reply_type, receiver,
                                    ),cluster_reply).await;
                                },
                                (Some(pty::Emit{in_chan,in_type,}),sch::NetBroadcastSender{max_ping,data_type,sender,},) => {
                                    let membrane = self.membrane.clone();
                                    self.launch_task(tid, FutureNetBroadCastSender::new(acknowledge_id, membrane, channel, 
                                        in_chan, in_type, max_ping, data_type, sender,
                                    ),cluster_reply).await;
                                },
                                (Some(pty::Read{out_chan,out_type,}),sch::NetBroadcastReceiver{max_ping,data_type,receiver,},) => {
                                    let membrane = self.membrane.clone();
                                    self.launch_task(tid, FutureNetBroadCastReceiver::new(acknowledge_id, membrane, channel,
                                        out_chan, out_type, max_ping, data_type, receiver,
                                    ),cluster_reply).await;
                                },

                                (Some(pty::Query{in_chan,in_type,out_chan,out_type}),sch::QuerySender{max_ping,query_type,reply_type,sender,},) => {
                                    let membrane = self.membrane.clone();
                                    self.launch_task(tid, FutureQuerySender::new(acknowledge_id, membrane, channel,
                                        in_chan, in_type, out_chan, out_type, max_ping, query_type, reply_type, sender,
                                    ),cluster_reply).await;
                                },
                                (Some(pty::Reply{in_type,out_type,mapper,}),sch::QueryReceiver{max_ping,query_type,reply_type,receiver,},) => {
                                    let membrane = self.membrane.clone();
                                    self.launch_task(tid, FutureQueryReceiver::new(acknowledge_id, membrane, channel,
                                            in_type, out_type, mapper, max_ping, query_type, reply_type, receiver,
                                    ),cluster_reply).await;
                                },
                                (Some(pty::Emit{in_chan,in_type,}),sch::BroadcastSender{max_ping,data_type,sender,},) => {
                                    let membrane = self.membrane.clone();
                                    self.launch_task(tid, FutureBroadCastSender::new(acknowledge_id, membrane, channel, 
                                        in_chan, in_type, max_ping, data_type, sender,
                                    ),cluster_reply).await;
                                },
                                (Some(pty::Read{out_chan,out_type,}),sch::BroadcastReceiver{max_ping,data_type,receiver,},) => {
                                    let membrane = self.membrane.clone();
                                    self.launch_task(tid, FutureBroadCastReceiver::new(acknowledge_id, membrane, channel,
                                        out_chan, out_type, max_ping, data_type, receiver,
                                    ),cluster_reply).await;
                                },
                                (Some(pty::Emit{in_chan,in_type,}),sch::SignalSender{max_ping,data_type,sender,},) => {
                                    let membrane = self.membrane.clone();
                                    self.launch_task(tid, FutureSignalSender::new(acknowledge_id, membrane, channel,
                                        in_chan, in_type, max_ping, data_type, sender,
                                    ),cluster_reply).await;
                                },
                                (Some(pty::RefRead{assert,out_type,}),sch::SignalReceiver{max_ping,data_type,receiver,},) => {
                                    let membrane = self.membrane.clone();
                                    self.launch_task(tid, FutureSignalReceiver::new(acknowledge_id, membrane, channel,
                                        assert, out_type, max_ping, data_type, receiver,
                                    ),cluster_reply).await;
                                },
                                // MANQUE LE FLOW INITIALISE
                                _ => panic!("impossible case"),
                            }
                        },
                        CtrlCell::TurnOnChl(channel) => {
                            let membrane = self.membrane.clone();
                            self.launch_task(tid, FutureTurnOnChannel::new(acknowledge_id, membrane, channel,),cluster_reply).await;
                        },
                        CtrlCell::TurnOffChl(channel) => { // turn off channel
                            let membrane = self.membrane.clone();
                            self.launch_task(tid, FutureTurnOffChannel::new(acknowledge_id, membrane, channel,),cluster_reply).await;
                        },
                        CtrlCell::PingChl(channel) => { // Ping the channel
                            let membrane = self.membrane.clone();
                            self.launch_task(tid, FuturePingChannel::new(acknowledge_id, membrane, channel,),cluster_reply).await;
                        },
                        CtrlCell::KillChl(channel) => { // Kill channel
                            let membrane = self.membrane.clone();
                            self.launch_task(tid, FutureKillChannel::new(acknowledge_id, membrane, channel,),cluster_reply).await;
                        },
                        CtrlCell::Kill => { // Kill Cell
                            self.kill().await; *alive.write().await = false; // kill process is not spawn, but processed directly
                            if cluster_reply.send(ReplyToMaster::Ok(acknowledge_id,)).is_err() { eprintln!("Killing servant! Failed to reply to cluster!"); }
                        },
                    };
                    if VERBOSE { println!("SERVANT {} : TASK {} DONE", self.servant_id, acknowledge_id,); }
                },
            }
        }
    }

}
