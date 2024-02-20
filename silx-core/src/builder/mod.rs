use std::{ 
    collections::{ BTreeMap, BTreeSet, HashMap, HashSet, }, fmt::Debug, net::SocketAddr, path::{Path, PathBuf} 
};

use serde::{ Serialize, Deserialize, };
use fnv::FnvHashMap;
use tokio::{
    net::{ TcpStream, TcpListener, }, spawn, sync::mpsc as msc, time::{ sleep, Duration, },
};

use crate::{
    shared::{
        utils::{ SendToMaster, ProcessInstance, },
        id_tools::{ IdBuilder, TaskIdGenerator, },
    },
    net::SilxProtocols,
    ChannelIdType, ServantIdType,
    structs::cells::{
        servant::Servant, master::Master,
        ctrl_message::{ RecvFromMaster, SendToServant, RecvFromServant, SetChannel as sch, },
    },
    traits::{
        FullId,
        procell::{ HasProcess, ProcessCell, }, 
        filable::{ Filable, Filed, RecFiled, },
    },
    structs::archmod::archannel::{ 
        ArchQuery, ArchBroadcast, ArchSignal, ChannelServer, ChannelClient,
    },
};


static REQUEST_COMMAND: &'static str = "REQUEST_INIT\n";
static READY_COMMAND: &'static str = "SIGNAL_READY\n";

/// hidden module
mod tool {
    use super::ServantBuilder;
    /// hidden trait for defining ServantBuilder object clone
    pub trait BoxClone {
        fn box_clone(&self) -> Box<dyn ServantBuilder>;
    }
    impl<S> BoxClone for S where S: 'static + ServantBuilder + Clone {
        fn box_clone(&self) -> Box<dyn ServantBuilder> { Box::new(self.clone()) }
    }    
}

/// Trait for defining servant builder parameters
pub trait ServantBuilderParameters {
    /// Max duration for master to servant query request
    /// * if this time is exceeded, the cluster master considers that the servant has broken down and kills him
    fn max_cycle_time(&self) -> Duration;

    /// build process instance
    /// * `task_id: IdBuilder` : 
    /// * `send_to_master: SendToMaster` : 
    /// * Output: process instance
    fn build_process(&self, task_id: IdBuilder, send_to_master: SendToMaster,) -> ProcessInstance;
}

#[typetag::serde(tag = "servant")]
/// Trait with serde tag and main servant builder implementations
pub trait ServantBuilder: Send + tool::BoxClone + ServantBuilderParameters {
    /// Clone the servant builder as boxed `dyn ServantBuilder`
    ///  * Output: boxed clone as `dyn ServantBuilder`
    fn dyn_clone(&self) -> Box<dyn ServantBuilder> { self.box_clone() }

    /// Build a servant instance
    /// * This method is for silx internal use
    /// * `servant_id: ServantIdType` : internal servant id
    /// * `servant_name: String` : servant name
    /// * `ch_naming: &FnvHashMap<ChannelIdType, String,>` : channels ids and names
    /// * `recv_from_master: RecvFromMaster` : channel receiver from the cluster master
    /// * `send_2_master: SendToMaster` : channel sender to the cluster master
    /// * `task_id: IdBuilder` : task id builder for tasking the cluster master
    ///  * Output: a servant instance or an error
    fn build(&self, servant_id: ServantIdType, servant_name: String, ch_naming: &FnvHashMap<ChannelIdType, String,>, 
        recv_from_master: RecvFromMaster, send_2_master: SendToMaster, task_id: IdBuilder,
    ) -> Result<Servant,String> {
        let named_process = self.build_process(task_id,send_2_master).0;
        let oprocess_cell =  ProcessCell::new(named_process, ch_naming,);
        let max_cycle_time = self.max_cycle_time();

        match oprocess_cell {
            None => Err("Failed to build process cell".to_string()),
            Some(pc) => {
                let processes = Box::new(pc) as Box<dyn HasProcess + Send>;
                Ok(Servant::new(servant_id, servant_name, processes, max_cycle_time, recv_from_master,))
            },
        }
    }
    // Nota: within builder, the processes are linked to names
}

pub enum MasterBuilder {}
impl MasterBuilder {
    fn build(// Nota: Servant naming and id are unique through the responsible cluster only
        cluster_id: SocketAddr,
        server: ChannelServer, // server is stored within master, so as to be active until cluster is dropped
        sv_naming: &FnvHashMap<ServantIdType, String,>,
        mut nchannels: BTreeMap<String, Vec<(ChannelIdType, sch)>>,
        mut nsend_to_servants: BTreeMap<String,SendToServant>, 
        mut nrecv_from_servants: BTreeMap<String,RecvFromServant>, 
        task_id: IdBuilder,
    ) -> Result<Master,String> {
        // on ne retient que la partie effectivement présente; Sv_naming peut contenir plus de label
        let send_to_servants = sv_naming.iter().filter_map(|(id,name)| nsend_to_servants.remove(name).map(|ch|(*id,ch))).collect();
        // par contre, erreur si des éléments de send_to_servants sont oubliés
        if !nsend_to_servants.is_empty() { return Err("send_to_servants: some servant id are undefined".to_string()); }
        let recv_from_servants = sv_naming.iter().filter_map(|(id,name)| nrecv_from_servants.remove(name).map(|ch|(*id,ch))).collect();
        if !nrecv_from_servants.is_empty() { return Err("recv_from_servants: some servant id are undefined".to_string()); }
        let channels: FnvHashMap<_,_> = sv_naming.iter().filter_map(|(id,name)| nchannels.remove(name).map(|ch|(*id,ch))).collect();
        if !nchannels.is_empty() { return Err("channel: some servant id are undefined".to_string()); }
        Ok(Master::new(cluster_id, server, send_to_servants, recv_from_servants, channels, task_id,))
    }
}

#[derive(Clone, Serialize, Deserialize, Debug,)]
/// doc to be defined
pub enum Channel {
    Query {
        cluster: SocketAddr,
        max_ping: Duration,
        query_type: FullId,
        reply_type: FullId,
        size: Option<usize>,
        input: BTreeSet<String>,
        output: BTreeSet<String>,
    },
    Broadcast {
        cluster: SocketAddr,
        max_ping: Duration,
        data_type: FullId,
        size: usize,
        input: BTreeSet<String>,
        output: BTreeSet<String>,
    },
    Signal {
        cluster: SocketAddr,
        max_ping: Duration,
        data_type: FullId,
        input: BTreeSet<String>,
        output: BTreeSet<String>,
    },
    NetQuery {
        max_ping: Duration,
        query_type: FullId,
        reply_type: FullId,
        size: Option<usize>,
        input:  (SocketAddr,BTreeSet<String>,),
        output: (SocketAddr,BTreeSet<String>,),
    },
    NetBroadcast {
        max_ping: Duration,
        data_type: FullId,
        size: usize,
        input:  (SocketAddr,BTreeSet<String>,),
        output: (SocketAddr,BTreeSet<String>,),
    },
}

impl Channel {
    pub fn has_cluster(&self, socket: &SocketAddr) -> bool {
        use Channel::{Query, Broadcast, Signal, NetQuery, NetBroadcast, };
        match self {
            Query { cluster, ..} | Broadcast { cluster, .. } | Signal { cluster, .. } => { socket == cluster },
            NetQuery { input: (icluster,_), output: (ocluster,_), .. } | NetBroadcast { input: (icluster,_), output: (ocluster,_), .. } => {
                icluster == socket || ocluster == socket
            },
        }
    } 
    pub fn has_input_servant(&self, name: &str) -> bool {
        use Channel::{Query, Broadcast, Signal, NetQuery, NetBroadcast, };
        match self {
            Query { input, ..} | Broadcast { input, .. } | Signal { input, .. } | 
                NetQuery { input: (_,input), .. } | NetBroadcast { input: (_,input), .. } => {
                    input.contains(name)
                }
        }
    } 
    pub fn has_output_servant(&self, name: &str) -> bool {
        use Channel::{Query, Broadcast, Signal, NetQuery, NetBroadcast, };
        match self {
            Query { output, ..} | Broadcast { output, .. } | Signal { output, .. } | 
                NetQuery { output: (_,output), .. } | NetBroadcast { output: (_,output), .. } => {
                    output.contains(name)
                },
        }
    } 
    pub fn has_servant(&self, name: &str) -> bool { self.has_input_servant(name) || self.has_output_servant(name) } 
}

#[derive(Clone,Serialize, Deserialize, Debug,)]
/// doc to be defined
pub struct FiledClusterBuilder {
    pub net_size: Option<usize>,
    pub named_servants: BTreeMap<String,Filed<Box<dyn ServantBuilder>>>,
    pub ctrl_ch_capacity: usize,
    // A généraliser en mpmc et network
}
impl Filable for FiledClusterBuilder {
    type Unfiled = ClusterBuilder;

    fn load<P: AsRef<Path>,>(&mut self, path: P,) -> Result<bool,String> {
        let path = path.as_ref();
        let mut updated = false;
        for (_,fserv) in self.named_servants.iter_mut() { updated |= fserv.load(path)?; }
        Ok(updated)
    }

    fn unload(&mut self, opath: Option<&Path>,) -> Result<Self::Unfiled,String,> {
        let mut named_servants = vec![];
        for (rname,fserv) in self.named_servants.iter_mut() { named_servants.push((rname.clone(),fserv.unload(opath.clone())?)); }
        let named_servants = named_servants.into_iter().collect();
        let net_size = self.net_size;
        let ctrl_ch_capacity = self.ctrl_ch_capacity;
        Ok(ClusterBuilder { net_size, named_servants, ctrl_ch_capacity, })
    }
}

#[derive(Clone,Serialize, Deserialize, Debug,)]
/// doc to be defined
pub struct ClusterBuilder {
    pub net_size: Option<usize>,
    pub named_servants: BTreeMap<String,Box<dyn ServantBuilder>>,
    pub ctrl_ch_capacity: usize,
}

#[derive(Clone,Serialize, Deserialize, Debug,)]
/// Filed definition of a starter, i.e. this definition contains files names in order to serialize into different sub-files.
/// * starters comes with two variants: `Main` and `Listener`
pub enum FiledStarter {
    /// Definition of the main cluster
    Main {
        /// List of the clusters definitions with their socket address
        builders: BTreeMap<SocketAddr,RecFiled<FiledClusterBuilder>>,
        /// List of the definitions of the channels with their names
        flow: BTreeMap<String,Filed<Channel>>,
        /// Socket address of the main cluster
        main: SocketAddr,
    },
    /// Definition of a slave cluster: this cluster will await parameters from the main
    Listener {
        /// Socket address of the main cluster
        main: SocketAddr,
        /// Socket address of this cluster
        this: SocketAddr,
    }
}

impl Filable for FiledStarter {
    type Unfiled = Starter;

    fn load<P: AsRef<Path>,>(&mut self, path: P,) -> Result<bool,String> {
        let mut updated = false;
        let path = path.as_ref();
        if let Self::Main { builders, flow, .. } = self {
            for (_,lbuild) in builders.iter_mut() { updated |= lbuild.load(path)? }
            for (_,lchan) in flow.iter_mut() { updated |= lchan.load(path)?; }
        }
        Ok(updated)
    }

    fn unload(&mut self, opath: Option<&Path>,) -> Result<Self::Unfiled,String,> {
        Ok(match self {
            Self::Listener { main: m, this: t, } => Starter::Listener { main: *m, this: *t, },
            Self::Main { builders, flow, main: m } => {
                let mut vbuilders = vec![];
                for (rname,fbuild) in builders.iter_mut() { 
                    let in_data = fbuild.unload(opath.clone(),)?;
                    vbuilders.push((rname.clone(),in_data));
                }
                let builders = vbuilders.into_iter().collect();
                let mut vflow = vec![];
                for (rname,fchan) in flow.iter_mut() { vflow.push((rname.clone(),fchan.unload(opath.clone())?)); }
                let flow = vflow.into_iter().collect();
                Starter::Main { builders, flow, main: *m, }
            },
        })
    }
}

#[derive(Clone, Debug,)]
/// Unfiled definition of a starter
/// * starters comes with two variants: `Main` and `Listener`
pub enum Starter {
    /// Definition of the main cluster
    Main {
        /// List of the clusters definitions with their socket address
        builders: BTreeMap<SocketAddr,ClusterBuilder>,
        /// List of the definitions of the channels with their names
        flow: BTreeMap<String,Channel>, 
        /// Socket address of the main cluster
        main: SocketAddr,
    },
    /// Definition of a slave cluster: this cluster will await parameters from the main
    Listener {
        /// Socket address of the main cluster
        main: SocketAddr,
        /// Socket address of this cluster
        this: SocketAddr,
    }
}


impl Starter {
    pub (crate) async fn build_all(main: &SocketAddr, 
        mut builders: BTreeMap<SocketAddr, ClusterBuilder>, flow: BTreeMap<String, Channel>,
    ) -> Result<(FnvHashMap<ChannelIdType,(String,Channel)>,ClusterBuilder,), String> {
        let mut chan_id = 0;
        let named_flow: FnvHashMap<ChannelIdType, (String,Channel)> = flow.into_iter().map(move |named| { 
            let kv = (chan_id,named); chan_id += 1; kv
        }).collect();
        let main_id_name_flow: FnvHashMap<ChannelIdType, (String,Channel),> = named_flow.iter().filter(|(_,(_,ref rc))|rc.has_cluster(main))
            .map(|(&i,rc)| (i, rc.clone())).collect();
        let main_builder = if let Some(b) = builders.remove(main) { b } else { 
            return Err(format!("Main cluster builder at address {} is undefined", main))
        };
        // building and running initilisation server:
        let listener = TcpListener::bind(main).await.expect(&format!("Starter::build_all -> Failed to listen on {}", main));
        let mut sockets = Vec::new();
        while !builders.is_empty() {
            let named_flow = named_flow.clone();
            let (mut socket, _) = listener.accept().await.expect(&format!("Starter::build_all -> Failed while listening on {}", main));
            let socket_addr = match SilxProtocols::pop_socket(&mut socket).await { // commencer par obtenir l'identifiant
                Ok(soa) => soa, Err(e) => return Err(format!("Starter::build_all - pop_socket failure -> {e}")),
            };
            if let Some(builder) = builders.remove(&socket_addr) {
                let (mut reader, mut writer) = socket.split();
                let sub_id_name_flow: FnvHashMap<ChannelIdType, (String,Channel),> = named_flow.iter().filter(|(_,(_,ref rc))|rc.has_cluster(&socket_addr))
                    .map(|(&i,rc)| (i, rc.clone())).collect();
                let flow_n_builder = (sub_id_name_flow,builder);
                let yaml: String = match serde_yaml::to_string(&flow_n_builder) {
                    Ok(s) => s, Err(_) => return Err(format!("Starter::build_all -> Failed to serialize flow and builder")),
                };
                let command = match SilxProtocols::pop_string(&mut reader).await {
                    Ok(s)                    => s,
                    Err(_)                   => return Err(format!("Starter::build_all - pop_string failure  -> Failed to read command bytes")),
                };
                if command.as_str() == REQUEST_COMMAND {
                    if SilxProtocols::push_string(&mut writer, &yaml,).await.is_err() {
                        return Err(format!("Starter::build_all - push_string failure  -> Failed to write Yaml sting"));
                    }
                } else { return Err(format!("Unknown command!")); }
            };
            sockets.push(socket);
        }
        for mut socket in sockets { // envoi du signal ready!
            if SilxProtocols::push_string(&mut socket, READY_COMMAND).await.is_err() {
                return Err(format!("Starter::build_all - push_string failure  -> Failed to send ready signal"));
            }    
        }
        Ok((main_id_name_flow, main_builder,))
    }

    // instrumental function
    async fn build_server_clients(
        this: SocketAddr, rid_name_flow: &FnvHashMap<ChannelIdType, (String,Channel,),>, net_size: Option<usize>
    ) -> Result<(ChannelServer, HashMap<SocketAddr, ChannelClient>),String> {
        let mut server = ChannelServer::bind(this).await?; // build channel server of the cluster
        let mut server_of_clients = HashSet::new(); // contains the address of all clients to be accepted by 'this' server
        let mut client_of_servers = HashSet::new(); // contains the address of all servers to connect by 'this' client
        for (_,(_,ref chan)) in rid_name_flow { // populate input_clients and output_clients
            match chan {
                Channel::NetQuery {  input:  (ina,_,), output: (outa,_,), .. } | Channel::NetBroadcast { input:  (ina,_,), output: (outa,_,), .. } => {
                    match (ina == &this, outa == &this) {
                        (true,false,) => { client_of_servers.insert(outa.clone()); },
                        (false,true,) => { server_of_clients.insert(ina.clone()); },
                        _ => panic!("Unexpected case"),
                    }
                },
                _ => (),
            }
        }
        let handle_accept = { // in this async thread, server will accept all client until requested client list is empty 
            spawn( async move {
                while !server_of_clients.is_empty() {
                    let _ = server.accept(&mut server_of_clients, net_size).await;
                }
                server
            })            
        };
        sleep(Duration::from_millis(100)).await; // sleep so as to start server async thread
        let mut clients = HashMap::new(); // create and populate clients
        for cos in client_of_servers {
            let client = ChannelClient::connect(this.clone(), cos).await?;
            clients.insert(cos,client);
            sleep(Duration::from_millis(100)).await; // sleep so as to avoid simultaneous
        }
        let server = match handle_accept.await { // wait until handle stop, and get back the server
            Ok(s) => s, Err(_) => return Err(format!("Failed to get back server from handle")),
        };
        Ok((server,clients))
    }

    async fn listen_init(main: SocketAddr, this: SocketAddr, ) -> Result<(FnvHashMap<ChannelIdType, (String,Channel,),>, ClusterBuilder,), String> {
        #[cfg(feature = "verbose2")]
        println!("{this}: try to connect {main}");
        let mut socket = match TcpStream::connect(&main).await {
            Ok(socket)  => socket,
            Err(e)           => { return Err(format!("Starter::listen_init -> failed to connect to main address {main} => {e}")); },
        };
        #[cfg(feature = "verbose2")]
        println!("{this}: Listening connection established");
        let (mut reader, mut writer) = socket.split();
        if SilxProtocols::push_socket(&mut writer, &this).await.is_err() {
            panic!("Starter::listen_init - push_socket -> failed to send identifier");
        }
        if SilxProtocols::push_string(&mut writer, REQUEST_COMMAND).await.is_err() {
            panic!("Starter::listen_init - push_string -> failed to request yaml");
        }

        let yaml_str = match SilxProtocols::pop_string(&mut reader).await {
            Ok(s)  => s,
            Err(e) => return Err(format!("Starter::listen_init - pop_string failure -> {e}")), 
        };
        let (id_name_flow,builder): (FnvHashMap<ChannelIdType, (String,Channel,),>, ClusterBuilder,) = match serde_yaml::from_str(&yaml_str) {
            Ok(builder) => builder,
            Err(e)      => {
                println!("Error: {:?}",e);
                return Err(format!("Failed to unserialize"))
            },
        };
        let ready = match SilxProtocols::pop_string(&mut reader).await {
            Ok(s)  => s,
            Err(e) => return Err(format!("Starter::listen_init - pop_string failure -> {e}")), 
        };
        if ready.as_str() != READY_COMMAND { return Err(format!("Fail to receive READY signal")) }
        Ok((id_name_flow,builder))
    }

    pub (crate) async fn listen(main: SocketAddr, this: SocketAddr, ) -> Result<(), String> {
        let (id_name_flow,builder) = Self::listen_init(main, this).await?;
        //
        let net_size = builder.net_size;
        let (server,clients) = Self::build_server_clients(this, &id_name_flow, net_size).await?;
        let (master,servants) = builder.build_cluster(this, server, clients, id_name_flow).await?;
        ClusterBuilder::run_cluster(master, servants,).await;
        Ok(())
    }

    /// Run the starter
    /// * Output: nothing or error
    pub async fn run(self) -> Result<(), String> {
        match self {
            Self::Listener { main, this, }       => Self::listen(main, this,).await,
            Self::Main { main, builders, flow, } => { 
                let (id_name_flow, main_builder,) = Self::build_all(&main, builders, flow,).await?; // build all cluster; binding on main is necessary
                let net_size = main_builder.net_size;
                //
                let (server,clients) = Self::build_server_clients(main, &id_name_flow, net_size).await?;
                let (master,servants) = main_builder.build_cluster(main, server, clients, id_name_flow).await?;
                ClusterBuilder::run_cluster(master, servants,).await;
                Ok(())
            },
        }
    }

    /// Load starter from saved files
    /// * `starter_path: P` : starter file name
    /// * `dir_path: Q` : directory of network saved files
    /// * `P` : type of path
    /// * `Q` : type of path
    /// * Output: Starter or error
    pub fn load<P,Q>(starter_path: P, dir_path: Q) -> Result<Self,String> where P: AsRef<Path>, Q: AsRef<Path> {
        let mut unloaded = RecFiled::<FiledStarter>::new_unloaded(starter_path);
        let save_dir = PathBuf::from(dir_path.as_ref());
        unloaded.load(&save_dir)?;
        unloaded.unwrap()
    }
}

impl ClusterBuilder {
    pub (crate) async fn build_cluster(self, cluster_id: SocketAddr, mut server: ChannelServer, mut clients: HashMap<SocketAddr, ChannelClient>, 
                                     id_name_flow: FnvHashMap<ChannelIdType,(String,Channel)>) -> Result<(Master,Vec<Servant>),String> {
        let ctrl_ch_capacity = self.ctrl_ch_capacity;
        // recupération de l'identification des canaux / trié par servant
        let ch_naming: BTreeMap<String,FnvHashMap<ChannelIdType, String,>> = self.named_servants.iter().map(|(serv_st,_)| {
            let sel_id_name_flow = id_name_flow.iter().filter(
                |(_,(_,ch))| ch.has_servant(serv_st)).map(|(id,(st,_))| (*id,st.to_string())
            ).collect();
            (serv_st.clone(),sel_id_name_flow)
        }).collect();
       
        // création de l'identifer de tâche
        let task_id = TaskIdGenerator::new();
        //création de l'identification des servant
        let mut sid: ServantIdType = 0;
        let sv_naming:FnvHashMap<ServantIdType, String,> = self.named_servants.iter().map(|(st,_)| {
            let sv_name = (sid,st.to_string()); sid += 1; sv_name
        } ).collect();
        // création des canaux de communication master -> servants ; indicé par le nom du servant
        let (nsend_to_servants, mut nrecv_from_master,): (BTreeMap<_,_>,BTreeMap<_,_>,) = self.named_servants.iter().map(|(st,_)| {
            let (sender,receiver) = msc::channel(ctrl_ch_capacity);
            ((st.to_string(),sender,),(st.to_string(),receiver,))
        } ).unzip();
        // création des canaux de communication servants -> master ; indicé par le nom du servant
        let (mut nsend_2_master,nrecv_from_servants,): (BTreeMap<_,_>,BTreeMap<_,_>) = self.named_servants.iter().map(|(st,_)| {
            let (sender,receiver) = msc::channel(ctrl_ch_capacity);
            ((st.to_string(),sender,),(st.to_string(),receiver,))
        } ).unzip();

        // création des servants
        let mut servants : Vec<Servant> = Vec::new();
        for (&servant_id,rname,) in &sv_naming {
            let rbuilder = self.named_servants.get(rname).expect("unexpected error: missing servant builder");
            let recv_from_master = match nrecv_from_master.remove(rname) {
                Some(r) => r, None => return Err(format!("failed to get recv_from_master for servant {}", rname)),
            };
            let send_2_master = match nsend_2_master.remove(rname) {
                Some(s) => s, None => return Err(format!("failed to get send_2_master for servant {}", rname)),
            };
            let servant = rbuilder.build(servant_id, rname.clone(), ch_naming.get(rname).expect("unexpected error"), recv_from_master, send_2_master, task_id.clone())?;
            servants.push(servant);
        };
        // création du maître
        let mut nchannels: BTreeMap<String, Vec<(ChannelIdType, sch)>> = BTreeMap::new(); // clef =  noms de servant du cluster
        for (_,&(_,ref chan)) in &id_name_flow { // preparation of structure (simple énumération des noms de servant sans filtrage par rapport au cluster)
            let (first, second) = match chan {
                Channel::Query { input, output, .. } | Channel::Broadcast { input, output, .. } | Channel::Signal { input, output, .. } => (input,Some(output)),
                Channel::NetQuery { input: (icluster,in_names,), .. } if icluster == &cluster_id => (in_names, None),
                Channel::NetQuery { output: (ocluster,out_names,), .. } if ocluster == &cluster_id => (out_names, None), 
                Channel::NetBroadcast { input: (icluster,in_names,), .. } if icluster == &cluster_id => (in_names, None),
                Channel::NetBroadcast { output: (ocluster,out_names,), .. } if ocluster == &cluster_id => (out_names, None),
                _ => panic!("Unexpected case!"),
            };
            for name in first { if !nchannels.contains_key(name) { nchannels.insert(name.to_string(), Vec::new()); } }
            if let Some(second) = second {
                for name in second { if !nchannels.contains_key(name) { nchannels.insert(name.to_string(), Vec::new()); } }    
            }
        }
        for (id,(_,chan)) in id_name_flow {
            match chan {
                Channel::Query { size, input, output, max_ping, query_type, reply_type, .. } => {
                    let (sender,receiver) = if let Some(size) = size { ArchQuery::bounded::<(),()>(size) } else { ArchQuery::unbounded::<(),()>() };
                    let sender = sender.inner(); let receiver = receiver.inner();
                    for iname in &input {
                        let query_type = query_type.clone();
                        let reply_type = reply_type.clone();
                        let sender = sender.clone();
                        nchannels.get_mut(iname).expect("Unexpected: missing key").push((id, sch::QuerySender { max_ping, query_type, reply_type, sender, }));
                    }
                    for oname in &output {
                        let query_type = query_type.clone();
                        let reply_type = reply_type.clone();
                        let receiver = receiver.clone();
                        nchannels.get_mut(oname).expect("Unexpected: missing key").push((id, sch::QueryReceiver { max_ping, query_type, reply_type, receiver }));
                    }
                },
                Channel::Broadcast { size, input, output, max_ping, data_type, .. } => {
                    let (sender,receiver) = ArchBroadcast::channel::<()>(size).await;
                    let sender = sender.inner(); let receiver = receiver.inner();
                    for iname in &input {
                        let data_type = data_type.clone();
                        let sender = sender.clone();
                        nchannels.get_mut(iname).expect("Unexpected: missing key").push((id, sch::BroadcastSender { max_ping, data_type, sender, }));
                    }
                    for oname in &output {
                        let data_type = data_type.clone();
                        let receiver = receiver.clone();
                        nchannels.get_mut(oname).expect("Unexpected: missing key").push((id, sch::BroadcastReceiver { max_ping, data_type, receiver }));
                    }          
                },
                Channel::Signal { input, output, max_ping, data_type, .. } => {
                    let (sender,receiver) = ArchSignal::channel::<()>();
                    let sender = sender.inner(); let receiver = receiver.inner();
                    for iname in &input {
                        let data_type = data_type.clone();
                        let sender = sender.clone();
                        nchannels.get_mut(iname).expect("Unexpected: missing key").push((id, sch::SignalSender { max_ping, data_type, sender, }));
                    }
                    for oname in &output {
                        let data_type = data_type.clone();
                        let receiver = receiver.clone();
                        nchannels.get_mut(oname).expect("Unexpected: missing key").push((id, sch::SignalReceiver { max_ping, data_type, receiver }));
                    }
                },
                Channel::NetQuery {
                    size,
                    max_ping,
                    query_type,
                    reply_type,
                    input:  (icluster,in_names,),
                    output: (ocluster,out_names,),
                } => {
                    if icluster == cluster_id {
                        if !in_names.is_empty() {
                            let sender = clients.get_mut(&ocluster).expect("Unexpected: client not found").sender_query::<(),()>(id, size,).await.inner();    
                            for iname in &in_names {
                                let query_type = query_type.clone();
                                let reply_type = reply_type.clone();
                                let sender = sender.clone();
                                nchannels.get_mut(iname).expect("Unexpected: missing key").push((id, sch::NetQuerySender { max_ping, query_type, reply_type, sender, }));
                            }    
                        }
                    }
                    if ocluster == cluster_id {
                        if !out_names.is_empty() {
                            let receiver = server.receiver_query::<(),()>(id, size,).await.inner();
                            for oname in &out_names {
                                let query_type = query_type.clone();
                                let reply_type = reply_type.clone();
                                let receiver = receiver.clone();
                                nchannels.get_mut(oname).expect("Unexpected: missing key").push((id, sch::NetQueryReceiver { max_ping, query_type, reply_type, receiver }));
                            }
                        }
    
                    }
                },
                Channel::NetBroadcast {
                    size,
                    max_ping,
                    data_type,            
                    input:  (icluster,in_names,),
                    output: (ocluster,out_names,),
                } => {
                    if icluster == cluster_id {
                        if !in_names.is_empty() {
                            let sender = clients.get_mut(&ocluster).expect("Unexpected: client not found").sender_broadcast::<()>(id, size,).await.inner();    
                            for iname in &in_names {
                                let data_type = data_type.clone();
                                let sender = sender.clone();
                                nchannels.get_mut(iname).expect("Unexpected: missing key").push((id, sch::NetBroadcastSender { max_ping, data_type, sender, }));
                            }
                        }    
                    }
                    if ocluster == cluster_id {
                        if !out_names.is_empty() {
                            let receiver = server.receiver_broadcast::<()>(id, size,).await.inner();
                            for oname in &out_names {
                                let data_type = data_type.clone();
                                let receiver = receiver.clone();
                                nchannels.get_mut(oname).expect("Unexpected: missing key").push((id, sch::NetBroadcastReceiver { max_ping, data_type, receiver }));
                            }
                        }
                    }
                },
            }
        }
        // en principe, nchannels ne doit pas contenir de vecteurs vides
        let master = MasterBuilder::build(cluster_id, server, &sv_naming, nchannels, nsend_to_servants, nrecv_from_servants, task_id.clone())?;
        #[cfg(feature = "verbose1")]
        println!("cluster {cluster_id} has been built");
        Ok((master, servants))
    }

    pub (crate) async fn run_cluster(master: Master, servants: Vec<Servant>,) { // run the cluster
        let cluster_id = master.cluster_id;
        let mut handles = Vec::new();
        for servant in servants { handles.push(spawn(servant.run())); }
        handles.push(spawn(master.run()));
        for handle in handles { 
            match handle.await {
                Ok(_) => (),
                Err(e) => println!("handle.await -> {}", e),
            } 
        }
        #[cfg(feature = "verbose1")]
        println!("cluster {cluster_id} is ended");
    }
}


impl Clone for Box<dyn ServantBuilder> {
    fn clone(&self) -> Self { self.dyn_clone() }
}

impl Debug for Box<dyn ServantBuilder> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f,"servant_builder")
    }
}
