// ======= IMPORTS

use std::{ collections::HashMap, net::SocketAddr, path::Path, time::Duration };
use silx_core::{ 
    servants::shutdown::ShutdownBuilder, 
    utils::{ StarterProducer, FiledStarter, RecFiled, },
};

use super::{ MyLooperBuilderVec, MyAdderBuilderVec, MyPowerBuilderVec, MyPowerBuilderAltVec, };

// ======= BUILDING STARTER
#[allow(dead_code)]
/// build starters loaders for the vector example (with 4 clusters)
/// * `main_addr: SocketAddr` : IP adresse of main servant
/// * `slave_1_addr: SocketAddr` : IP adresse of slave servant 1
/// * `slave_2_addr: SocketAddr` : IP adresse of slave servant 2
/// * `main_starter_path: P` : path of starter file of main servant 
/// * `slave_1_starter_path: Q` : path of starter file of slave servant 1
/// * `slave_2_starter_path: R` : path of starter file of slave servant 2
/// * `P: AsRef<Path>` : type of starter file path of main servant
/// * `Q: AsRef<Path>` : type of starter file path of slave servant 1
/// * `R: AsRef<Path>` : type of starter file path of slave servant 2
/// * Output: sequence of starters loaders indexed with their IP addresses. The resulting process will solve the following task:
///     * 20 vectors are generated starting from \[1,2,3\] with step \[1,2,3\] (time-spaced by 1s)
///     * these vectors are raised element-wise to the power of 2
///     * results are summed
pub fn build_my_starter_vec<P: AsRef<Path>,Q: AsRef<Path>,R: AsRef<Path>> (
    main_addr: SocketAddr, slave_1_addr: SocketAddr, slave_2_addr: SocketAddr, 
    main_starter_path: P, slave_1_starter_path: Q, slave_2_starter_path: R,
) -> Result<HashMap<SocketAddr,RecFiled<FiledStarter>>,String> {
    // set builder paths for main, slave1 and slave2
    let main_builder_path = "silx/vec/builders/main_builder.yaml";
    let slave_1_builder_path = "silx/vec/builders/slave_1_builder.yaml";
    let slave_2_builder_path = "silx/vec/builders/slave_2_builder.yaml";
    // set control channel capacity
    let ctrl_ch_capacity = 16;
    // set networked channel capacity
    let net_size = Some(16);
    // set max ping for failure detection (1s ; this is much longer than necessary)
    let max_ping = Duration::from_millis(1000);

    // set the starters producer
    let start_prod = StarterProducer::new(
        // producer is initialized with main cluster parameters
        main_addr, main_starter_path, main_builder_path, net_size, ctrl_ch_capacity
    ).add_cluster(
        // adding slave1 cluster parameters
        slave_1_addr, slave_1_starter_path, slave_1_builder_path, net_size, ctrl_ch_capacity
    )?.add_cluster(
        // adding slave2 cluster parameters
        slave_2_addr, slave_2_starter_path, slave_2_builder_path, net_size, ctrl_ch_capacity
    )?.done();

    // add proceses to the producer
    let start_prod = start_prod.add_process(
        &main_addr, format!("looper"), // add process looper to main cluster 
        "silx/vec/servants/servant_looper.yaml", // process definition is serialized within servant_looper.yaml
        MyLooperBuilderVec::default_channels(20,3), // call the constructor of this process
    )?.add_process(
        &slave_1_addr, format!("power_1"), // add process "power_1 to slave cluster 1
        "silx/vec/servants/servant_power_1.yaml", // process definition is serialized within servant_power_1.yaml
        MyPowerBuilderVec::default_channels(2,1), // call the constructor of this process
    )?.add_process(
        &slave_1_addr, format!("power_2"), // add process power_2 to slave cluster 1
        "silx/vec/servants/servant_power_2.yaml", // process definition is serialized within servant_power_2.yaml
        MyPowerBuilderAltVec::default_channels(2,2), // call the constructor of this process
    )?.add_process(
        &slave_2_addr, format!("adder"), // add process adder to slave cluster 2
        "silx/vec/servants/servant_adder.yaml", // process definition is serialized within servant_adder.yaml
        MyAdderBuilderVec::default_channels(), // call the constructor of this process
    )?.add_process(
        &main_addr, format!("shutdown_1"), // add process shutdown_1 to main cluster 
        "silx/vec/servants/servant_shutdown_1.yaml", // process definition is serialized within servant_shutdown_1.yaml
        ShutdownBuilder::new(format!("Shutdown_1")), // call the constructor of this process
    )?.add_process(
        &slave_1_addr, format!("shutdown_2"), // add process shutdown_2 to slave cluster 1
        "silx/vec/servants/servant_shutdown_2.yaml", // process definition is serialized within servant_shutdown_2.yaml
        ShutdownBuilder::new(format!("Shutdown_2")), // call the constructor of this process
    )?.add_process(
        &slave_2_addr, format!("shutdown_3"), // add process shutdown_3 to slave cluster 2
        "silx/vec/servants/servant_shutdown_3.yaml", // process definition is serialized within servant_shutdown_3.yaml
        ShutdownBuilder::new(format!("Shutdown_3")), // call the constructor of this process
    )?.done();

    // add channels to the producer
    Ok(start_prod.add_net_query( // add query channel between two clusters
        "silx/vec/channels/channel_pow.yaml", // channel definition is serialized within channel_pow.yaml
        format!("Power"), // channel name 
        main_addr, [format!("looper"),], // input cluster and input servants  
        slave_1_addr, [format!("power_1"),format!("power_2"),], // output cluster and output servants 
        max_ping, Some(16),
    )?.add_net_broadcast( // add broadcast channel between two clusters
        "silx/vec/channels/channel_add.yaml", // channel definition is serialized within channel_add.yaml 
        format!("Adder"), // channel name
        main_addr, [format!("looper"),], // input cluster and input servants
        slave_2_addr, [format!("adder"),], // output cluster and output servants 
        max_ping, 16,
    )?.add_broadcast( // add broadcast channel within the same cluster
        "silx/vec/channels/channel_shutdown_1.yaml", // channel definition is serialized within channel_shutdown_1.yaml 
        format!("Shutdown_1"), // channel name 
        main_addr, // cluster address 
        [format!("looper"),], // input servants 
        [format!("shutdown_1"),], // output servants 
        max_ping, 16,
    )?.add_net_broadcast( // add broadcast channel between two clusters
        "silx/vec/channels/channel_shutdown_2.yaml", // channel definition is serialized within channel_shutdown_2.yaml 
        format!("Shutdown_2"), // channel name 
        main_addr, [format!("looper"),], // input cluster and input servants
        slave_1_addr, [format!("shutdown_2"),], // output cluster and output servants 
        max_ping, 16,
    )?.add_net_broadcast( // add broadcast channel between two clusters
        "silx/vec/channels/channel_shutdown_3.yaml", // channel definition is serialized within channel_shutdown_3.yaml 
        format!("Shutdown_3"), // channel name 
        main_addr, [format!("looper"),], // input cluster and input servants
        slave_2_addr, [format!("shutdown_3"),], // output cluster and output servants 
        max_ping, 16,
    )?.done())
}

// ======= BUILDING STARTER
#[allow(dead_code)]
/// build starter loader for the vector example (with 1 cluster)
/// * `main_addr: SocketAddr` : IP adresse of main servant
/// * `main_starter_path: P` : path of starter file of main servant 
/// * `P: AsRef<Path>` : type of starter file path of main servant
/// * Output: sequence of starters (actually only main) loaders indexed with their IP addresses 
///     * 20 vectors are generated starting from \[1,2,3\] with step \[1,2,3\] (time-spaced by 1s)
///     * these vectors are raised element-wise to the power of 2
///     * results are summed
pub fn build_my_starter_vec_mono<P: AsRef<Path>,> (
    main_addr: SocketAddr, main_starter_path: P,
) -> Result<HashMap<SocketAddr,RecFiled<FiledStarter>>,String> {
    // set builder path for main
    let main_builder_path = "silx/vec-mono/builders/main_builder.yaml";
    // set control channel capacity
    let ctrl_ch_capacity = 16;
    // set networked channel capacity
    let net_size = Some(16);
    // set max ping for failure detection (1s ; this is much longer than necessary)
    let max_ping = Duration::from_millis(1000);

    // set the starters producer
    let start_prod = StarterProducer::new(
        // producer is initialized with main cluster parameters
        main_addr, main_starter_path, main_builder_path, net_size, ctrl_ch_capacity
    ).done();

    // add proceses to the producer
    let start_prod = start_prod.add_process(
        &main_addr, format!("looper"), // add process looper to main cluster 
        "silx/vec-mono/servants/servant_looper.yaml", // process definition is serialized within servant_looper.yaml
        MyLooperBuilderVec::default_channels(20,1), // call the constructor of this process
    )?.add_process(
        &main_addr, format!("power_1"), // add process power_1 to main cluster
        "silx/vec-mono/servants/servant_power_1.yaml", // process definition is serialized within servant_power_1.yaml
        MyPowerBuilderVec::default_channels(2,1), // call the constructor of this process
    )?.add_process(
        &main_addr, format!("power_2"), // add process power_2 to main cluster
        "silx/vec-mono/servants/servant_power_2.yaml", // process definition is serialized within servant_power_2.yaml
        MyPowerBuilderAltVec::default_channels(2,2), // call the constructor of this process
    )?.add_process(
        &main_addr, format!("adder"), // add process adder to main cluster
        "silx/vec-mono/servants/servant_adder.yaml", // process definition is serialized within servant_adder.yaml
        MyAdderBuilderVec::default_channels(), // call the constructor of this process
    )?.add_process(
        &main_addr, format!("shutdown"), // add process shutdown to main cluster
        "silx/vec-mono/servants/servant_shutdown.yaml", // process definition is serialized within servant_shutdown.yaml
        ShutdownBuilder::new(format!("Shutdown")), // call the constructor of this process
    )?.done();

    // add channels to the producer
    Ok(start_prod.add_query( // add query channel within the same cluster
        "silx/vec-mono/channels/channel_pow.yaml", // channel definition is serialized within channel_pow.yaml 
        format!("Power"), // channel name 
        main_addr, // cluster address 
        [format!("looper"),], // input servants 
        [format!("power_1"),format!("power_2"),], // output servants 
        max_ping, Some(16),
    )?.add_broadcast( // add broadcast channel within the same cluster
        "silx/vec-mono/channels/channel_add.yaml", // channel definition is serialized within channel_add.yaml 
        format!("Adder"), // channel name 
        main_addr, // cluster address 
        [format!("looper"),], // input servants
        [format!("adder"),], // output servants 
        max_ping, 16,
    )?.add_broadcast( // add broadcast channel within the same cluster
        "silx/vec-mono/channels/channel_shutdown.yaml", // channel definition is serialized within channel_shutdown.yaml 
        format!("Shutdown"), // channel name 
        main_addr, // cluster address 
        [format!("looper"),], // input servants 
        [format!("shutdown"),], // output servants 
        max_ping, 16,
    )?.done())
}