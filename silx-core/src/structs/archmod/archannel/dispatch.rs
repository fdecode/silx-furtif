use std::marker::PhantomData;

use async_channel as asy;

use super::{ ArchData, SerializedData, SlxData, };

////////////////////////////////////////////
// Async channels

/// Dispatch channel builder for archived data
/// * Dispatch means that many senders send to many receivers, but only one receiver takes the data
pub struct ArchDispatch;
impl ArchDispatch {
    /// Unbounded channel builder for dispatch
    /// * `U` : type of the data; needs to implement `SlxData`
    /// * Output: dispatch sender and receiver
    pub fn unbounded<U>() -> (ArchDispatchSender<U>,ArchDispatchReceiver<U>) where U: SlxData {
        let ((sender,receiver),phantom) = (asy::unbounded(),PhantomData);
        let sender = SerializedDataDispatchSender(sender);
        let receiver = SerializedDataDispatchReceiver(receiver);
        (ArchDispatchSender { sender, phantom, }, ArchDispatchReceiver {receiver, phantom, },)
    }
    /// Bounded channel builder for dispatch
    /// * `capacity: usize` : capacity of the channel
    /// * `U` : type of the data; needs to implement `SlxData`
    /// * Output: dispatch sender and receiver
    pub fn bounded<U>(capacity: usize) -> (ArchDispatchSender<U>,ArchDispatchReceiver<U>) where U: SlxData {
        let ((sender,receiver),phantom) = (asy::bounded(capacity),PhantomData);
        let sender = SerializedDataDispatchSender(sender);
        let receiver = SerializedDataDispatchReceiver(receiver);
        (ArchDispatchSender { sender, phantom, }, ArchDispatchReceiver {receiver, phantom, },)
    }
}
#[derive(Clone,Debug)]
/// Dispatch sender for serialized data
pub struct SerializedDataDispatchSender(asy::Sender<SerializedData>);
#[derive(Clone,Debug)]
/// Dispatch receiver for serialized data
pub struct SerializedDataDispatchReceiver(asy::Receiver<SerializedData>);
/// Dispatch sender for archived data
/// * `U` : type of the data; needs to implement `SlxData`
pub struct ArchDispatchSender<U> where U: SlxData {
    sender: SerializedDataDispatchSender,
    phantom: PhantomData<U>,
}
/// Dispatch receiver for archived data
/// * `U` : type of the data; needs to implement `SlxData`
pub struct ArchDispatchReceiver<U> where U: SlxData {
    receiver: SerializedDataDispatchReceiver,
    phantom: PhantomData<U>
}

impl<U> Clone for ArchDispatchSender<U> where U: SlxData {
    fn clone(&self) -> Self { 
        let Self { ref sender, phantom } = *self; let sender = sender.clone();
        Self { sender, phantom, }
    }
}
impl<U> Clone for ArchDispatchReceiver<U> where U: SlxData {
    fn clone(&self) -> Self {
        let Self { ref receiver, phantom } = *self; let receiver = receiver.clone();
        Self { receiver, phantom, }
    }
}
impl SerializedDataDispatchReceiver {
    pub (crate) fn try_recv(&self) -> Result<SerializedData, asy::TryRecvError> { self.0.try_recv() }
    pub (crate) async fn recv(&self) -> Result<SerializedData, asy::RecvError> { self.0.recv().await }
    pub (crate) fn close(&self) -> bool { self.0.close() }
    pub (crate) fn is_closed(&self) -> bool { self.0.is_closed() }
    pub (crate) fn is_empty(&self) -> bool { self.0.is_empty() }
    pub (crate) fn is_full(&self) -> bool { self.0.is_full() }
    pub (crate) fn len(&self) -> usize { self.0.len() }
    pub (crate) fn capacity(&self) -> Option<usize> { self.0.capacity() }
    pub (crate) fn receiver_count(&self) -> usize { self.0.receiver_count() }
    pub (crate) fn sender_count(&self) -> usize { self.0.sender_count() }
}
impl SerializedDataDispatchSender {
    pub (crate) fn try_send(&self, msg: SerializedData) -> Result<(), asy::TrySendError<SerializedData>> { self.0.try_send(msg) }
    pub (crate) async fn send(&self, msg: SerializedData) -> Result<(), asy::SendError<SerializedData>> { self.0.send(msg).await }
    pub (crate) fn close(&self) -> bool { self.0.close() }
    pub (crate) fn is_closed(&self) -> bool { self.0.is_closed() }
    pub (crate) fn is_empty(&self) -> bool { self.0.is_empty() }
    pub (crate) fn is_full(&self) -> bool { self.0.is_full() }
    pub (crate) fn len(&self) -> usize { self.0.len() }
    pub (crate) fn capacity(&self) -> Option<usize> { self.0.capacity() }
    pub (crate) fn receiver_count(&self) -> usize { self.0.receiver_count() }
    pub (crate) fn sender_count(&self) -> usize { self.0.sender_count() }
}
impl<U> ArchDispatchReceiver<U> where U: SlxData {
    pub(crate) fn inner(self) -> SerializedDataDispatchReceiver { self.receiver }
    /// Try to receive archived data without awaiting
    /// * Output: the archived data or a reception diagnosis
    pub fn try_recv(&self) -> Result<ArchData<U>, asy::TryRecvError> { Ok(ArchData::from_bytes(self.receiver.try_recv()?)) }
    /// Receive archived data
    /// * Output: the archived data or an error
    pub async fn recv(&self) -> Result<ArchData<U>, asy::RecvError> { Ok(ArchData::from_bytes(self.receiver.recv().await?)) }
    /// Close the channel: returns true if this call has closed the channel and it was not closed already
    /// * Output: a boolean
    pub fn close(&self) -> bool { self.receiver.close() }
    /// Is the channel closed?
    /// * Output: a boolean
    pub fn is_closed(&self) -> bool { self.receiver.is_closed() }
    /// Is the channel empty?
    /// * Output: a boolean
    pub fn is_empty(&self) -> bool { self.receiver.is_empty() }
    /// Is the channel full?
    /// * Output: a boolean
    pub fn is_full(&self) -> bool { self.receiver.is_full() }
    /// Number of messages in the channel
    /// * Output: the number of messages
    pub fn len(&self) -> usize { self.receiver.len() }
    /// Capacity of the channel
    /// * Output: the capacity if defined; otherwise the channel is unsized
    pub fn capacity(&self) -> Option<usize> { self.receiver.capacity() }
    /// Count the receivers
    /// * Output: number of receivers
    pub fn receiver_count(&self) -> usize { self.receiver.receiver_count() }
    /// Count the senders
    /// * Output: number of senders
    pub fn sender_count(&self) -> usize { self.receiver.sender_count() }
}
impl<U> ArchDispatchSender<U> where U: SlxData {
    pub(crate) fn inner(self) -> SerializedDataDispatchSender { self.sender }
    /// Try to send archived data
    /// * `value: ArchData<U>` : value to be sent
    /// * Output: nothing or or a sending diagnosis
    pub fn try_send(&self, value: ArchData<U>) -> Result<(), asy::TrySendError<ArchData<U>>> { 
        match self.sender.try_send(value.bytes) { Ok(())  => Ok(()),
            Err(asy::TrySendError::Full(bytes)) => Err(asy::TrySendError::Full(ArchData::from_bytes(bytes))),
            Err(asy::TrySendError::Closed(bytes)) => Err(asy::TrySendError::Closed(ArchData::from_bytes(bytes))),
        }
    }
    /// Send archived data
    /// * `value: ArchData<U>` : value to be sent
    /// * Output: nothing or an error
    pub async fn send(&self, value: ArchData<U>) -> Result<(), asy::SendError<ArchData<U>>> {
        match self.sender.send(value.bytes).await { Ok(())  => Ok(()),
            Err(asy::SendError(bytes)) => Err(asy::SendError(ArchData::from_bytes(bytes))),
        }
    }
    /// Close the channel: returns true if this call has closed the channel and it was not closed already
    /// * Output: a boolean
    pub fn close(&self) -> bool { self.sender.close() }
     /// Is the channel closed?
    /// * Output: a boolean
    pub fn is_closed(&self) -> bool { self.sender.is_closed() }
    /// Is the channel empty?
    /// * Output: a boolean
    pub fn is_empty(&self) -> bool { self.sender.is_empty() }
    /// Is the channel full?
    /// * Output: a boolean
    pub fn is_full(&self) -> bool { self.sender.is_full() }
    /// Number of messages in the channel
    /// * Output: the number of messages
    pub fn len(&self) -> usize { self.sender.len() }
    /// Capacity of the channel
    /// * Output: the capacity if defined; otherwise the channel is unsized
    pub fn capacity(&self) -> Option<usize> { self.sender.capacity() }
    /// Count the receivers
    /// * Output: number of receivers
    pub fn receiver_count(&self) -> usize { self.sender.receiver_count() }
    /// Count the senders
    /// * Output: number of senders
    pub fn sender_count(&self) -> usize { self.sender.sender_count() }
}