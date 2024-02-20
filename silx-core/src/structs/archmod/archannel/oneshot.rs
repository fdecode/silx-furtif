use std::{ task::{ Context, Poll, }, marker::PhantomData, pin::Pin, future::Future, };

use tokio::sync::oneshot as ost;

use super::{ ArchData, SerializedData, SlxData, };

////////////////////////////////////////////
// Oneshot channels

/// Oneshot channel for serialized data within a cluster; oneshot channel is used only once
pub struct SerializedDataOneshot;
impl SerializedDataOneshot {
    pub (crate) fn channel() -> (SerializedDataOneshotSender, SerializedDataOneshotReceiver) {
        let (sender,receiver) = ost::channel();
        (SerializedDataOneshotSender { sender, }, SerializedDataOneshotReceiver { receiver, })
    }
}

/// Oneshot channel for archived data within a cluster; oneshot channel is used only once
pub struct ArchOneshot;
impl ArchOneshot {
    /// Build oneshot channel for archived data
    /// * `U` : type of the data; needs to implement `SlxData`
    /// * Output: a sender and a receiver
    pub fn channel<U>() -> (ArchOneshotSender<U>, ArchOneshotReceiver<U>) where U: SlxData {
        let (sender,receiver) = SerializedDataOneshot::channel();
        let phantom = PhantomData;
        (ArchOneshotSender { sender, phantom, }, ArchOneshotReceiver { receiver, phantom, })
    }
}

#[derive(Debug)]
/// Oneshot sender for serialized data within a cluster
pub struct SerializedDataOneshotSender {
    sender: ost::Sender<SerializedData>,
}
#[derive(Debug)]
/// Oneshot receiver for serialized data within a cluster; this receiver implement Future and can be awaited until data is recieved
pub struct SerializedDataOneshotReceiver {
    receiver: ost::Receiver<SerializedData>,
}
/// Oneshot sender for archived data within a cluster
/// * `U` : type of the data; needs to implement `SlxData`
pub struct ArchOneshotSender<U> where U: SlxData {
    pub (super) sender: SerializedDataOneshotSender,
    phantom: PhantomData<U>,
}
/// Oneshot receiver for archived data within a cluster; this receiver implement Future and can be awaited until data is recieved
/// * `U` : type of the data; needs to implement `SlxData`
pub struct ArchOneshotReceiver<U> where U: SlxData {
    receiver: SerializedDataOneshotReceiver,
    phantom: PhantomData<U>
}

impl SerializedDataOneshotSender {
    pub (crate) fn send(self, t: SerializedData) -> Result<(), SerializedData> { self.sender.send(t) }
    pub (crate) async fn closed(&mut self) { self.sender.closed().await }
    pub (crate) fn is_closed(&self) -> bool { self.sender.is_closed() }
    pub (crate) fn poll_closed(&mut self, cx: &mut Context<'_>) -> Poll<()> { self.sender.poll_closed(cx) }
}
impl SerializedDataOneshotReceiver {
    pub (crate) fn close(&mut self) { self.receiver.close() }
    pub (crate) fn try_recv(&mut self) -> Result<SerializedData, ost::error::TryRecvError> { self.receiver.try_recv() }
    pub (crate) fn blocking_recv(self) -> Result<SerializedData, ost::error::RecvError> { self.receiver.blocking_recv() }
}
impl Future for SerializedDataOneshotReceiver
{
    /// Output of future is the serialized data or error
    type Output = Result<SerializedData, ost::error::RecvError>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> { 
        let receiver = unsafe { self.map_unchecked_mut(|s| &mut s.receiver) };
        receiver.poll(cx) 
    }
}

impl<U> ArchOneshotSender<U> where U: SlxData {
    pub (super) fn from_bytes_sender(sender: SerializedDataOneshotSender,) -> Self { Self{ sender, phantom: PhantomData} }
    /// Send archived data
    /// * `value: ArchData<U>` : archived data to be sent
    pub fn send(self, value: ArchData<U>) -> Result<(), ArchData<U>> { 
        match self.sender.send(value.bytes) {
            Ok(()) => Ok(()), Err(bytes) => Err(ArchData::from_bytes(bytes)),
        } 
    }
    /// Wait the channel to be closed
    pub async fn closed(&mut self) { self.sender.closed().await }
    /// Checks whether the channel is closed
    pub fn is_closed(&self) -> bool { self.sender.is_closed() }
    /// Checks whether the channel has been closed, and if not, schedules the Waker in the provided Context to receive a notification when the channel is closed.
    /// * `cx: &mut Context<'_>` : provided context
    pub fn poll_closed(&mut self, cx: &mut Context<'_>) -> Poll<()> { self.sender.poll_closed(cx) }
}
impl<U> ArchOneshotReceiver<U> where U: SlxData {
    /// Close the receiver
    pub fn close(&mut self) { self.receiver.close() }
    /// Try to receive data
    /// * Output: received data or error
    pub fn try_recv(&mut self) -> Result<ArchData<U>, ost::error::TryRecvError> { Ok(ArchData::from_bytes(self.receiver.try_recv()?)) }
    /// Blocks the thread until receiving data
    /// * Output: received data or error
    pub fn blocking_recv(self) -> Result<ArchData<U>, ost::error::RecvError> { Ok(ArchData::from_bytes(self.receiver.blocking_recv()?)) }
}
impl<U> Future for ArchOneshotReceiver<U> where U: SlxData {
    /// Output of future is the archived data or error
    type Output = Result<ArchData<U>, ost::error::RecvError>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> { 
        let receiver = unsafe { self.map_unchecked_mut(|s| &mut s.receiver) };
        match receiver.poll(cx) {
            Poll::Pending          => Poll::Pending,
            Poll::Ready(Ok(bytes)) => Poll::Ready(Ok(ArchData::from_bytes(bytes))),
            Poll::Ready(Err(e))    => Poll::Ready(Err(e)),
        }
    }
}
