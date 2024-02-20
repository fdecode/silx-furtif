use std::{ 
    collections::{ BTreeMap, HashMap, BTreeSet, }, net::SocketAddr, iter::once, path::{ Path, PathBuf, }, time::Duration, 
};
use tokio::sync::mpsc as msc;
use crate::{
    traits::{ procell::ProcessSignature as ps, FullId, },
    shared::id_tools::TaskIdGenerator,
    builder::{ FiledClusterBuilder, Channel, },
    shared::utils::{ FiledStarter, Filed, RecFiled, ServantBuilder, }
};

type SB = Box<dyn ServantBuilder>;

/// Builder for silx network, including clusters definitions only
pub struct StarterProducer {
    main: SocketAddr,
    // addr -> (path_starter, path_builder, net_capacity, crl_capacity,)
    clusters: BTreeMap<SocketAddr,(PathBuf,PathBuf,Option<usize>,usize,)>, 
}

/// Builder for silx network starter, including clusters definitions with servants names and builder files
pub struct StarterProducerWithProcesses {
    main: SocketAddr,
    servants: BTreeSet<String>,
    clusters: BTreeMap<SocketAddr,(PathBuf,PathBuf,Option<usize>,usize,BTreeMap<String,(HashMap<String, ps>,PathBuf,SB)>)>,
}

/// Builder for silx network starter, including clusters definitions with: servants names and  builder files; data channels names, types and builder files
pub struct StarterProducerWithFlow {
    main: SocketAddr,
    clusters: BTreeMap<SocketAddr,(PathBuf,PathBuf,Option<usize>,usize,BTreeMap<String,(HashMap<String, ps>,PathBuf,SB)>)>,
    flow: BTreeMap<String,(PathBuf,Channel)>,
}

impl StarterProducer {
    /// Constructor of initial starters builder of the whole network: only contains main cluster information
    /// * `main: SocketAddr` : socket address of main cluster
    /// * `path_starter: P` : path of the file where main cluster starter is serialized
    /// * `path_builder: Q` : path of the file where main cluster builder is serialized
    /// * `net_capacity: Option<usize>` : capacity of network channel (`None` for unlimited)
    /// * `ctrl_capacity: usize` : capacity of control channels between the servants and the master of the cluster
    /// * `P` : type of path
    /// * `Q` : type of path
    /// * Output: the initial starter builder
    pub fn new<P,Q>(main: SocketAddr, path_starter: P, path_builder: Q, 
                    net_capacity: Option<usize>, ctrl_capacity: usize,) -> Self where P: AsRef<Path>, Q: AsRef<Path>, {
        let path_starter = path_starter.as_ref().to_path_buf();
        let path_builder = path_builder.as_ref().to_path_buf();
        let clusters: BTreeMap<_,_> = once((main, (path_starter, path_builder, net_capacity, ctrl_capacity,))).collect();
        Self { main, clusters, }                
    }
    /// Add a new cluster to starters builder
    /// * `addr: SocketAddr` : socket address of added cluster
    /// * `path_starter: P` : path of the file where the cluster starter is serialized
    /// * `path_builder: Q` : path of the file where the cluster builder is serialized
    /// * `net_capacity: Option<usize>` : capacity of network channel (`None` for unlimited)
    /// * `ctrl_capacity: usize` : capacity of control channels between the servants and the master of the cluster
    /// * `P` : type of path
    /// * `Q` : type of path
    /// * Output: completed starter builder or error
    pub fn add_cluster<P,Q>(mut self, addr: SocketAddr, path_starter: P, path_builder: Q, 
                    net_capacity: Option<usize>, ctrl_capacity: usize,) -> Result<Self,String> where P: AsRef<Path>, Q: AsRef<Path>, {
        let path_starter = path_starter.as_ref().to_path_buf();
        let path_builder = path_builder.as_ref().to_path_buf();
        if self.clusters.insert(addr,(path_starter, path_builder, net_capacity, ctrl_capacity,)).is_some() { 
            Err(format!("Address used twice for clusters")) }
        else { Ok(self) }
    }
    /// Finalize the starters builder in order to proceed next to servants additions
    /// * Output: starter builder ready for servants additions
    pub fn done(self) -> StarterProducerWithProcesses {
        let Self { main, clusters, } = self;
        let clusters = clusters.into_iter().map(|(k,(a,b,c,d,))| (k,(a,b,c,d,BTreeMap::new()))).collect();
        let servants = BTreeSet::new();
        StarterProducerWithProcesses { main, clusters, servants, }
    } 
}

impl StarterProducerWithProcesses {
    /// Add a new servant to starters builder
    /// * `cluster: SocketAddr` : socket address of the cluster to which the servant is added
    /// * `name: String` : name of the servant
    /// * `path: P` : path of the file where servant builder is serialized
    /// * `builder: B` : builder of the servant
    /// * `B` : type of servant builder
    /// * `P` : type of path
    /// * Output: completed starter builder or error
    pub fn add_process<B,P,>(mut self, cluster: &SocketAddr, name: String, path: P, builder: B,) -> Result<Self,String>
            where B: 'static + ServantBuilder, P: AsRef<Path> {
        if !self.servants.insert(name.clone()) { return Err(format!("Servant name {} is multiply defined",name)) }
        let task_id = TaskIdGenerator::new(); 
        let names_chan: HashMap<String, ps> = { 
            let (sender, receiver) = msc::channel(1);
            let names = builder.build_process(task_id,sender).0.signature().clone();
            let _ = receiver; names
        };
        let pathed_servant: (_,PathBuf,SB) =(names_chan,path.as_ref().to_path_buf(), Box::new(builder));
        if let Some(rbt) = self.clusters.get_mut(cluster) { 
            if rbt.4.insert(name,pathed_servant).is_some() { panic!("unexpected error"); } Ok(self)
        } else { Err(format!("Cluster address has not been entered")) }
    }
    /// Finalize the starters builder in order to proceed next to channels additions
    /// * Output: starter builder ready for channels additions
    pub fn done(self) -> StarterProducerWithFlow {
        let Self { main, clusters, .. } = self;
        let flow = BTreeMap::new();
        StarterProducerWithFlow { main, clusters, flow, }
    } 
}

impl StarterProducerWithFlow {
    /// Add a new intracluster query channel to starters builder
    /// * `path: P` : path of the file where channel is serialized
    /// * `name: String` : name of the channel
    /// * `cluster: SocketAddr` : socket address of the cluster in which the channel operates
    /// * `in_names: I` : collection of querying servants of the cluster
    /// * `out_names: O` : collection of replying servants of the cluster
    /// * `max_ping: Duration` : max ping duration for the channel
    /// * `size: Option<usize>` : size of the channel (`None` for unlimited)
    /// * `P` : type of path
    /// * `I` : type of the collection of querying servants
    /// * `O` : type of the collection of replying servants
    /// * Output: completed starter builder or error
    pub fn add_query<P,I,O>(mut self, path: P, name: String, cluster: SocketAddr, 
                        in_names: I, out_names: O, max_ping: Duration, size: Option<usize>,) -> Result<Self,String> 
                                                        where P: AsRef<Path>, I: IntoIterator<Item=String>, O: IntoIterator<Item=String> {
        let (input,output,query_type, reply_type) = self.get_query_sign(&name, &cluster, &cluster, in_names, out_names,)?;
        let channel = Channel::Query { max_ping, size, query_type, reply_type, cluster, input, output, };
        self.flow.insert(name,(path.as_ref().to_path_buf(),channel)); Ok(self)
    }
    /// Add a new intracluster broadcast channel to starters builder
    /// * `path: P` : path of the file where channel is serialized
    /// * `name: String` : name of the channel
    /// * `cluster: SocketAddr` : socket address of the cluster in which the channel operates
    /// * `in_names: I` : collection of querying servants of the cluster
    /// * `out_names: O` : collection of replying servants of the cluster
    /// * `max_ping: Duration` : max ping duration for the channel
    /// * `size: usize` : size of the channel
    /// * `P` : type of path
    /// * `I` : type of the collection of querying servants
    /// * `O` : type of the collection of replying servants
    /// * Output: completed starter builder or error
    pub fn add_broadcast<P,I,O>(mut self, path: P, name: String, cluster: SocketAddr, 
                    in_names: I, out_names: O, max_ping: Duration, size: usize,) -> Result<Self,String>
                                                        where P: AsRef<Path>, I: IntoIterator<Item=String>, O: IntoIterator<Item=String> {
        let (input,output,data_type) = self.get_broadcast_sign(&name, &cluster, &cluster, in_names, out_names,)?;
        let channel = Channel::Broadcast { max_ping, size, data_type, cluster, input, output, };
        self.flow.insert(name,(path.as_ref().to_path_buf(),channel)); Ok(self)
    }
    /// Add a new signal channel (intracluster) to starters builder
    /// * `path: P` : path of the file where channel is serialized
    /// * `name: String` : name of the channel
    /// * `cluster: SocketAddr` : socket address of the cluster in which the channel operates
    /// * `in_names: I` : collection of querying servants of the cluster
    /// * `out_names: O` : collection of replying servants of the cluster
    /// * `max_ping: Duration` : max ping duration for the channel
    /// * `P` : type of path
    /// * `I` : type of the collection of querying servants
    /// * `O` : type of the collection of replying servants
    /// * Output: completed starter builder or error
    pub fn add_signal<P,I,O>(mut self, path: P, name: String, cluster: SocketAddr,
                    in_names: I, out_names: O, max_ping: Duration,) -> Result<Self,String>
                                                        where P: AsRef<Path>, I: IntoIterator<Item=String>, O: IntoIterator<Item=String> {
        let (input,output,data_type) = self.get_signal_sign(&name, &cluster, &cluster, in_names, out_names,)?;
        let channel = Channel::Signal { max_ping, data_type, cluster, input, output, };
        self.flow.insert(name,(path.as_ref().to_path_buf(),channel)); Ok(self)
    }
    /// Add a new intercluster query channel to starters builder
    /// * `path: P` : path of the file where channel is serialized
    /// * `name: String` : name of the channel
    /// * `in_cluster: SocketAddr` : socket address of the cluster from which the channel operates
    /// * `in_names: I` : collection of querying servants of the input cluster
    /// * `out_cluster: SocketAddr` : socket address of the cluster to which the channel operates
    /// * `out_names: O` : collection of replying servants of the output cluster
    /// * `max_ping: Duration` : max ping duration for the channel
    /// * `size: Option<usize>` : size of the channel (`None` for unlimited)
    /// * `P` : type of path
    /// * `I` : type of the collection of querying servants
    /// * `O` : type of the collection of replying servants
    /// * Output: completed starter builder or error
    pub fn add_net_query<P,I,O>(mut self, path: P, name: String, in_cluster: SocketAddr, 
                    in_names: I, out_cluster: SocketAddr, out_names: O, max_ping: Duration, size: Option<usize>,) -> Result<Self,String>
                                                        where P: AsRef<Path>, I: IntoIterator<Item=String>, O: IntoIterator<Item=String> {
        let (input,output,query_type, reply_type) = self.get_query_sign(&name, &in_cluster, &out_cluster, in_names, out_names,)?;
        let input = (in_cluster,input); let output = (out_cluster,output);
        let channel = Channel::NetQuery { max_ping, size, query_type, reply_type, input, output, };
        self.flow.insert(name,(path.as_ref().to_path_buf(),channel)); Ok(self)
    }
    /// Add a new intercluster broadcast channel to starters builder
    /// * `path: P` : path of the file where channel is serialized
    /// * `name: String` : name of the channel
    /// * `in_cluster: SocketAddr` : socket address of the cluster from which the channel operates
    /// * `in_names: I` : collection of querying servants of the input cluster
    /// * `out_cluster: SocketAddr` : socket address of the cluster to which the channel operates
    /// * `out_names: O` : collection of replying servants of the output cluster
    /// * `max_ping: Duration` : max ping duration for the channel
    /// * `size: usize` : size of the channel
    /// * `P` : type of path
    /// * `I` : type of the collection of querying servants
    /// * `O` : type of the collection of replying servants
    /// * Output: completed starter builder or error
    pub fn add_net_broadcast<P,I,O>(mut self, path: P, name: String, in_cluster: SocketAddr, 
                    in_names: I, out_cluster: SocketAddr, out_names: O, max_ping: Duration, size: usize,) -> Result<Self,String> 
                                                        where P: AsRef<Path>, I: IntoIterator<Item=String>, O: IntoIterator<Item=String> {
        let (input,output,data_type) = self.get_broadcast_sign(&name, &in_cluster, &out_cluster, in_names, out_names,)?;
        let input = (in_cluster,input); let output = (out_cluster,output);
        let channel = Channel::NetBroadcast { max_ping, size, data_type, input, output, };
        self.flow.insert(name,(path.as_ref().to_path_buf(),channel)); Ok(self)
    }
    /// Finalize the starters builder and get the list of starters definitions
    /// * Output: a collection which maps from cluster socket address to starter definition 
    pub fn done(self) -> HashMap<SocketAddr,RecFiled<FiledStarter>> {
        let Self { main, clusters, flow, } = self;
        let mut result: HashMap<_,_> = clusters.iter().filter_map(|(&this,(path,..))| {
            if this == main { None } else { Some((this,RecFiled::new_partially_loaded(path.clone(),FiledStarter::Listener { main, this, }))) }
        }).collect();
        let path = clusters.get(&main).expect("unexpected error").0.clone();
        let builders: BTreeMap<SocketAddr,RecFiled<FiledClusterBuilder>> = clusters.into_iter().map(|(s,(_,p,nc,cc,btm))|{
            let named_servant = btm.into_iter().map(|(s,(_,p,sb))| (s, Filed::new_loaded(p,sb))).collect();
            (s, RecFiled::new_partially_loaded(p,FiledClusterBuilder { net_size: nc, ctrl_ch_capacity: cc, named_servants: named_servant }))
        }).collect();
        let flow = flow.into_iter().map(|(s,(p,c))| (s, Filed::new_loaded(p,c))).collect();
        let main_starter = RecFiled::new_partially_loaded(path, FiledStarter::Main { builders, flow, main, });
        result.insert(main, main_starter); result
    } 
    /// Finalize the starters builder and get the list of starters definitions
    /// * Some coherence tests are done
    /// * Output: a collection which maps from cluster socket address to starter definition 
    pub fn done_right(self) -> Result<HashMap<SocketAddr,RecFiled<FiledStarter>>,String> {
        // completeness test
        if !self.clusters.iter().flat_map(|(_,(..,m))| m.iter().map(|(_,(b,..))|b)).all(|b|b.is_empty()) { 
            return Err(format!("Some servant connectors are not used"));
        }
        Ok(self.done())
    } 

    fn get_query_sign<I,O>(&mut self, name: &str, in_addr: &SocketAddr, out_addr: &SocketAddr, in_names: I, out_names: O,) 
            -> Result<(BTreeSet<String>,BTreeSet<String>,FullId,FullId),String> where I: IntoIterator<Item=String>, O: IntoIterator<Item=String> {
        let input: BTreeSet<_> = in_names.into_iter().collect();
        let output: BTreeSet<_> = out_names.into_iter().collect();
        if input.is_empty() { return Err(format!("Producer - Flow: input is empty!")); }
        if output.is_empty() { return Err(format!("Producer - Flow: output is empty!")); }
        let mut in_out: Option<(FullId,FullId)> = None;
        match self.clusters.get_mut(in_addr) {
            None => return Err(format!("Producer - Flow: unknown cluster {}!", in_addr)),
            Some((.., ref mut loc_cluster)) => {
                for in_name in &input {
                    let ids = match loc_cluster.get_mut(in_name) {
                        None => return Err(format!("Producer - Flow: unknown input process {}!",in_name)),
                        Some((ref mut loc_cluster, ..)) => {
                            // test connectors
                            match loc_cluster.remove(name) {
                                Some(ps::Query { in_type, out_type, }) => (in_type, out_type,),
                                _ => return Err(format!("Producer - Flow: no query at {} for channel {}!", in_name, name)),
                            }
                        }            
                    };
                    if in_out.is_none() { in_out = Some(ids); } 
                    else { if in_out != Some(ids) { return Err(format!("Producer - Flow: conflicting type for channel {}!",name)); } }
                }
            }
        }
        match self.clusters.get_mut(out_addr) {
            None => return Err(format!("Producer - Flow: unknown cluster {}!", out_addr)),
            Some((.., ref mut loc_cluster)) => {
                for out_name in &output {
                    let ids = match loc_cluster.get_mut(out_name) {
                        None => return Err(format!("Producer - Flow: unknown output process {}!",out_name)),
                        Some((ref mut loc_cluster, ..)) => {
                            // test connectors
                            match loc_cluster.remove(name) {
                                Some(ps::Reply { in_type, out_type, }) => (in_type, out_type,),
                                _ => return Err(format!("Producer - Flow: no reply at {} for channel {}!", out_name, name)),
                            }
                        }            
                    };
                    // initialization is not needed here         
                    if in_out != Some(ids) { return Err(format!("Producer - Flow: conflicting type for channel {}!",name)); }
                }   
            }
        }
        let (query_type, reply_type) = in_out.expect("Producer - Flow: unexpected error");
        Ok((input, output, query_type, reply_type))
    }
    fn get_broadcast_sign<I,O>(&mut self, name: &str, in_addr: &SocketAddr, out_addr: &SocketAddr, in_names: I, out_names: O,) 
            -> Result<(BTreeSet<String>,BTreeSet<String>,FullId,),String> where I: IntoIterator<Item=String>, O: IntoIterator<Item=String> {
        let input: BTreeSet<_> = in_names.into_iter().collect();
        let output: BTreeSet<_> = out_names.into_iter().collect();
        if input.is_empty() { return Err(format!("Producer - Flow: input is empty!")); }
        if output.is_empty() { return Err(format!("Producer - Flow: output is empty!")); }
        let mut datyp: Option<FullId> = None;
        match self.clusters.get_mut(in_addr) {
            None => return Err(format!("Producer - Flow: unknown cluster {}!", in_addr)),
            Some((.., ref mut loc_cluster)) => {
                for in_name in &input {
                    let idt = match loc_cluster.get_mut(in_name) {
                        None => return Err(format!("Producer - Flow: unknown input process {}!",in_name)),
                        Some((ref mut loc_cluster, ..)) => {
                            // test connectors
                            match loc_cluster.remove(name) {
                                Some(ps::Emit { in_type, }) => in_type,
                                _ => return Err(format!("Producer - Flow: no emit at {} for channel {}!", in_name, name)),
                            }
                        }            
                    };
                    if datyp.is_none() { datyp = Some(idt); } 
                    else { if datyp != Some(idt) { return Err(format!("Producer - Flow: conflicting type for channel {}!",name)); } }
                }
            }
        }
        match self.clusters.get_mut(out_addr) {
            None => return Err(format!("Producer - Flow: unknown cluster {}!", out_addr)),
            Some((.., ref mut loc_cluster)) => {
                for out_name in &output {
                    let idt = match loc_cluster.get_mut(out_name) {
                        None => return Err(format!("Producer - Flow: unknown output process {}!",out_name)),
                        Some((ref mut loc_cluster, ..)) => {
                            // test connectors
                            match loc_cluster.remove(name) {
                                Some(ps::Read { out_type, }) => out_type,
                                _ => return Err(format!("Producer - Flow: no read at {} for channel {}!", out_name, name)),
                            }
                        }            
                    };
                    // initialization is not needed here         
                    if datyp != Some(idt) { return Err(format!("Producer - Flow: conflicting type for channel {}!",name)); }
                }   
            }
        }
        let data_type = datyp.expect("Producer - Flow: unexpected error");
        Ok((input, output, data_type))
    }
    fn get_signal_sign<I,O>(&mut self, name: &str, in_addr: &SocketAddr, out_addr: &SocketAddr, in_names: I, out_names: O,) 
            -> Result<(BTreeSet<String>,BTreeSet<String>,FullId,),String> where I: IntoIterator<Item=String>, O: IntoIterator<Item=String> {
        let input: BTreeSet<_> = in_names.into_iter().collect();
        let output: BTreeSet<_> = out_names.into_iter().collect();
        if input.is_empty() { return Err(format!("Producer - Flow: input is empty!")); }
        if output.is_empty() { return Err(format!("Producer - Flow: output is empty!")); }
        let mut datyp: Option<FullId> = None;
        match self.clusters.get_mut(in_addr) {
            None => return Err(format!("Producer - Flow: unknown cluster {}!", in_addr)),
            Some((.., ref mut loc_cluster)) => {
                for in_name in &input {
                    let idt = match loc_cluster.get_mut(in_name) {
                        None => return Err(format!("Producer - Flow: unknown input process {}!",in_name)),
                        Some((ref mut loc_cluster, ..)) => {
                            // test connectors
                            match loc_cluster.remove(name) {
                                Some(ps::Emit { in_type, }) => in_type,
                                _ => return Err(format!("Producer - Flow: no emit at {} for channel {}!", in_name, name)),
                            }
                        }            
                    };
                    if datyp.is_none() { datyp = Some(idt); } 
                    else { if datyp != Some(idt) { return Err(format!("Producer - Flow: conflicting type for channel {}!",name)); } }
                }
            }
        }
        match self.clusters.get_mut(out_addr) {
            None => return Err(format!("Producer - Flow: unknown cluster {}!", out_addr)),
            Some((.., ref mut loc_cluster)) => {
                for out_name in &output {
                    let idt = match loc_cluster.get_mut(out_name) {
                        None => return Err(format!("Producer - Flow: unknown output process {}!",out_name)),
                        Some((ref mut loc_cluster, ..)) => {
                            // test connectors
                            match loc_cluster.remove(name) {
                                Some(ps::RefRead { out_type, }) => out_type,
                                _ => return Err(format!("Producer - Flow: no ref read at {} for channel {}!", out_name, name)),
                            }
                        }            
                    };
                    // initialization is not needed here         
                    if datyp != Some(idt) { return Err(format!("Producer - Flow: conflicting type for channel {}!",name)); }
                }   
            }
        }
        let data_type = datyp.expect("Producer - Flow: unexpected error");
        Ok((input, output, data_type))
    }
}
