use std::{ marker::PhantomData, sync::Arc, };

use tokio::sync::watch as wtc;

use super::{ ArchData, SerializedData, SlxData, };


////////////////////////////////////////////
// Signal channels

/// Archived signal channel within a cluster: signal is an history free 'instantaneous' data (queueless channel)
pub struct ArchSignal;
impl ArchSignal {
    /// Archived signal channel builder
    /// * `U` : type of the archived data; needs to implement `SlxData`
    /// * Output: sender and receiver
    pub fn channel<U>() -> (ArchSignalSender<U>,ArchSignalReceiver<U>) where U: SlxData {
        let (sender,receiver) = wtc::channel(SerializedData::undefined()); // initialize with undefined data
        let sender = SerializedDataSignalSender { sender: Arc::new(sender), }; 
        let receiver = SerializedDataSignalReceiver { receiver, };
        let phantom = PhantomData;
        (ArchSignalSender { sender, phantom, }, ArchSignalReceiver { receiver, phantom, })
    }
}


#[derive(Debug)]
/// Signal sender for serialized data; serialized data does not contain type information
pub struct SerializedDataSignalSender {
    sender: Arc<wtc::Sender<SerializedData>>,
}
#[derive(Debug)]
/// Signal receiver for serialized data; serialized data does not contain type information
pub struct SerializedDataSignalReceiver {
    receiver: wtc::Receiver<SerializedData>,
}
/// Signal sender for archived data; archived data are serialized data with type information
/// * `U` : type of the data; eeds to implement `SlxData`
pub struct ArchSignalSender<U> {
    sender: SerializedDataSignalSender,
    phantom: PhantomData<U>,
}
/// Signal receiver for archived data; archived data are serialized data with type information
/// * `U` : type of the data; eeds to implement `SlxData`
pub struct ArchSignalReceiver<U> {
    receiver: SerializedDataSignalReceiver,
    phantom: PhantomData<U>
}

impl Clone for SerializedDataSignalSender {
    fn clone(&self) -> Self {
        let Self { sender, } = self; let sender = sender.clone(); Self { sender, }
    }
}
impl Clone for SerializedDataSignalReceiver {
    fn clone(&self) -> Self {
        let Self { receiver, } = self; let receiver = receiver.clone(); Self { receiver, }        
    }
}
impl<U> Clone for ArchSignalSender<U> where U: SlxData {
    fn clone(&self) -> Self {
        let Self { ref sender, phantom } = *self; let sender = sender.clone(); Self { sender, phantom, }        
    }
}
impl<U> Clone for ArchSignalReceiver<U> where U: SlxData {
    fn clone(&self) -> Self {
        let Self { ref receiver, phantom } = *self; let receiver = receiver.clone(); Self { receiver, phantom, }        
    }
}
impl SerializedDataSignalSender {
    pub (crate) fn send(&self, value: SerializedData) -> Result<(), wtc::error::SendError<SerializedData>> { self.sender.send(value) }
    pub (crate) fn send_replace(&self, value: SerializedData) -> Option<SerializedData> { // Option None means that replaced data is undefined (that is: empty)
        let av = self.sender.send_replace(value);
        if av.is_undefined() { None } else { Some(av) }
    }
    pub (crate) fn borrow(&self) -> Option<SerializedData> { 
        let bor = self.sender.borrow();
        if bor.is_undefined() { None } else { Some(bor.clone()) }
    }
    pub (crate) fn is_closed(&self) -> bool { self.sender.is_closed() }
    pub (crate) async fn closed(&self) { self.sender.closed().await }
    pub (crate) fn subscribe(&self) -> SerializedDataSignalReceiver { let receiver = self.sender.subscribe(); SerializedDataSignalReceiver { receiver, } }
    pub (crate) fn receiver_count(&self) -> usize { self.sender.receiver_count() }
}
impl SerializedDataSignalReceiver {
    pub (crate) fn borrow(&self) -> Option<SerializedData> { 
        let bor = self.receiver.borrow();
        if bor.is_undefined() { None } else { Some(bor.clone()) }
    }
    pub (crate) fn borrow_and_update(&mut self) -> Option<SerializedData> { 
        let bor = self.receiver.borrow_and_update();
        if bor.is_undefined() { None } else { Some(bor.clone()) }
    }
    pub (crate) fn has_changed(&self) -> Result<bool, wtc::error::RecvError> { self.receiver.has_changed() }
    pub (crate) async fn changed(&mut self) -> Result<(), wtc::error::RecvError> { self.receiver.changed().await }
}
impl<U> ArchSignalSender<U> where U: SlxData {
    pub(crate) fn inner(self) -> SerializedDataSignalSender { self.sender }

    /// Send archived data
    /// * `value: ArchData<U>` : archived data to be sent
    pub fn send(&self, value: ArchData<U>) -> Result<(), wtc::error::SendError<ArchData<U>>> { 
        match self.sender.send(value.bytes) {
            Ok(())  => Ok(()),
            Err(wtc::error::SendError(bytes)) => Err(wtc::error::SendError(ArchData::from_bytes(bytes))),
        }
    }
    /// Send archived data by replacing previous data
    /// * `value: ArchData<U>` : archived data to be sent
    /// * Output: previous data if it exists
    pub fn send_replace(&self, value: ArchData<U>) -> Option<ArchData<U>> { 
        match self.sender.send_replace(value.bytes) {
            None => None, Some(data) => Some(ArchData::from_bytes(data)),
        }
    }
    /// Get a clone of sent data (not a borrow at this time of implementation)
    /// * Output: sent data if it exists
    pub fn borrow(&self) -> Option<ArchData<U>> { 
        match self.sender.borrow() {
            None => None, Some(data) => Some(unsafe { std::mem::transmute(data) }),
        }         
    }
    /// Check if channel is closed
    pub fn is_closed(&self) -> bool { self.sender.is_closed() }
    /// Wait channel to be closed
    pub async fn closed(&self) { self.sender.closed().await }
    /// Create a receiver connected to this sender
    /// * Output: a receiver
    pub fn subscribe(&self) -> ArchSignalReceiver<U> { let receiver = self.sender.subscribe(); ArchSignalReceiver { receiver, phantom: PhantomData, } }
    /// Number of receivers connected to this sender
    /// * Output: number of receivers
    pub fn receiver_count(&self) -> usize { self.sender.receiver_count() }
}

impl<U> ArchSignalReceiver<U> where U: SlxData {
    pub(crate) fn inner(self) -> SerializedDataSignalReceiver { self.receiver }
    /// Get a clone of sent data (not a borrow at this time of implementation)
    /// * Output: sent data if it exists
    pub fn borrow(&self) -> Option<ArchData<U>> { 
        match self.receiver.borrow() {
            None => None, Some(data) => Some(unsafe { std::mem::transmute(data) }),
        }         
    }
    /// Get a clone of sent data (not a borrow at this time of implementation) and mark this data as seen
    /// * Output: sent data if it exists
    pub fn borrow_and_update(&mut self) -> Option<ArchData<U>> { 
        match self.receiver.borrow_and_update() {
            None => None, Some(data) => Some(unsafe { std::mem::transmute(data) }),
        }         
    }
    /// Check if the channel contains a new signal 
    pub fn has_changed(&self) -> Result<bool, wtc::error::RecvError> { self.receiver.has_changed() }
    /// Wait the channel to contain a new signal 
    pub async fn changed(&mut self) -> Result<(), wtc::error::RecvError> { self.receiver.changed().await }
}
