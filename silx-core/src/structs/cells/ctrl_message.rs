use std::time::Duration;

use tokio::sync::{ mpsc as msc, oneshot as osh, };

use crate::{
    shared::id_tools::{ TaskId, AcknowledgeId, }, 
    ChannelIdType, traits::FullId,
    structs::archmod::archannel::{ 
        SerializedDataQuerySender as BinQySender, SerializedDataQueryReceiver as BinQyReceiver, 
        RootSerializedDataBroadcastSender as BinBcSender, RootSerializedDataBroadcastReceiver as BinBcReceiver, 
        SerializedDataSignalSender as BinSgSender, SerializedDataSignalReceiver as BinSgReceiver, 
    },
};


///////////////////
// Main commands

/// doc to be defined
pub enum MsgFromMaster {
    Ctrl (TaskId,CtrlCell),      
}

/// Message from the master of a cluster to a servant following a request
pub enum ReplyToServant {
    /// Aknowledgement message for a task
    Aknowledged(AcknowledgeId,),
}

/// Task request from a servant to the master of a cluster
pub enum MsgFromServant {
    /// Aknowledgement message for a task
    FailureChl(TaskId,ChannelIdType),
    /// Aknowledgement message for a task
    StaledChl(TaskId,ChannelIdType),
    /// Request a new task for shuting down the cluster
    Shutdown(TaskId,),
}

/// doc to be defined
pub enum ReplyToMaster {
    Ok(AcknowledgeId,),
    Undefined(AcknowledgeId,),
    PingFail(AcknowledgeId,),
    Failure(AcknowledgeId,),
    WrongType(AcknowledgeId,),
    OutOfTime(AcknowledgeId,),
}

///////////////////
// Sub-commands

/// doc to be defined
pub enum CtrlCell {
    SetChl(ChannelIdType, SetChannel),
    TurnOnChl(ChannelIdType),
    TurnOffChl(ChannelIdType),
    PingChl(ChannelIdType),
    KillChl(ChannelIdType),
    Kill,
}

#[derive(Clone)]
/// doc to be defined
pub enum SetChannel {
    // Nota: query is multisender to multireceiver
    NetQuerySender { // defining receiver of the channel
        max_ping: Duration,
        query_type: FullId, // input_type is the type of sent data
        reply_type: FullId, // output_type is the type of returned data by oneshot sender
        sender: BinQySender,
    },
    NetQueryReceiver { // defining sender of the channel
        max_ping: Duration,
        query_type: FullId, // input_type is the type of received data
        reply_type: FullId, // output_type is the type of returned data by oneshot sender
        receiver: BinQyReceiver,
    },
    // Nota: broadcast is multisender to multireceiver
    NetBroadcastSender { // defining receiver of the channel
        max_ping: Duration,
        data_type: FullId, // data_type is the type of received data
        sender: BinBcSender,
    },
    NetBroadcastReceiver { // defining sender of the channel
        max_ping: Duration,
        data_type: FullId, // data_type is the type of received data
        receiver: BinBcReceiver,
    },
    // Nota: query is multisender to multireceiver
    QuerySender { // defining receiver of the channel
        max_ping: Duration,
        query_type: FullId, // input_type is the type of sent data
        reply_type: FullId, // output_type is the type of returned data by oneshot sender
        sender: BinQySender,
    },
    QueryReceiver { // defining sender of the channel
        max_ping: Duration,
        query_type: FullId, // input_type is the type of received data
        reply_type: FullId, // output_type is the type of returned data by oneshot sender
        receiver: BinQyReceiver,
    },
    // Nota: broadcast is multisender to multireceiver
    BroadcastSender { // defining receiver of the channel
        max_ping: Duration,
        data_type: FullId, // data_type is the type of received data
        sender: BinBcSender,
    },
    BroadcastReceiver { // defining sender of the channel
        max_ping: Duration,
        data_type: FullId, // data_type is the type of received data
        receiver: BinBcReceiver,
    },
    SignalSender {
        max_ping: Duration,
        data_type: FullId, // data_type is the type of received data
        sender: BinSgSender,
    },
    SignalReceiver {
        max_ping: Duration,
        data_type: FullId, // data_type is the type of received data
        receiver: BinSgReceiver,
    },
}

pub type SenderToMaster = osh::Sender<ReplyToMaster>;

/// Alias for a channel sender from servant to master
pub type SendToMaster = msc::Sender<(MsgFromServant,osh::Sender<ReplyToServant>)>;

pub type RecvFromServant = msc::Receiver<(MsgFromServant,osh::Sender<ReplyToServant>)>;
pub type SendToServant = msc::Sender<(MsgFromMaster,SenderToMaster)>;
pub type RecvFromMaster = msc::Receiver<(MsgFromMaster,SenderToMaster)>;

impl MsgFromServant {
    pub async fn send(self, sender: &SendToMaster,) -> Result<osh::Receiver<ReplyToServant>,String> {
        let (osender,oreceiver,) = osh::channel();
        match sender.send((self,osender)).await {
            Ok(()) => Ok(oreceiver), Err(_) => Err(format!("MsgFromServant: failed to send message")),
        }
    }
}

impl MsgFromMaster {
    pub async fn send(self, sender: &SendToServant,) -> Result<osh::Receiver<ReplyToMaster>,String> {
        let (osender,oreceiver,) = osh::channel();
        match sender.send((self,osender)).await {
            Ok(()) => Ok(oreceiver), Err(_) => Err(format!("MsgFromMaster: failed to send message")),
        }
    }
}