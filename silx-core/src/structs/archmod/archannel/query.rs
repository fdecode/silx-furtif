use std::marker::PhantomData;

use async_channel as asy;

use super::{ ArchData, ArchOneshotSender, SerializedDataOneshotSender, SerializedData, SlxData, };

////////////////////////////////////////////
// Async channels

/// Query channel builder for archived data
/// * Query channel combine dispatch channel for sending the query and oneshot channel for receiving the answer
/// * As a consequence: only one receiver will answer to the query
pub struct ArchQuery;
impl ArchQuery {
    /// Unbounded channel builder for query
    /// * `U` : type of the query data; needs to implement `SlxData`
    /// * `V` : type of the reply data; needs to implement `SlxData`
    /// Output: query sender and receiver
    pub fn unbounded<U,V>() -> (ArchQuerySender<U,V>,ArchQueryReceiver<U,V>) where U: SlxData, V: SlxData, {
        let ((sender,receiver),phantom) = (asy::unbounded(),PhantomData);
        let sender = SerializedDataQuerySender(sender);
        let receiver = SerializedDataQueryReceiver(receiver);
        (ArchQuerySender { sender, phantom, }, ArchQueryReceiver { receiver, phantom, },)
    }
    /// Bounded channel builder for query
    /// * `capacity: usize` : capacity of the channel
    /// * `U` : type of the query data; needs to implement `SlxData`
    /// * `V` : type of the reply data; needs to implement `SlxData`
    /// Output: query sender and receiver
    pub fn bounded<U,V>(capacity:usize) -> (ArchQuerySender<U,V>,ArchQueryReceiver<U,V>) where U: SlxData, V: SlxData, {
        let ((sender,receiver),phantom) = (asy::bounded(capacity),PhantomData);
        let sender = SerializedDataQuerySender(sender);
        let receiver = SerializedDataQueryReceiver(receiver);
        (ArchQuerySender { sender, phantom, }, ArchQueryReceiver { receiver, phantom, },)
    }
}
#[derive(Clone,Debug)]
/// Query sender for serialized data
pub struct SerializedDataQuerySender(asy::Sender<(SerializedData,SerializedDataOneshotSender,)>);
#[derive(Clone,Debug)]
/// Query receiver for serialized data
pub struct SerializedDataQueryReceiver(asy::Receiver<(SerializedData,SerializedDataOneshotSender,)>);
/// Query sender for archived data
/// * `U` : type of the query data; needs to implement `SlxData`
/// * `V` : type of the reply data; needs to implement `SlxData`
pub struct ArchQuerySender<U,V> where U: SlxData, V: SlxData, { // U: query type; V: answer type (oneshot);
    sender: SerializedDataQuerySender,
    phantom: PhantomData<(U,V)>,
}
/// Query receiver for archived data
/// * `U` : type of the query data; needs to implement `SlxData`
/// * `V` : type of the reply data; needs to implement `SlxData`
pub struct ArchQueryReceiver<U,V> where U: SlxData, V:SlxData, { // U: query type; V: answer type (oneshot);
    receiver: SerializedDataQueryReceiver,
    phantom: PhantomData<(U,V)>
}

impl<U,V> Clone for ArchQuerySender<U,V> where U: SlxData, V: SlxData, {
    fn clone(&self) -> Self { 
        let Self { ref sender, phantom } = *self; let sender = sender.clone();
        Self { sender, phantom, }
    }
}
impl<U,V> Clone for ArchQueryReceiver<U,V> where U: SlxData, V: SlxData, {
    fn clone(&self) -> Self {
        let Self { ref receiver, phantom } = *self; let receiver = receiver.clone();
        Self { receiver, phantom, }
    }
}
impl SerializedDataQueryReceiver {
    pub (crate) fn try_recv(&self) -> Result<(SerializedData,SerializedDataOneshotSender,), asy::TryRecvError> { self.0.try_recv() }
    pub (crate) async fn recv(&self) -> Result<(SerializedData,SerializedDataOneshotSender,), asy::RecvError> { self.0.recv().await }
    pub (crate) fn close(&self) -> bool { self.0.close() }
    pub (crate) fn is_closed(&self) -> bool { self.0.is_closed() }
    pub (crate) fn is_empty(&self) -> bool { self.0.is_empty() }
    pub (crate) fn is_full(&self) -> bool { self.0.is_full() }
    pub (crate) fn len(&self) -> usize { self.0.len() }
    pub (crate) fn capacity(&self) -> Option<usize> { self.0.capacity() }
    pub (crate) fn receiver_count(&self) -> usize { self.0.receiver_count() }
    pub (crate) fn sender_count(&self) -> usize { self.0.sender_count() }
}
impl SerializedDataQuerySender {
    pub (crate) fn try_send(&self, msg: (SerializedData,SerializedDataOneshotSender,)) -> Result<(), asy::TrySendError<(SerializedData,SerializedDataOneshotSender,)>> { 
        self.0.try_send(msg) 
    }
    pub (crate) async fn send(&self, msg: (SerializedData,SerializedDataOneshotSender,)) -> Result<(), asy::SendError<(SerializedData,SerializedDataOneshotSender,)>> { 
        self.0.send(msg).await 
    }
    pub (crate) fn close(&self) -> bool { self.0.close() }
    pub (crate) fn is_closed(&self) -> bool { self.0.is_closed() }
    pub (crate) fn is_empty(&self) -> bool { self.0.is_empty() }
    pub (crate) fn is_full(&self) -> bool { self.0.is_full() }
    pub (crate) fn len(&self) -> usize { self.0.len() }
    pub (crate) fn capacity(&self) -> Option<usize> { self.0.capacity() }
    pub (crate) fn receiver_count(&self) -> usize { self.0.receiver_count() }
    pub (crate) fn sender_count(&self) -> usize { self.0.sender_count() }
}

impl<U,V> ArchQueryReceiver<U,V> where U: SlxData, V: SlxData, {
    pub (crate) fn inner(self) -> SerializedDataQueryReceiver { self.receiver }
    /// Try to receive query of archived data without awaiting
    /// * Output: the input archived data and the reply channel, or a reception diagnosis
    pub fn try_recv(&self) -> Result<(ArchData<U>,ArchOneshotSender<V>,), asy::TryRecvError> { 
        let (bytes,sender) = self.receiver.try_recv()?;
        let sender = unsafe { std::mem::transmute(sender) }; // TO CHECK: does this transmute work?
        Ok((ArchData::from_bytes(bytes),sender)) 
    }
    /// Receive query of archived data
    /// * Output: the input archived data and the reply channel, or an error
    pub async fn recv(&self) -> Result<(ArchData<U>,ArchOneshotSender<V>,), asy::RecvError> { 
        let (bytes,sender) = self.receiver.recv().await?;
        let sender = unsafe { std::mem::transmute(sender) }; // TO CHECK: does this transmute work?
        Ok((ArchData::from_bytes(bytes),sender))
    }
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

impl<U,V> ArchQuerySender<U,V> where U: SlxData, V: SlxData, {
    pub (crate) fn inner(self) -> SerializedDataQuerySender { self.sender }
    /// Try to send query of archived data without awaiting
    /// * `(query,oneshot,) : (ArchData<U>, ArchOneshotSender<V>,)` : archived input and reply channel
    /// * Output: nothing or or a sending diagnosis
    pub fn try_send(&self, (query,oneshot,) : (ArchData<U>, ArchOneshotSender<V>,)) -> Result<(), asy::TrySendError<(ArchData<U>,ArchOneshotSender<V>,)>> { 
        match self.sender.try_send((query.bytes, oneshot.sender,)) { Ok(())  => Ok(()),
            Err(asy::TrySendError::Full((bytes,oneshot))) => 
                                                    Err(asy::TrySendError::Full((ArchData::from_bytes(bytes), ArchOneshotSender::from_bytes_sender(oneshot)))),
            Err(asy::TrySendError::Closed((bytes,oneshot))) => 
                                                    Err(asy::TrySendError::Closed((ArchData::from_bytes(bytes), ArchOneshotSender::from_bytes_sender(oneshot)))),
        }
    }
    /// Send query of archived data
    /// * `(query,oneshot,) : (ArchData<U>, ArchOneshotSender<V>,)` : archived input and reply channel
    /// * Output: nothing or an error
    pub async fn send(&self, (query,oneshot,) : (ArchData<U>,ArchOneshotSender<V>,)) -> Result<(), asy::SendError<(ArchData<U>,ArchOneshotSender<V>,)>> {
        match self.sender.send((query.bytes, oneshot.sender,)).await { Ok(())  => Ok(()),
            Err(asy::SendError((bytes,oneshot))) => Err(asy::SendError((ArchData::from_bytes(bytes), ArchOneshotSender::from_bytes_sender(oneshot)))),
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