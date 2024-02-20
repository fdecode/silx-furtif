use std::{ sync::Arc,  net::SocketAddr, collections::{ HashMap, HashSet, }, };

use crate::{
    structs::archmod::archannel::{
        ArchQuerySender, ArchQueryReceiver, RootArchBroadcastSender, RootArchBroadcastReceiver, ArchQuery, ArchBroadcast,
        SerializedDataOneshot, SerializedDataOneshotSender, SerializedDataQuerySender, SerializedDataBroadcastSender,
    },
    net::{ SilxProtocols, NetTaggedSerializedData as NTSData, },
    ChannelIdType, QueryIdType,
    shared::utils::SlxData,
};
use slab::Slab;
use tokio::{
    spawn, task::JoinHandle,
    sync::{ RwLock, Mutex, mpsc, }, 
    net::{ TcpStream, TcpListener, ToSocketAddrs, tcp::OwnedWriteHalf, },
};
use fnv::FnvHashMap;

const VERBOSE: bool = false;

/// doc to be defined
enum MpscSender<T> {
    Bounded(mpsc::Sender<T>),
    Unbounded(mpsc::UnboundedSender<T>),
}
impl<T> MpscSender<T> {
    async fn send(&self, value: T) -> Result<(),String> {
        match self {
            Self::Bounded(sender)   => {
                match sender.send(value).await {
                    Err(_) => Err("Bounded: failed to send".to_string()),
                    Ok(()) => Ok(()),
                }
            },
            Self::Unbounded(sender)   => {
                match sender.send(value) {
                    Err(_) => Err("Unbounded: failed to send".to_string()),
                    Ok(()) => Ok(()),                    
                }
            },
        }
    }
}
impl<T> Clone for MpscSender<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Bounded(sender)   => { Self::Bounded(sender.clone()) },
            Self::Unbounded(sender) => { Self::Unbounded(sender.clone()) },
        }
    }
}

/// doc to be defined
enum MpscReceiver<T> {
    Bounded(mpsc::Receiver<T>),
    Unbounded(mpsc::UnboundedReceiver<T>),
}

impl<T> MpscReceiver<T> {
    async fn recv(&mut self,) -> Option<T> {
        match self {
            Self::Bounded(receiver)   => { receiver.recv().await },
            Self::Unbounded(receiver) =>  { receiver.recv().await },
        }
    }
}

#[derive(Debug)]
/// Channel server
/// * channel server will handle query and broadcast transmissions between two machines
pub struct ChannelServer {
    channels_query: Arc<RwLock<FnvHashMap<ChannelIdType, SerializedDataQuerySender>>>,
    channels_broadcast: Arc<RwLock<FnvHashMap<ChannelIdType, SerializedDataBroadcastSender>>>,
    sockets: HashMap<SocketAddr,(JoinHandle<()>,JoinHandle<()>,)>,
    listener: TcpListener,
    alive: Arc<RwLock<bool>>,
}

#[derive(Debug)]
/// Channel client
/// * channel lient will connect to channel server for query and broadcast transmissions between two machines
pub struct ChannelClient {
    channels_reply: Arc<Mutex<FnvHashMap<ChannelIdType, Slab<SerializedDataOneshotSender>>>>,
    handles: Arc<RwLock<FnvHashMap<ChannelIdType,JoinHandle<()>>>>,
    handle_reply: JoinHandle<()>,
    writer: Arc<Mutex<OwnedWriteHalf>>,
    do_loop: Arc<RwLock<bool>>,
    alive: Arc<RwLock<bool>>,
}

impl ChannelServer {
    /// Build channel server
    /// * `addr: A` : Socket address to be bound to
    /// * `A: ToSocketAddrs` : type of the socket address
    /// * Output: the channel server or an error
    pub async fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self,String,> { // creation du canal (le receiver est le binder)
        let listener = match TcpListener::bind(addr).await {
            Ok(l) => Ok(l),
            Err(_) => Err("Failed to bind".to_string()),
        }?;
        let channels_query = Arc::new(RwLock::new(Default::default()));
        let channels_broadcast = Arc::new(RwLock::new(Default::default()));
        let alive = Arc::new(RwLock::new(true));
        let sockets = HashMap::new();
        Ok(Self { listener, channels_query, channels_broadcast, sockets, alive, })
    }

    /// Kill the server
    pub async fn kill(&mut self) {
        *self.alive.write().await = false;
        for (_,(ref h1,ref h2,)) in self.sockets.iter() { h1.abort(); h2.abort(); }
        for (_,(h1,h2,)) in self.sockets.drain() { let _ = h1.await; let _ = h2.await; }
        self.channels_query.write().await.clear();
        self.channels_broadcast.write().await.clear();
    }

    /// Accept a list of clients
    /// * `clients: &mut HashSet<SocketAddr>` : list of clients identified with their socket address
    /// * `capacity: Option<usize>` : size parameter for defining bounded or unbounnded channels
    /// Nothing or an error
    pub async fn accept(&mut self, clients: &mut HashSet<SocketAddr>, capacity: Option<usize>,) -> Result<(),String> {
        match self.listener.accept().await {
            Ok((mut stream,socket)) => {
                let ok = match SilxProtocols::pop_socket(&mut stream).await {
                    Ok(soa) => clients.remove(&soa),
                    Err(_)  => return Err(format!("Failed to read client id")),
                };
                if !ok { return Err(format!("Connecting client is not allowed")); }
                let (mut reader, mut writer) = stream.into_split();
                let (sender,mut receiver) = match capacity {
                    Some(size) => {
                        let (s,r) = mpsc::channel::<NTSData>(size);
                        (MpscSender::Bounded(s), MpscReceiver::Bounded(r))
                    },
                    None       => {
                        let (s,r) = mpsc::unbounded_channel::<NTSData>();
                        (MpscSender::Unbounded(s), MpscReceiver::Unbounded(r))
                    },
                };
                let alive = self.alive.clone();
                let do_loop = Arc::new(RwLock::new(true));
                let channels_broadcast = self.channels_broadcast.clone();
                let channels_query = self.channels_query.clone();
                let handlein = spawn({ let do_loop = do_loop.clone(); let sender = sender.clone(); async move { 
                    while *alive.read().await && *do_loop.read().await {
                        match SilxProtocols::pop_tagged_serialized_data(&mut reader,).await {
                            Err(st)         => { 
                                if VERBOSE { eprintln!("Input net error: {}", st); }
                                *do_loop.write().await = false; 
                            },
                            Ok(NTSData::Broadcast(channel, data,)) => {
                                match channels_broadcast.read().await.get(&channel) {
                                    None         => { eprintln!("unknown channel -> Broadcast {channel}"); *do_loop.write().await = false; },
                                    Some(brsend) => { match brsend.send(data) {
                                        Err(e)  => eprintln!("Net error: failed to broadcast -> {}",e),
                                        Ok(_)  => (),
                                    } }
                                }
                            },
                            Ok(NTSData::Query(channel, query, data,)) => {
                                match channels_query.read().await.get(&channel) {
                                    None         => { eprintln!("unknown channel -> Query {channel}"); *do_loop.write().await = false; },
                                    Some(brsend) => {
                                        let (ossender,osreceiver) = SerializedDataOneshot::channel();
                                        spawn({ let sender = sender.clone(); async move {
                                            match osreceiver.await {
                                                Err(e)    => eprintln!("Net error: failed to read reply -> {}",e),
                                                Ok(data) => {
                                                    match sender.send(NTSData::Reply(channel,query,data,)).await {
                                                        Ok(()) => (),
                                                        Err(e) => {
                                                            eprintln!("failed to send back data -> {}",e);// *do_loop.write().await = false;
                                                        },
                                                    }
                                                },
                                            };
                                        }});
                                        match brsend.send((data, ossender)).await {
                                            Err(e) => { eprintln!("Net error: failed to send final query -> {}",e) },
                                            Ok(()) => (),
                                        } 
                                    }
                                }
                            },
                            Ok(NTSData::Reply(..)) => { eprintln!("Unexpected reply tag error"); *do_loop.write().await = false; },
                        }
                    } 
                }});
                let alive = self.alive.clone();
                let handleout = spawn({ let do_loop = do_loop.clone(); async move { 
                    while *alive.read().await && *do_loop.read().await {
                        match receiver.recv().await {
                            None                                       => {
                                if VERBOSE && *alive.read().await { eprintln!("output net error: channel closed"); }
                            },
                            Some(NTSData::Reply(channel,query,data,)) => {
                                match SilxProtocols::push_tagged_serialized_data(&mut writer,&NTSData::Reply(channel,query,data,)).await {
                                    Ok(()) => (),
                                    Err(e) => {
                                        eprintln!("output net error: failed to post reply -> {}",e); *do_loop.write().await = false;
                                    },
                                }
                            },
                            Some(_)                                    => {
                                eprintln!("output net error: unexpected Query or Broadcast"); *do_loop.write().await = false;
                            },
                        }
                    } 
                }});
                self.sockets.insert(socket, (handlein,handleout)); Ok(())
            },
            Err(_) => Err("Failed to receive channel connections".to_string()),
        }
    }

    /// Register a query receiver for archived data
    /// * `channel: ChannelIdType` : channel identifier
    /// * `capacity: Option<usize>` : size parameter for defining bounded or unbounnded channels
    /// * `U` : type of the query data; needs to implement `SlxData`
    /// * `V` : type of the reply data; needs to implement `SlxData`
    /// * Output: query receiver for archived data
    pub async fn receiver_query<U,V>(&mut self, channel: ChannelIdType, capacity: Option<usize>,) -> ArchQueryReceiver<U,V>
                                                                                                    where U: SlxData, V: SlxData, {
        let (sender,receiver) = match capacity {
            Some(n) => ArchQuery::bounded::<U,V>(n),
            None    => ArchQuery::unbounded::<U,V>(),
        };
        self.channels_query.write().await.insert(channel,sender.inner()); receiver
    }

    /// Register a broadcast receiver for archived data
    /// * `channel: ChannelIdType` : channel identifier
    /// * `capacity: usize` : capacity of the channel
    /// * `U` : type of the data; needs to implement `SlxData`
    /// * Output: broadcast receiver for archived data
    pub async fn receiver_broadcast<U>(&mut self, channel: ChannelIdType, capacity: usize,) -> RootArchBroadcastReceiver<U> where U: SlxData {
        let (sender,receiver) = ArchBroadcast::channel::<U>(capacity).await;
        self.channels_broadcast.write().await.insert(channel,sender.inner().instance()); receiver
    }
}

impl ChannelClient {

    /// Kill the client
    pub async fn kill(&mut self) {
        *self.alive.write().await = false;
        for (_,h) in self.handles.read().await.iter() { h.abort(); }
        self.handle_reply.abort();
        for (_,h) in self.handles.write().await.drain() { let _ = h.abort(); }
        self.channels_reply.lock().await.clear();
    }

    /// Connect to a channel server and build a channel client
    /// * `id_client: SocketAddr` : socket address of the client
    /// * `addr: A` : socket address of the channel server
    /// * `A: ToSocketAddrs` : type of the server socket address
    /// * Output: the channel client or an error
    pub async fn connect<A: ToSocketAddrs>(id_client: SocketAddr, addr: A) -> Result<Self,String,> { 
        match TcpStream::connect(addr).await {
            Err(e)  => Err(format!("Failed to connect channel client -> {e}")),
            Ok(mut ts)  => {
                if SilxProtocols::push_socket(&mut ts,&id_client).await.is_err() { return Err(format!("failed to send client id")); }
                let alive = Arc::new(RwLock::new(true));
                let do_loop = Arc::new(RwLock::new(true));
                let (mut reader,writer,) = ts.into_split();
                let writer = Arc::new(Mutex::new(writer));
                let channels_reply: Arc<Mutex<FnvHashMap<ChannelIdType, Slab<SerializedDataOneshotSender>>>> = Arc::new(Mutex::new(Default::default()));
                let handles: Arc<RwLock<FnvHashMap<ChannelIdType,JoinHandle<()>>>> = Arc::new(RwLock::new(Default::default()));
                let handle_reply = spawn({
                    let alive = alive.clone(); 
                    let do_loop = do_loop.clone(); let channels_reply = channels_reply.clone(); let handles = handles.clone(); async move {
                        while *alive.read().await && *do_loop.read().await {
                            match SilxProtocols::pop_tagged_serialized_data(&mut reader,).await {
                                Err(st)         => { 
                                    if VERBOSE { eprintln!("Input net error: {}", st); }
                                    *do_loop.write().await = false; 
                                },
                                Ok(NTSData::Reply(channel,query,data,)) => {
                                    match channels_reply.lock().await.get_mut(&channel) {
                                        None         => { eprintln!("unknown channel -> {channel}"); *do_loop.write().await = false; },
                                        Some(slab) => {
                                            match slab.try_remove(query as usize) {
                                                None   => { 
                                                    eprintln!("unknown oneshot reply channel"); *do_loop.write().await = false; 
                                                },
                                                Some(os) => {
                                                    if os.send(data).is_err() {
                                                        eprintln!("net oneshot reply: failed to send data"); // *do_loop.write().await = false; 
                                                    }
                                                },
                                            }
                                        }
                                    }
                                },
                                Ok(NTSData::Broadcast(..)) | Ok(NTSData::Query(..)) => {
                                    eprintln!("Unexpected net teg error"); *do_loop.write().await = false; 
                                },
                            }
                        }
                        channels_reply.lock().await.clear();
                        handles.write().await.clear();
                    }
                });
                Ok(Self { channels_reply, handle_reply, handles, writer, do_loop, alive, })
            }
        }
    }

    /// Register a query sender for archived data
    /// * `channel: ChannelIdType` : channel identifier
    /// * `capacity: Option<usize>` : size parameter for defining bounded or unbounnded channels
    /// * `U` : type of the query data; needs to implement `SlxData`
    /// * `V` : type of the reply data; needs to implement `SlxData`
    /// * Output: query sender for archived data
    pub async fn sender_query<U,V>(&mut self, channel: ChannelIdType, capacity: Option<usize>,) -> ArchQuerySender<U,V>
                                                                                                where U: SlxData, V: SlxData, {
        let (sender,receiver) = match capacity {
            None    => ArchQuery::unbounded::<U,V>(),
            Some(n) => ArchQuery::bounded::<U,V>(n),
        };
        let receiver = receiver.inner();
        let channels_reply = self.channels_reply.clone();
        channels_reply.lock().await.insert(channel,Slab::new());
        let handle = spawn({ let alive = self.alive.clone(); let do_loop = self.do_loop.clone(); let writer = self.writer.clone(); async move {
            while *alive.read().await && *do_loop.read().await {
                match receiver.recv().await {
                    Ok((data,ossender)) => {
                        match channels_reply.lock().await.get_mut(&channel) {
                            Some(slab) => {
                                let query = slab.insert(ossender) as QueryIdType;
                                let writer = &mut *writer.lock().await;
                                let nts_dat = NTSData::Query(channel,query,data,);
                                if SilxProtocols::push_tagged_serialized_data(writer, &nts_dat).await.is_err() {
                                    eprintln!("failed to push query {} on channel {}",query,channel); *do_loop.write().await = false;
                                }
                            },
                            None       => {
                                eprintln!("failed to find slab for channel {}",channel); *do_loop.write().await = false; 
                            },
                        }
                    },
                    Err(_)               => {
                        if VERBOSE { eprintln!("failed to receive from channel {}", channel); }
                        *do_loop.write().await = false; 
                    },
                }
            } 
        }});
        self.handles.write().await.insert(channel,handle); sender
    }

    /// Register a broadcast sender for archived data
    /// * `channel: ChannelIdType` : channel identifier
    /// * `capacity: usize` : capacity of the channel
    /// * `U` : type of the data; needs to implement `SlxData`
    /// * Output: broadcast sender for archived data
    pub async fn sender_broadcast<U>(&mut self, channel: ChannelIdType, capacity: usize,) -> RootArchBroadcastSender<U> where U: SlxData {
        let (sender,receiver) = ArchBroadcast::channel::<U>(capacity).await;
        let mut receiver = receiver.inner().instance();
        let handle = spawn({ let alive = self.alive.clone(); let do_loop = self.do_loop.clone(); let writer = self.writer.clone(); async move {
            while *alive.read().await && *do_loop.read().await {
                match receiver.recv().await {
                    Ok(data) => {
                        let writer = &mut *writer.lock().await;
                        let nts_dat = NTSData::Broadcast(channel,data,);
                        match SilxProtocols::push_tagged_serialized_data(writer, &nts_dat).await {
                            Ok(()) => (),
                            Err(e) => {
                                eprintln!("net error: failed to post broadcast -> {}",e); *do_loop.write().await = false;
                            },
                        }
                    },
                    Err(_)               => {
                        if VERBOSE { eprintln!("failed to receive from channel {}", channel); }
                        *do_loop.write().await = false; 
                    },
                }
            }
        }});
        self.handles.write().await.insert(channel,handle); sender
    }

}

/// Experimentation example of channel server
pub async fn exp_channel_server() {
    use std::{ pin::Pin, str::FromStr, };
    use crate::{
        structs::archmod::archannel::ArchOneshot,
        shared::utils::ArchSized,
    };

    let size = 11024;
    let osize = Some(size);
    let mut channel_server = ChannelServer::bind("127.0.0.1:8085").await.expect("fails to open channel server");
    println!("binded channel server");
    let id_client = SocketAddr::from_str("1.2.3.4:5678").expect("failed to parse socket address");
    let mut clients: HashSet<_> = [
        id_client.clone(), 
        SocketAddr::from_str("4.2.3.21:58").expect("failed to parse socket address"), 
    ].into_iter().collect();
    let mut channel_client = ChannelClient::connect(id_client.clone(),"127.0.0.1:8085",).await.expect("fails to open channel client");
    println!("connected channel client");
    channel_server.accept(&mut clients,osize,).await.expect("failed to accept");

    // root channels
    let root_broadcast_out = channel_server.receiver_broadcast::<[u32;3]>(123,size).await;
    let root_broadcast_in = channel_client.sender_broadcast::<[u32;3]>(123,size).await;
    // active channels
    let query_out1 = channel_server.receiver_query::<[u32;5],[u32;5]>(113,osize).await;
    let query_in1 = channel_client.sender_query::<[u32;5],[u32;5]>(113,osize).await;
    let query_in2 = query_in1.clone();
    let query_out2 = query_out1.clone();
    let mut broadcast_out1 = root_broadcast_out.instance();
    let broadcast_in1 = root_broadcast_in.instance();
    let mut broadcast_out2 = root_broadcast_out.instance();
    let broadcast_in2 = root_broadcast_in.instance();

    // broadcast
    println!("=== broadcast ======================");
    let bytes = [1u32,2,3].arch_sized().expect("failed to serialize");
    let _ = broadcast_in1.send(bytes);
    let bytes = [4u32,2,11].arch_sized().expect("failed to serialize");
    let _ = broadcast_in2.send(bytes);
    let bytes = [3u32,1,4].arch_sized().expect("failed to serialize");
    let _ = broadcast_in1.send(bytes);
    println!("data sent");
    let arch_data = broadcast_out1.recv().await.expect("Failed to read bytes");
    let bytes = arch_data.archive_ref().expect("fail to get archive ref");
    println!("received bytes = {:?}",bytes);
    let arch_data = broadcast_out2.recv().await.expect("Failed to read bytes");
    let bytes = arch_data.archive_ref().expect("fail to get archive ref");
    println!("received bytes = {:?}",bytes);
    let arch_data = broadcast_out1.recv().await.expect("Failed to read bytes");
    let bytes = arch_data.archive_ref().expect("fail to get archive ref");
    println!("received bytes = {:?}",bytes);
    let arch_data = broadcast_out2.recv().await.expect("Failed to read bytes");
    let bytes = arch_data.archive_ref().expect("fail to get archive ref");
    println!("received bytes = {:?}",bytes);
    let arch_data = broadcast_out1.recv().await.expect("Failed to read bytes");
    let bytes = arch_data.archive_ref().expect("fail to get archive ref");
    println!("received bytes = {:?}",bytes);
    let arch_data = broadcast_out2.recv().await.expect("Failed to read bytes");
    let bytes = arch_data.archive_ref().expect("fail to get archive ref");
    println!("received bytes = {:?}",bytes);
   
    // query
    println!("=== query ==========================");
    let (sender1,receiver1) = ArchOneshot::channel();
    let (sender2,receiver2) = ArchOneshot::channel();
    let (sender3,receiver3) = ArchOneshot::channel();
    let bytes = [9u32,28,14,7,1].arch_sized().expect("failed to serialize");
    let _ = query_in1.send((bytes,sender1)).await;
    let bytes = [5u32,16,8,4,2].arch_sized().expect("failed to serialize");
    let _ = query_in2.send((bytes,sender2)).await;
    let bytes = [4u32,3,2,1,0].arch_sized().expect("failed to serialize");
    let _ = query_in1.send((bytes,sender3)).await;
    println!("query sent");
    let (bytes,sender1) = query_out1.recv().await.expect("Failed to read bytes");
    let mut rbytes = bytes.clone(); Pin::new(&mut rbytes).archive_mut().expect("fail to get archive ref").reverse();
    sender1.send(rbytes).expect("failed to send reply 1");
    println!("received bytes 1 = {:?}",bytes.archive_ref().expect("fail to get archive ref"));
    let (bytes,sender2) = query_out2.recv().await.expect("Failed to read bytes");
    let mut rbytes = bytes.clone(); Pin::new(&mut rbytes).archive_mut().expect("fail to get archive ref").reverse();
    sender2.send(rbytes).expect("failed to send reply 2");
    println!("received bytes 2 = {:?}",bytes.archive_ref().expect("fail to get archive ref"));
    let (bytes,sender3) = query_out1.recv().await.expect("Failed to read bytes");
    let mut rbytes = bytes.clone(); Pin::new(&mut rbytes).archive_mut().expect("fail to get archive ref").reverse();
    sender3.send(rbytes).expect("failed to send reply 3");
    println!("received bytes 3 = {:?}",bytes.archive_ref().expect("fail to get archive ref"));
    println!("query received and reply sent");
    
    let arch_data = receiver1.await.expect("failed to receive reply");
    let bytes = arch_data.archive_ref().expect("fail to get archive ref");
    println!("received reply 1 = {:?}",bytes);
    let arch_data = receiver3.await.expect("failed to receive reply");
    let bytes = arch_data.archive_ref().expect("fail to get archive ref");
    println!("received reply 3 = {:?}",bytes);
    let arch_data = receiver2.await.expect("failed to receive reply");
    let bytes = arch_data.archive_ref().expect("fail to get archive ref");
    println!("received reply 2 = {:?}",bytes);
    println!("reply received");

    channel_client.kill().await;
    println!("channel client killed");
    channel_server.kill().await;
    println!("channel server killed");
}

