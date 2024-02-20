use std::{ marker::PhantomData, sync::Arc, };

use tokio::{ sync::broadcast as bca, task::JoinHandle, spawn, };

use super::{ ArchData, SerializedData, SlxData, };

////////////////////////////////////////////
// Broadcast channels

/// Broadcast channel builder for archived data
/// * Broadcast means that many senders send to many receivers and data is cloned for each receiver
pub struct ArchBroadcast;
impl ArchBroadcast {
    /// Channel builder for broadcast
    /// * `capacity: usize` : capacity of the channel
    /// * `U` : type of the data; needs to implement `SlxData`
    /// * Output: root sender and receiver
    ///   * these sender and receiver contains a handle reference to a process which maintains the channel alive even if there is no connection
    ///   * the channel can be killed by means of the handle
    pub async fn channel<U>(capacity: usize) -> (RootArchBroadcastSender<U>,RootArchBroadcastReceiver<U>) where U: SlxData {
        let (sender,mut receiver) = bca::channel(capacity);
        let ref_sender = sender.clone();
        let handle_sender = Arc::new(spawn(async move { while receiver.recv().await.is_ok() {} }));
        let handle_receiver = handle_sender.clone();
        let sender = RootSerializedDataBroadcastSender { sender, handle_sender, }; 
        let receiver = RootSerializedDataBroadcastReceiver { ref_sender, handle_receiver, }; 
        (RootArchBroadcastSender { sender, phantom: PhantomData, }, RootArchBroadcastReceiver { receiver, phantom: PhantomData, },)
    }
}


#[derive(Clone,Debug)]
/// Root broadcast sender for serialized data
/// * contains a handle reference to a process which maintains the channel alive even if there is no connection
/// * the channel can be killed by means of the handle
pub struct RootSerializedDataBroadcastSender {
    sender: bca::Sender<SerializedData>,
    handle_sender: Arc<JoinHandle<()>>,
}
#[derive(Clone,Debug)]
/// Root broadcast receiver for serialized data
/// * contains a handle reference to a process which maintains the channel alive even if there is no connection
/// * the channel can be killed by means of the handle
pub struct RootSerializedDataBroadcastReceiver {
    ref_sender: bca::Sender<SerializedData>,
    handle_receiver: Arc<JoinHandle<()>>,
}
/// Root broadcast sender for archived data
/// * `U` : type of the data; needs to implement `SlxData`
pub struct RootArchBroadcastSender<U> where U: SlxData {
    sender: RootSerializedDataBroadcastSender,
    phantom: PhantomData<U>,
}
/// Root broadcast receiver for archived data
/// * `U` : type of the data; needs to implement `SlxData`
pub struct RootArchBroadcastReceiver<U> where U: SlxData {
    receiver: RootSerializedDataBroadcastReceiver,
    phantom: PhantomData<U>,
}

#[derive(Debug)]
/// Broadcast sender for serialized data:
/// * This is obtained from root by instanciating
pub struct SerializedDataBroadcastSender {
    sender: bca::Sender<SerializedData>,
}
#[derive(Debug)]
/// Broadcast receiver for serialized data:
/// * This is obtained from root by instanciating
pub struct SerializedDataBroadcastReceiver {
    receiver: bca::Receiver<SerializedData>,
}
/// Broadcast sender for archived data:
/// * This is obtained from root by instanciating
/// * `U` : type of the data; needs to implement `SlxData`
pub struct ArchBroadcastSender<U> {
    sender: SerializedDataBroadcastSender,
    phantom: PhantomData<U>,
}
/// Broadcast receiver for archived data:
/// * This is obtained from root by instanciating
/// * `U` : type of the data; needs to implement `SlxData`
pub struct ArchBroadcastReceiver<U> {
    receiver: SerializedDataBroadcastReceiver,
    phantom: PhantomData<U>
}

impl<U> Clone for RootArchBroadcastSender<U> where U: SlxData {
    fn clone(&self) -> Self {
        let Self { ref sender, phantom } = *self; let sender = sender.clone(); Self { sender, phantom, }        
    }
}
impl<U> Clone for RootArchBroadcastReceiver<U> where U: SlxData {
    fn clone(&self) -> Self {
        let Self { ref receiver, phantom } = *self; let receiver = receiver.clone(); Self { receiver, phantom, }        
    }
}
impl<U> RootArchBroadcastSender<U> where U: SlxData {
    pub(crate) fn inner(self) -> RootSerializedDataBroadcastSender { self.sender }
    /// Kill the channel
    pub fn deep_kill(&self) { self.sender.deep_kill(); }
    /// Instanciate the sender
    pub fn instance(&self) -> ArchBroadcastSender<U> {
        let Self { ref sender, phantom } = *self; let sender = sender.instance(); ArchBroadcastSender { sender, phantom, }
    }
}
impl RootSerializedDataBroadcastSender {
    pub (crate) fn deep_kill(&self) { self.handle_sender.abort(); }
    
    pub (crate) fn instance(&self) -> SerializedDataBroadcastSender { 
        let Self { ref sender, .. } = *self; let sender = sender.clone(); SerializedDataBroadcastSender { sender, }
    }
}
impl<U> RootArchBroadcastReceiver<U> where U: SlxData {
    pub (crate) fn inner(self) -> RootSerializedDataBroadcastReceiver { self.receiver }
    /// Kill the channel
    pub fn deep_kill(&self) { self.receiver.deep_kill(); }
    /// Instanciate the receiver
    pub fn instance(&self) -> ArchBroadcastReceiver<U> {
        let Self { ref receiver, phantom } = *self; let receiver = receiver.instance(); ArchBroadcastReceiver { receiver, phantom, }
    }
}
impl RootSerializedDataBroadcastReceiver {
    pub (crate) fn deep_kill(&self) { self.handle_receiver.abort(); }

    pub (crate) fn instance(&self) -> SerializedDataBroadcastReceiver { 
        let Self { ref ref_sender, .. } = *self; let receiver = ref_sender.subscribe(); SerializedDataBroadcastReceiver { receiver, }
    }
}
impl SerializedDataBroadcastSender {
    pub (crate) fn send(&self, value: SerializedData) -> Result<usize, bca::error::SendError<SerializedData>> { self.sender.send(value) }
    pub (crate) fn receiver_count(&self) -> usize { self.sender.receiver_count() }
}
impl SerializedDataBroadcastReceiver {
    pub (crate) async fn recv(&mut self) -> Result<SerializedData, bca::error::RecvError> { self.receiver.recv().await }
    pub (crate) fn try_recv(&mut self) -> Result<SerializedData, bca::error::TryRecvError> { self.receiver.try_recv() }
}
impl<U> ArchBroadcastSender<U> where U: SlxData {
    /// Send archived data
    /// * `value: ArchData<U>` : value to be sent
    /// * Output: the number of receivers of the data or an error
    pub fn send(&self, value: ArchData<U>) -> Result<usize, bca::error::SendError<ArchData<U>>> { 
        match self.sender.send(value.bytes) {
            Ok(u)                      => Ok(u),
            Err(bca::error::SendError(bytes)) => Err(bca::error::SendError(ArchData::from_bytes(bytes)))
        }
    }
    /// Count the number of receivers
    /// * Output: the number of receivers
    pub fn receiver_count(&self) -> usize { self.sender.receiver_count() }
}
impl<U> ArchBroadcastReceiver<U> where U: SlxData {
    /// Receive archived data
    /// * Output: the archived data or an error
    pub async fn recv(&mut self) -> Result<ArchData<U>, bca::error::RecvError> { Ok(ArchData::from_bytes(self.receiver.recv().await?)) }
    /// Try to receive archived data without awaiting
    /// * Output: the archived data or a reception diagnosis
    pub fn try_recv(&mut self) -> Result<ArchData<U>, bca::error::TryRecvError> { Ok(ArchData::from_bytes(self.receiver.try_recv()?)) }
}
