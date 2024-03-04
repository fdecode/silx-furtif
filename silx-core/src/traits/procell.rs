use crate::{ 
    ChannelIdType, 
    traits::FullId,
    shared::utils::{ SendToMaster, terminated, SlxData, },
    structs::{
        cells::components::{ Mapper, Assert, },
        archmod::{ 
            ser_data::SerializedData,
            archdata::ArchData, 
            archannel::{ SerializedDataDispatchReceiver, SerializedDataDispatchSender, ArchDispatch, ArchDispatchReceiver, ArchDispatchSender, }, 
        },
    },
};
use std::{ future::Future, sync::Arc, collections::HashMap, pin::Pin, };

use hashed_type_def::HashedTypeDef;

use fnv::FnvHashMap;
use tokio::{ spawn, sync::Mutex, task::JoinHandle, }; 


#[derive(Clone,)]
/// doc to be defined
pub enum ProcessSignature {
    Query { 
        in_type: FullId,
        out_type: FullId,
    },
    Reply { 
        in_type: FullId,
        out_type: FullId,
    },
    Emit {
        in_type: FullId,
    },
    Read { 
        out_type: FullId,
    },
    RefRead { 
        out_type: FullId,
    },
}

#[derive(Clone,)]
/// doc to be defined
pub enum ProcessType {
    Query { 
        in_chan: SerializedDataDispatchReceiver,
        in_type: FullId,
        out_chan: SerializedDataDispatchSender,
        out_type: FullId,
    },
    Reply { 
        in_type: FullId,
        out_type: FullId,
        mapper: Mapper,
    },
    Emit {
        in_chan: SerializedDataDispatchReceiver,
        in_type: FullId,
    },
    Read { 
        out_chan: SerializedDataDispatchSender,
        out_type: FullId,
    },
    RefRead { 
        out_type: FullId,
        assert: Assert,
    },
}

/// doc to be defined
pub (crate) trait HasNamedProcess where Self: __seal__::Sealer {
    fn processes(&self,) -> &HashMap<String, (FullId, Option<FullId>, ProcessType)>;

    fn activate(&mut self);
    
    fn signature(&self) -> &HashMap<String, ProcessSignature>;
}

/// doc to be defined
pub trait HasProcess where Self: __seal__::Sealer {
    // Get processer for a given channel
    fn process(&self, channel: ChannelIdType,) -> Option<ProcessType>;
}

/// doc to be defined
struct NamedProcess<D, F> where F: Future<Output=()> + Send + 'static {
    #[allow(dead_code)]
    data: D,
    future: Option<F>,
    #[allow(dead_code)]
    handle: Option<JoinHandle<()>>,
    _send_to_master: SendToMaster,  // necessary so as to not close channel illegaly
    processes: HashMap<String, (FullId, Option<FullId>, ProcessType)>,
    signature: HashMap<String, ProcessSignature>,
}

#[derive(Clone)]
/// doc to be defined
pub struct ProcessCell {
    #[allow(dead_code)]
    named_process: Arc<Mutex<Box<dyn HasNamedProcess + Send>>>,
    map_process: FnvHashMap<ChannelIdType,ProcessType>,
}

/// Process instance for a servant
pub struct ProcessInstance(pub (crate) Box<dyn HasNamedProcess + Send>);

/// Process producer which manages the construction of a servant instance
pub struct ProcessProducer {
    send_to_master: SendToMaster,
    processes: HashMap<String, (FullId, Option<FullId>, ProcessType)>,
}

impl ProcessProducer {
    /// Constructor of a process producer
    /// * `send_to_master: &SendToMaster` : a channel sender from the servant to the master of the cluster
    /// * Output: the process producer
    pub fn new(send_to_master: &SendToMaster,) -> Self { 
        let send_to_master = send_to_master.clone(); let processes = Default::default(); Self { send_to_master, processes, } 
    }
    fn get_signature(processes: &HashMap<String, (FullId, Option<FullId>, ProcessType)>,) -> HashMap<String, ProcessSignature> {
        processes.iter().map(|(s,(_,_,ref pt))| (s.clone(), match pt {
            ProcessType::Query { in_type: i, out_type: o, .. } => ProcessSignature::Query { in_type: i.clone(), out_type: o.clone(), },
            ProcessType::Reply { in_type: i, out_type: o, .. } => ProcessSignature::Reply { in_type: i.clone(), out_type: o.clone(), },
            ProcessType::Emit { in_type: i, .. }               => ProcessSignature::Emit { in_type: i.clone(), }, 
            ProcessType::Read { out_type: o, .. }              => ProcessSignature::Read { out_type: o.clone(), },
            ProcessType::RefRead { out_type: o, .. }           => ProcessSignature::RefRead { out_type: o.clone(), },
        })).collect()
    }

    /// Generate an empty process instance
    /// * Output: a process instance with a future which is always pending (see the definition of `terminated()`)
    pub fn named_process(self,) -> ProcessInstance {        
        let Self { send_to_master, processes } = self; 
        let future = Some(terminated());
        let signature = Self::get_signature(&processes);
        ProcessInstance(Box::new(NamedProcess { _send_to_master: send_to_master, data:(), future, handle:None, signature, processes, }))
    }

    /// Generate an empty process instance with data
    /// * `data: D` : a data
    /// * `D` : type of the data
    /// * Output: a process instance with data and a future which is always pending (see the definition of `terminated()`)
    pub fn named_process_with_data<D>(self, data: D,) -> ProcessInstance where D: 'static + Send, {        
        let Self { send_to_master, processes } = self; 
        let future = Some(terminated());
        let signature = Self::get_signature(&processes);
        ProcessInstance(Box::new(NamedProcess { _send_to_master: send_to_master, data, future, handle:None, signature, processes, }))
    }

    /// Generate a process instance for a given future
    /// * `future: F` : a future to be run by the process instance
    /// * `F` : type of the future
    /// * Output: a process instance wrapping the given future
    pub fn named_process_with_future<F>(self, future: F,) -> ProcessInstance where F: Future<Output = ()> + Send + 'static, {        
        let Self { send_to_master, processes } = self;
        let future = Some(future);
        let signature = Self::get_signature(&processes);
        ProcessInstance(Box::new(NamedProcess { _send_to_master: send_to_master, data: (), future, handle: None, signature, processes, }))
    }

    /// Generate a process instance for a given future
    /// * `data: D` : a data
    /// * `future: F` : a future to be run by the process instance
    /// * `D` : type of the data
    /// * `F` : type of the future
    /// * Output: a process instance wrapping the given future with data
    pub fn named_process_with_data_future<D,F>(self, data: D, future: F,) -> ProcessInstance 
                                where D: 'static + Send, F: Future<Output = ()> + Send + 'static, {        
        let Self { send_to_master, processes } = self; 
        let future = Some(future);
        let signature = Self::get_signature(&processes);
        ProcessInstance(Box::new(NamedProcess { _send_to_master: send_to_master, data, future, handle: None, signature, processes, }))
    }

    /// Add a reply-to-query component of type 2 to the process producer
    /// * `name_channel: &String` : name of the query channel
    ///   * channels connected to the servant necessarily have different names
    /// * `f: F` : closure processing the reply
    /// * `U` : type of the query; needs to implement `SlxData`
    /// * `V` : type of the reply; needs to implement `SlxData`
    /// * `F` : type of the processing closure; needs to implement `Fn(&'static mut ArchData<U>) -> Pin<Box<dyn Future<Output = ArchData<V> > + Send>>` 
    /// * Output: nothing or an error
    pub fn add_reply2<U,V,F>(&mut self, name_channel: &String, f: F) -> Result<(),String>
            where F: Fn(&'static mut ArchData<U>) -> Pin<Box<dyn Future<Output = ArchData<V> > + Send>> + Clone + Send + Sync + 'static, 
                  U: 'static + SlxData + Send, V: 'static + SlxData, {
        let name = name_channel.clone();
        let name_u = U::UUID; let name_v = V::UUID;
        let process_type = ProcessType::Reply{ in_type: name_u.clone(), out_type: name_v.clone(),
            mapper: Arc::new(move |bytes: &'static mut SerializedData,| {
                let addr = bytes as *mut SerializedData;
                let bytes = unsafe{ addr.as_mut::<'static>() }.expect("unexpected error: *mut SerializedData to &mut SerializedData");
                let addr = addr  as *mut ArchData<U>; 
                let arch = unsafe { addr.as_mut::<'static>() }.expect("unexpected error: *mut SerializedData to &mut SerializedData");
                let f = f.clone();
                Box::pin( async move {
                    let mut result = f(arch).await.to_bytes();
                    std::mem::swap(bytes, &mut result);
                } )
            } ), 
        };
        if self.processes.insert(name, (name_u,Some(name_v),process_type)).is_none() { Ok(()) } 
        else { Err(format!("Duplicate channel name: {}", name_channel)) } 
    }

    /// Add a reply-to-query component of type 1 to the process producer
    /// * `name_channel: &String` : name of the query channel
    ///   * channels connected to the servant necessarily have different names
    /// * `f: F` : closure processing the reply
    /// * `U` : type of the query and of reply; needs to implement `SlxData`
    /// * `F` : type of the processing closure; needs to implement `Fn(&'static mut ArchData<U>) -> Pin<Box<dyn Future<Output = ArchData<V> > + Send>>` 
    /// * Output: nothing or an error
    pub fn add_reply1<U,F>(&mut self, name_channel: &String, f: F) -> Result<(),String>
            where F: Fn(Pin<&'static mut ArchData<U>>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static,
                  U: 'static + SlxData {
        let name = name_channel.clone();
        let name_u = U::UUID;
        let mapper = Arc::new(move |bytes: &'static mut SerializedData,| f(unsafe { std::mem::transmute(Pin::new(bytes)) }));
        let process_type = ProcessType::Reply{ in_type: name_u.clone(), out_type: name_u.clone(), mapper, };
        if self.processes.insert(name, (name_u.clone(),Some(name_u),process_type)).is_none() { Ok(()) } 
        else { Err(format!("Duplicate channel name: {}", name_channel)) } 
    }

    /// Add a query-and-get-reply component to the process producer
    /// * `name_channel: &String` : name of the query channel
    ///   * channels connected to the servant necessarily have different names
    /// * `capacity: Option<usize>` : capacity of the channel (`None` for unlimited)
    /// * `U` : type of the query; needs to implement `SlxData`
    /// * `V` : type of the reply; needs to implement `SlxData`
    /// * Output: a dispatch sender for the query and a dispatch receiver for the reply or an error
    pub fn add_query<U,V,>(&mut self, name_channel: &String, capacity: Option<usize>,) -> Result<(ArchDispatchSender<U>, ArchDispatchReceiver<V>),String>
            where U: SlxData, V: SlxData,  { 
        let name = name_channel.clone();
        let name_u = U::UUID; let name_v = V::UUID;
        let (usender, ureceiver) = if let Some(capacity) = capacity { ArchDispatch::bounded::<U>(capacity) } else { ArchDispatch::unbounded::<U>() };
        let (vsender, vreceiver) = if let Some(capacity) = capacity { ArchDispatch::bounded::<V>(capacity) } else { ArchDispatch::unbounded::<V>() };
        let in_chan = ureceiver.inner();
        let out_chan = vsender.inner();
        let process_type = ProcessType::Query{ in_chan, in_type: name_u.clone(), out_chan, out_type: name_v.clone(), };
        if self.processes.insert(name, (name_u,Some(name_v),process_type)).is_none() { Ok((usender,vreceiver)) } 
        else { Err(format!("Duplicate channel name: {}", name_channel)) } 
    }

    /// Add an emit component to the process producer
    /// * `name_channel: &String` : name of the emitting channel
    ///   * channels connected to the servant necessarily have different names
    /// * `capacity: Option<usize>` : capacity of the channel (`None` for unlimited)
    /// * `U` : type of the emitted data; needs to implement `SlxData`
    /// * Output: a dispatch sender for emitting or an error
    pub fn add_emit<U,>(&mut self, name_channel: &String, capacity: Option<usize>,) -> Result<ArchDispatchSender<U>,String> where U: SlxData, { 
        let name = name_channel.clone();
        let name_u = U::UUID;
        let (usender, ureceiver) = if let Some(capacity) = capacity { ArchDispatch::bounded::<U>(capacity) } else { ArchDispatch::unbounded::<U>() };
        let in_chan = ureceiver.inner();
        let process_type = ProcessType::Emit{ in_chan, in_type: name_u.clone(), };
        if self.processes.insert(name, (name_u,None,process_type)).is_none() { Ok(usender) } 
        else { Err(format!("Duplicate channel name: {}", name_channel)) } 
    }

    /// Add a read component to the process producer
    /// * `name_channel: &String` : name of the reading channel
    ///   * channels connected to the servant necessarily have different names
    /// * `capacity: Option<usize>` : capacity of the channel (`None` for unlimited)
    /// * `V` : type of the read data; needs to implement `SlxData`
    /// * Output: a dispatch receiver for reading or an error
    pub fn add_read<V,>(&mut self, name_channel: &String, capacity: Option<usize>,) -> Result<ArchDispatchReceiver<V>,String> where V: SlxData, {
        let name = name_channel.clone();
        let name_v = V::UUID;
        let (vsender, vreceiver) = if let Some(capacity) = capacity { ArchDispatch::bounded::<V>(capacity) } else { ArchDispatch::unbounded::<V>() };
        let out_chan = vsender.inner();
        let process_type = ProcessType::Read{ out_chan, out_type: name_v.clone(), };
        if self.processes.insert(name, (name_v,None,process_type)).is_none() { Ok(vreceiver) } 
        else { Err(format!("Duplicate channel name: {}", name_channel)) } 
    }

    /// Add a read-by-reference component to the process producer; the data referenceis then processed by a closure
    /// * `name_channel: &String` : name of the read-by-reference channel
    /// * `reader: F` : a reader for processing the data reference
    ///   * channels connected to the servant necessarily have different names
    /// * `V` : type of the read data; needs to implement `SlxData`
    /// * `F` : type of the reading closure; needs to implement `Fn(&'static ArchData<V>) -> Pin<Box<dyn Future<Output = ()> + Send>>`
    /// * Output: nothing or an error
    pub fn add_ref_read<V,F,>(&mut self, name_channel: &String, reader: F,) -> Result<(),String>
            where F: Fn(&'static ArchData<V>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static, V: 'static + SlxData, { 
        let name = name_channel.clone();                
        let name_v = <V as HashedTypeDef>::UUID;
        let assert = Arc::new(move |bytes: &SerializedData,| reader(unsafe { std::mem::transmute(bytes) }));
        let process_type = ProcessType::RefRead{ out_type: name_v.clone(), assert, };
        if self.processes.insert(name, (name_v,None,process_type)).is_none() { Ok(()) }
        else { Err(format!("Duplicate channel name: {}", name_channel)) }
    }
}

impl<D,F> __seal__::Sealer for NamedProcess<D,F> where F: Future<Output=()> + Send + 'static {}
impl<D,F> HasNamedProcess for NamedProcess<D,F> where F: Future<Output=()> + Send + 'static {
    fn activate(&mut self) {
        let mut tmp = None;
        std::mem::swap(&mut tmp, &mut self.future);
        if let Some(future) = tmp { self.handle = Some(spawn(future)); }
    }
    fn signature(&self) -> &HashMap<String, ProcessSignature> { &self.signature }
    fn processes(&self,) -> &HashMap<String, (FullId, Option<FullId>, ProcessType)> { &self.processes } 
}

impl ProcessCell {
    pub (crate) fn new(mut named_process: Box<dyn HasNamedProcess + Send>, map_name: &FnvHashMap<ChannelIdType,String>,) -> Option<Self> {
        let omap_process = {
            named_process.activate();
            let processes = named_process.processes();
            let compliant = map_name.values().all(|v| processes.contains_key(v));
            if compliant {
                let map_process: FnvHashMap<ChannelIdType,ProcessType> = map_name.iter()
                    .map(|(&channel,name)| (channel, processes.get(name).expect("unexpected error").2.clone())).collect();
                Some(map_process)
            } else { None }
        };
        if let Some(map_process) = omap_process {
            let named_process = Arc::new(Mutex::new(named_process));
            Some(Self { named_process, map_process, })    
        } else { None }
    }
}
impl __seal__::Sealer for ProcessCell {}
impl HasProcess for ProcessCell {
    fn process(&self, channel: ChannelIdType,) -> Option<ProcessType>  { self.map_process.get(&channel).cloned() }
}


mod __seal__ {
    pub trait Sealer {}
}