use rkyv::AlignedVec;
use tokio::io::{ AsyncReadExt, AsyncWriteExt, };
use num_enum::{ IntoPrimitive, TryFromPrimitive, };
use std::{ net::SocketAddr, str::FromStr, };
use crate::{ ChannelIdType, QueryIdType, structs::archmod::ser_data::{ SerializedData, SerializedDataType, }, };

#[derive(IntoPrimitive, TryFromPrimitive,)]
#[repr(u8)]
/// doc to be defined
enum NetCase {
    Socket = 0,
    String,
    SizedBroadcast,
    SizedQuery,
    SizedReply,
}

#[derive(Debug,)]
/// doc to be defined
pub(crate) enum NetTag {
    Socket,
    String,
    SizedBroadcast(ChannelIdType,u32,),
    SizedQuery(ChannelIdType, QueryIdType,u32,),
    SizedReply(ChannelIdType, QueryIdType,u32,),
}

#[derive(Debug,)]
/// doc to be defined
pub(crate) enum NetTaggedSerializedData {
    Broadcast(ChannelIdType, SerializedData,),
    Query(ChannelIdType, QueryIdType, SerializedData,),
    Reply(ChannelIdType, QueryIdType,  SerializedData,),
}

/// doc to be defined
pub(crate) enum SilxProtocols {}

impl SilxProtocols {
    async fn push<W: AsyncWriteExt + Unpin> (mut writer: W, tag: NetTag, data: &[u8]) -> Result<(),String> {
        match tag {
            NetTag::Socket                           => {
                if writer.write_u8(NetCase::Socket.into()).await.is_err() { return Err("Failed to write case".to_string()); }
                if writer.write_u32_le(data.len() as u32).await.is_err() { return Err("Failed to write bytes length".to_string()); }
                if writer.write_all(data).await.is_err() { return Err("Failed to write bytes".to_string()); }   
            },
            NetTag::String                           => {
                if writer.write_u8(NetCase::String.into()).await.is_err() { return Err("Failed to write case".to_string()); }
                if writer.write_u32_le(data.len() as u32).await.is_err() { return Err("Failed to write bytes length".to_string()); }
                if writer.write_all(data).await.is_err() { return Err("Failed to write bytes".to_string()); }   
            },
            NetTag::SizedBroadcast(channel,root)     => {
                if writer.write_u8(NetCase::SizedBroadcast.into()).await.is_err() { return Err("Failed to write case".to_string()); }
                if writer.write_u64_le(channel).await.is_err() { return Err("Failed to write channel".to_string()); }
                if writer.write_u32_le(root).await.is_err() { return Err("Failed to write root".to_string()); }
                if writer.write_u32_le(data.len() as u32).await.is_err() { return Err("Failed to write bytes length".to_string()); }
                if writer.write_all(data).await.is_err() { return Err("Failed to write bytes".to_string()); }   
            },
            NetTag::SizedQuery(channel,query,root)   => {
                if writer.write_u8(NetCase::SizedQuery.into()).await.is_err() { return Err("Failed to write case".to_string()); }
                if writer.write_u64_le(channel).await.is_err() { return Err("Failed to write channel".to_string()); }
                if writer.write_u64_le(query).await.is_err() { return Err("Failed to write query".to_string()); }
                if writer.write_u32_le(root).await.is_err() { return Err("Failed to write root".to_string()); }
                if writer.write_u32_le(data.len() as u32).await.is_err() { return Err("Failed to write bytes length".to_string()); }
                if writer.write_all(data).await.is_err() { return Err("Failed to write bytes".to_string()); }   
            },
            NetTag::SizedReply(channel,query,root)   => {
                if writer.write_u8(NetCase::SizedReply.into()).await.is_err() { return Err("Failed to write case".to_string()); }
                if writer.write_u64_le(channel).await.is_err() { return Err("Failed to write channel".to_string()); }
                if writer.write_u64_le(query).await.is_err() { return Err("Failed to write query".to_string()); }
                if writer.write_u32_le(root).await.is_err() { return Err("Failed to write root".to_string()); }
                if writer.write_u32_le(data.len() as u32).await.is_err() { return Err("Failed to write bytes length".to_string()); }
                if writer.write_all(data).await.is_err() { return Err("Failed to write bytes".to_string()); }   
            },
        };
        Ok(())
    } 
    async fn pop<R: AsyncReadExt + Unpin>(mut reader: R) -> Result<(NetTag,AlignedVec),String> {
        let cas: NetCase = match reader.read_u8().await { 
            Ok(cas) => match NetCase::try_from(cas) { Ok(cas) => cas, _ => return Err("Unknown case".to_string()), },
            _ => return Err("Failed to read case".to_string()), 
        };
        let tag = match cas {
            NetCase::Socket           => NetTag::Socket,
            NetCase::String           => NetTag::String,
            NetCase::SizedBroadcast   => {
                let channel = match reader.read_u64_le().await { Ok(c) => c, _ => return Err("Failed to read channel".to_string()), };
                let root = match reader.read_u32_le().await { Ok(root) => root, _ => return Err("Failed to read root".to_string()), };
                NetTag::SizedBroadcast(channel,root)
            },
            NetCase::SizedQuery       => {
                let channel = match reader.read_u64_le().await { Ok(c) => c, _ => return Err("Failed to read channel".to_string()), };
                let query = match reader.read_u64_le().await { Ok(c) => c, _ => return Err("Failed to read query".to_string()), };
                let root = match reader.read_u32_le().await { Ok(root) => root, _ => return Err("Failed to read root".to_string()), };
                NetTag::SizedQuery(channel,query,root)
            },
            NetCase::SizedReply       => {
                let channel = match reader.read_u64_le().await { Ok(c) => c, _ => return Err("Failed to read channel".to_string()), };
                let query = match reader.read_u64_le().await { Ok(c) => c, _ => return Err("Failed to read query".to_string()), };
                let root = match reader.read_u32_le().await { Ok(root) => root, _ => return Err("Failed to read root".to_string()), };
                NetTag::SizedReply(channel,query,root)
            },
        };
        let len = match reader.read_u32_le().await {
            Ok(len) => len,
            _       => return Err("Failed to read bytes length".to_string()),
        };
        let len: usize = len as usize;
        let mut buffer = AlignedVec::with_capacity(len);
        unsafe { buffer.set_len(len); }
        match reader.read_exact(&mut buffer).await {
            Ok(_)  => (),
            Err(_) => { return Err("Failed to read bytes".to_string()); },
        }
        Ok((tag,buffer,))
    }

    pub(crate) async fn push_socket<W: AsyncWriteExt + Unpin>(mut writer: W, socket: &SocketAddr) -> Result<(),String> {
        let str_s = socket.to_string();
        let bytes = str_s.as_bytes();
        SilxProtocols::push(&mut writer, NetTag::Socket, bytes).await
    }
    pub(crate) async fn pop_socket<R: AsyncReadExt + Unpin>(mut reader: R) -> Result<SocketAddr,String> {
        let bytes = match SilxProtocols::pop(&mut reader).await {
            Ok((NetTag::Socket,bytes,)) => bytes,
            Ok(dat)                     => return Err(format!("Wrong net data : {:?}", dat)),
            Err(e)                      => return Err(e),
        };
        match String::from_utf8(bytes.iter().cloned().collect()) {
            Ok(s)  => {
                match SocketAddr::from_str(&s) {
                    Ok(soa) => Ok(soa),
                    Err(_)  => Err("Failed to parse into socket address".to_string()),
                }
            },
            Err(_) => return Err("Found invalid UTF-8".to_string()), 
        }

    }

    pub(crate) async fn push_string<W: AsyncWriteExt + Unpin>(mut writer: W, s: &str) -> Result<(),String> {
        let bytes = s.as_bytes();
        SilxProtocols::push(&mut writer, NetTag::String, bytes).await
    }
    pub(crate) async fn pop_string<R: AsyncReadExt + Unpin>(mut reader: R) -> Result<String,String> {
        let bytes = match SilxProtocols::pop(&mut reader).await {
            Ok((NetTag::String,bytes,)) => bytes,
            Ok(dat)                     => return Err(format!("Wrong net data : {:?}", dat)),
            Err(e)                      => return Err(e),
        };
        match String::from_utf8(bytes.iter().cloned().collect()) {
            Ok(s)  => Ok(s),
            Err(_) => return Err("Found invalid UTF-8".to_string()), 
        }
    }

    pub(crate) async fn push_tagged_serialized_data<W: AsyncWriteExt + Unpin>(mut writer: W, s: &NetTaggedSerializedData) -> Result<(),String> {
        use NetTaggedSerializedData::{ Broadcast as NBroadcast, Query as NQuery, Reply as NReply, };
        use SilxProtocols as sp;
        type S = SerializedData;
        use SerializedDataType::Sized;
        use NetTag::{ SizedBroadcast, SizedQuery, SizedReply, };
        match s {
            NBroadcast(channel, S { data_type: Sized, root, data, },)      => sp::push(&mut writer, SizedBroadcast(*channel,*root), data).await,
            NQuery(channel, query, S { data_type: Sized, root, data, },)   => sp::push(&mut writer, SizedQuery(*channel,*query, *root), data).await,
            NReply(channel, query, S { data_type: Sized, root, data, },)   => sp::push(&mut writer, SizedReply(*channel,*query, *root), data).await,
            _ => Err(format!("Undefined data: forbidden!")),
        }
    }
    pub(crate) async fn pop_tagged_serialized_data<R: AsyncReadExt + Unpin>(mut reader: R) -> Result<NetTaggedSerializedData,String> {
        use NetTaggedSerializedData::{ Broadcast as NBroadcast, Query as NQuery, Reply as NReply, };
        match SilxProtocols::pop(&mut reader).await {
            Ok((NetTag::SizedBroadcast(channel,root),bytes,))     => Ok(NBroadcast(channel, SerializedData::sized_from(root, bytes,),)),
            Ok((NetTag::SizedQuery(channel,query,root),bytes,))   => Ok(NQuery(channel, query, SerializedData::sized_from(root, bytes,),)),
            Ok((NetTag::SizedReply(channel,query,root),bytes,))   => Ok(NReply(channel, query, SerializedData::sized_from(root, bytes,),)),
            Ok((NetTag::Socket,_))                                    => Err(format!("Wrong net data : NetTag::Socket")),
            Ok((NetTag::String,_))                                    => Err(format!("Wrong net data : NetTag::String")),
            Err(e)                                                => Err(e),
        }
    }
}