use crate::{
    ChannelIdType, 
    structs::archmod::{ 
        ser_data::SerializedData,
        archannel::{ 
            SerializedDataQuerySender as BinQySender, SerializedDataQueryReceiver as BinQyReceiver, 
            RootSerializedDataBroadcastSender as BinBcSender, RootSerializedDataBroadcastReceiver as BinBcReceiver, 
            SerializedDataSignalSender as BinSgSender, SerializedDataSignalReceiver as BinSgReceiver,
        },
    },
};
use std::{ sync::Arc, time::Duration, pin::Pin, future::Future, };
use tokio::{
    time::timeout,
    task::JoinHandle,
    sync::{ Notify, RwLock, },
};
use fnv::FnvHashMap;

pub type MembraneType = Arc<RwLock<FnvHashMap<ChannelIdType, (Channelling,Arc<Flag>,Arc<Ping>,Arc<RwLock<JoinHandle<()>>>)>>>;
pub type Mapper = Arc<dyn Fn(&'static mut SerializedData,) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;
pub type Assert = Arc<dyn Fn(&'static SerializedData,) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;


#[derive(Clone)]
/// doc to be defined
pub enum Channelling {
    NetQuerySend(BinQySender),
    NetQueryRecv(BinQyReceiver),
    NetBroadcastSend(BinBcSender),
    NetBroadcastRecv(BinBcReceiver),
    QuerySend(BinQySender),
    QueryRecv(BinQyReceiver),
    BroadcastSend(BinBcSender),
    BroadcastRecv(BinBcReceiver),
    SignalSend(BinSgSender),
    SignalRecv(BinSgReceiver),
}

/// doc to be defined
pub struct Ping {
    duration: Duration,
    notifier: Notify,
}
impl Ping {
    pub fn new(duration: Duration,) -> Self { Self { duration, notifier: Notify::new(), } }

    // ping the process (done by the main process)
    pub async fn ping(&self) -> bool {
        timeout(self.duration, self.notifier.notified(),).await.is_ok()
    }
    // notify activity (done by the subprocess)
    pub fn unping(&self) { self.notifier.notify_waiters(); }
}

/// doc to be defined
pub struct Flag {
    activated: RwLock<bool>,
    notifier: Notify,
}
impl Flag {
    pub fn new() -> Self { Self { activated: RwLock::new(false), notifier: Notify::new(), } }
    pub async fn go(&self) { if !*self.activated.read().await { self.notifier.notified().await; } }
    pub async fn activate(&self) { *self.activated.write().await = true; self.notifier.notify_waiters(); }
    pub async fn desactivate(&self) { *self.activated.write().await = false; } 
}


