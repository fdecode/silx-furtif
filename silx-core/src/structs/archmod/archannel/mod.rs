use super::{ archdata::{ ArchData, SlxData, }, ser_data::SerializedData, };

/// Signal channel (queueless channel)
mod signal;
pub use signal::{ ArchSignal, ArchSignalReceiver, ArchSignalSender, SerializedDataSignalReceiver, SerializedDataSignalSender, };

/// Oneshot channel (for sending only one information)
mod oneshot;
pub use oneshot::{ SerializedDataOneshot, ArchOneshot, ArchOneshotReceiver, ArchOneshotSender, SerializedDataOneshotReceiver, SerializedDataOneshotSender, };

/// Broadcast channel: many senders send to many receivers and data is cloned for each receiver
mod broadcast;
pub use broadcast::{ ArchBroadcast, 
    RootArchBroadcastReceiver, RootArchBroadcastSender, RootSerializedDataBroadcastReceiver, RootSerializedDataBroadcastSender, 
    ArchBroadcastReceiver, ArchBroadcastSender, SerializedDataBroadcastSender, 
};

/// Dispatch channel: many senders send to many receivers, but only one receiver takes the data
mod dispatch;
pub use dispatch::{ ArchDispatch, ArchDispatchReceiver, ArchDispatchSender, SerializedDataDispatchReceiver, SerializedDataDispatchSender, };

/// Channel for querying data
mod query;
pub use query::{ ArchQuery, ArchQueryReceiver, ArchQuerySender, SerializedDataQueryReceiver, SerializedDataQuerySender, };

/// Channel through socket between two machines
mod socket_channel;
pub use socket_channel::{ ChannelClient, ChannelServer, exp_channel_server, };