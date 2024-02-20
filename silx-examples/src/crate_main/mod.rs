// ======= SERVANTS DEFINITIONS
/// Definition of the servant which enumerate the scalar 
mod my_looper; pub use my_looper::MyLooperBuilder;
/// Definition of the servant which compute the scalar sum 
mod my_adder; pub use my_adder::MyAdderBuilder;
/// Definition of the servant which compute the scalar power 
mod my_power; pub use my_power::MyPowerBuilder;
/// Alternative definition of the servant which compute the scalar power 
mod my_power_alt; pub use my_power_alt::MyPowerBuilderAlt;
/// Construction of the starters for the scalar case
mod my_starter; pub use my_starter::{ build_my_starter_scalar, build_my_starter_scalar_mono, };
/// Definition of the servant which enumerate the vector 
mod my_looper_vec; pub use my_looper_vec::MyLooperBuilderVec;
/// Definition of the servant which compute the vector sum 
mod my_adder_vec; pub use my_adder_vec::MyAdderBuilderVec;
/// Definition of the servant which compute the vector element-wise power 
mod my_power_vec; pub use my_power_vec::MyPowerBuilderVec;
/// Alternative definition of the servant which compute the vector element-wise power 
mod my_power_alt_vec; pub use my_power_alt_vec::MyPowerBuilderAltVec;
/// Construction of the starters for the vector case
mod my_starter_vec; pub use my_starter_vec::{ build_my_starter_vec, build_my_starter_vec_mono, };

use std::{ net::{ IpAddr, Ipv4Addr, SocketAddr, }, path::PathBuf, time::Duration };
use tokio::{spawn, time::sleep};
use silx_core::utils::{ FiledStarter, RecFiled, Filable, };

/// Example of method for loading a silx network and running it
/// * `starter_path: &str` : path of the starter file
/// * `save_path: &str` : path of saving directory
/// * Output : return nothing of a String error message
pub async fn exp_load_start(starter_path: &str, save_path: &str) -> Result<(), String> {
    // load serialized starter command
    let mut starter =  RecFiled::<FiledStarter>::new_unloaded(
        PathBuf::from(starter_path)
    );
    // load entire starter data
    starter.load(&save_path)?;
    // get the starter
    let starter_in = starter.unwrap()?;
    match starter_in.run().await { // run the starter 
        Ok(()) => Ok(()), 
        Err(e) => Err(format!("Failed (loaded) to run starter: {}",e)), 
    }
}

/// network test example with scalar data (3 clusters)
///   1. Starters are first defined from scratch and saved on disk
///   2. The built starters are run
///     * numbers from 1 to 20 are generated (time-spaced by 1s)
///     * these numbers are raised to the power of 2
///     * results are summed
///   3. The saved starters are loaded and run again
pub async fn exp_silx_scalar() -> Result<(),String> {
    // ======= BUILDING NETWORK FROM SCRATCH AND RUNNING IT
    println!("=================================================");
    println!("==                                             ==");
    println!("==   (a) Starters defined from scratch         ==");
    println!("==   (b) Starters definitions saved on disk    ==");
    println!("==   (c) Running defined network               ==");
    println!("==                                             ==");
    println!("=================================================");
    println!();
    // ======= STARTERS DEFINITIONS
    // define net address of the 3 clusters: main, slave1, slave2
    let main_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8180);
    let slave_1_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8181);
    let slave_2_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8182);
    // set starter paths for main, slave1 and slave2
    let main_starter_path = "silx/scalar/starters/main.yaml";
    let slave_1_starter_path = "silx/scalar/starters/slave_1.yaml";
    let slave_2_starter_path = "silx/scalar/starters/slave_2.yaml";
    // build the starters loaders
    let mut starters = build_my_starter_scalar(
        main_addr, slave_1_addr, slave_2_addr, main_starter_path, slave_1_starter_path, slave_2_starter_path,
    ).unwrap();
    // get each starter loader
    let mut main_starter = starters.remove(&main_addr).unwrap();
    let mut slave_1_starter = starters.remove(&slave_1_addr).unwrap();
    let mut slave_2_starter = starters.remove(&slave_2_addr).unwrap();
    // define save path for the network
    let save_path = PathBuf::from("main_examples_data/saved_data/");
    // save the starters loaders on disk and get the starters
    let main_starter_in = main_starter.unload(Some(&save_path))?;
    let slave_1_starter_in = slave_1_starter.unload(Some(&save_path))?;
    let slave_2_starter_in = slave_2_starter.unload(Some(&save_path))?;

    // ======= RUNNING NETWORK
    println!("Start starters");
    // run slave1 in a spawned process
    let handle_slave_1 = spawn(async move { 
        // NOTA: main starter should be launched FIRST, because it plays as server and TCP stream connection may be blocking
        sleep(Duration::from_millis(100)).await; // so, sleep awhile so that main starts first
        match slave_1_starter_in.run().await {
            Ok(()) => println!("Done handle_slave_1"), Err(e) => println!("Failed to run starter: {}",e),
        }
    });
    // run slave2 in a spawned process
    let handle_slave_2 = spawn(async move { 
        // NOTA: main starter should be launched FIRST, because it plays as server and TCP stream connection may be blocking
        sleep(Duration::from_millis(100)).await; // so, sleep awhile so that main starts first
        match slave_2_starter_in.run().await {
            Ok(()) => println!("Done handle_slave_2"), Err(e) => println!("Failed to run starter: {}",e),
        }
    });
    // run main process
    match main_starter_in.run().await { Ok(()) => println!("Done main"), Err(e) => println!("Failed to run starter: {}",e), };
    // and await other threads to end
    if handle_slave_1.await.is_err() { println!("ERROR slave_starter_2: Handle failure!"); }
    if handle_slave_2.await.is_err() { println!("ERROR slave_starter_2: Handle failure!"); }

    // ======= RUNNING NETWORK BY LOADING IT
    println!("\n");
    println!("=======================================================");
    println!("==                                                   ==");
    println!("==   New run based on definitions loaded from disk   ==");
    println!("==                                                   ==");
    println!("=======================================================");
    println!();
    // load from disk and run slave1 in a spawned process
    let handle_slave_1 = spawn({
        let save_path = save_path.to_str().unwrap().to_string();
        let slave_1_starter_path = slave_1_starter_path.to_string();
        async move {
            // NOTA: main starter should be launched FIRST, because it plays as server and TCP stream connection may be blocking
            sleep(Duration::from_millis(100)).await; // so, sleep awhile so that main starts first
            exp_load_start(&slave_1_starter_path, &save_path).await
        }
    });
    // load from disk and run slave2 in a spawned process
    let handle_slave_2 = spawn({
        let save_path = save_path.to_str().unwrap().to_string();
        let slave_2_starter_path = slave_2_starter_path.to_string();
        async move {
            // NOTA: main starter should be launched FIRST, because it plays as server and TCP stream connection may be blocking
            sleep(Duration::from_millis(100)).await; // so, sleep awhile so that main starts first
            exp_load_start(&slave_2_starter_path, &save_path).await
        }
    });
    // load from disk, run main and wait for result; set message error (empty if no error)
    let result0 = match exp_load_start(&main_starter_path, save_path.to_str().unwrap()).await {
        Ok(()) => String::new(),
        Err(e) => format!("main_starter error: {e}"),
    };
    // wait for result of slave1; chain message erreor if any
    let result1 = match handle_slave_1.await { 
        Ok(Ok(())) => result0,
        Ok(Err(e)) => format!("{result0}\nslave_starter_1 error: {e}"),
        Err(e) => format!("{result0}\nslave_starter_1 handle error: {e}"),
    };
    // wait for result of slave2; chain message erreor if any
    let result2 = match handle_slave_2.await { 
        Ok(Ok(())) => result1,
        Ok(Err(e)) => format!("{result1}\nslave_starter_2 error: {e}"),
        Err(e) => format!("{result1}\nslave_starter_2 handle error: {e}"),
    };
    // publish arrors if any
    if result2.is_empty() { Ok(()) } else { Err(result2) }
}

/// network test example with scalar data (1 cluster)
///   1. Starter is first defined from scratch and saved on disk
///   2. The built starter is run
///     * numbers from 1 to 20 are generated (time-spaced by 1s)
///     * these numbers are raised to the power of 2
///     * results are summed
///   3. The saved starter is loaded and run again
pub async fn exp_silx_scalar_mono() -> Result<(),String> {
    // ======= BUILDING NETWORK FROM SCRATCH AND RUNNING IT
    println!("=====================================================");
    println!("==                                                 ==");
    println!("==   (a) Starters defined from scratch             ==");
    println!("==   (b) Starters definitions saved on disk        ==");
    println!("==   (c) Running defined network                   ==");
    println!("==                                                 ==");
    println!("=====================================================");
    println!();

    // ======= STARTERS DEFINITIONS
    // define net address of cluster main
    let main_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8180);
    // set starter path for main
    let main_starter_path = "silx/scalar-mono/starter/main.yaml";
    // build the starters loaders (actually only main starter)
    let mut starters = build_my_starter_scalar_mono(
        main_addr, main_starter_path,
    ).unwrap();
    // get main starter loader
    let mut main_starter = starters.remove(&main_addr).unwrap();
    // define save path for the network
    let save_path = PathBuf::from("main_examples_data/saved_data/");
    // save the starter loader on disk and get the starters
    let main_starter_in = main_starter.unload(Some(&save_path))?;

    // ======= RUNNING NETWORK
    println!("Start starters");
    // run main process
    match main_starter_in.run().await { Ok(()) => println!("Done main"), Err(e) => println!("Failed to run starter: {}",e), };

    // ======= RUNNING NETWORK BY LOADING IT
    println!("\n");
    println!("=======================================================");
    println!("==                                                   ==");
    println!("==   New run based on definitions loaded from disk   ==");
    println!("==                                                   ==");
    println!("=======================================================");
    println!();
    // load from disk, run main and wait for result; set message error (empty if no error)
    match exp_load_start(&main_starter_path, save_path.to_str().unwrap()).await {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("main_starter error: {e}")),
    }
}


/// network test example with vector data (3 clusters)
///   1. Starters are first defined from scratch and saved on disk
///   2. The built starters are run
///     * 20 vectors are generated starting from \[1,2,3\] with step \[1,2,3\] (time-spaced by 1s)
///     * these vectors are raised element-wise to the power of 2
///     * results are summed
///   3. The saved starters are loaded and run again
pub async fn exp_silx_vec() -> Result<(),String> {
    // ======= BUILDING NETWORK FROM SCRATCH AND RUNNING IT
    println!("=================================================");
    println!("==                                             ==");
    println!("==   (a) Starters defined from scratch         ==");
    println!("==   (b) Starters definitions saved on disk    ==");
    println!("==   (c) Running defined network               ==");
    println!("==                                             ==");
    println!("=================================================");
    println!();

    // ======= STARTERS DEFINITIONS
    // define net address of the 3 clusters: main, slave1, slave2
    let main_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8180);
    let slave_1_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8181);
    let slave_2_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8182);
    // set starter paths for main, slave1 and slave2
    let main_starter_path = "silx/vec/starters/main.yaml";
    let slave_1_starter_path = "silx/vec/starters/slave_1.yaml";
    let slave_2_starter_path = "silx/vec/starters/slave_2.yaml";
    // build the starters loaders
    let mut starters = build_my_starter_vec(
        main_addr, slave_1_addr, slave_2_addr, main_starter_path, slave_1_starter_path, slave_2_starter_path,
    ).unwrap();
    // get each starter loader
    let mut main_starter = starters.remove(&main_addr).unwrap();
    let mut slave_1_starter = starters.remove(&slave_1_addr).unwrap();
    let mut slave_2_starter = starters.remove(&slave_2_addr).unwrap();
    // define save path for the network
    let save_path = PathBuf::from("main_examples_data/saved_data/");
    // save the starters loaders on disk and get the starters
    let main_starter_in = main_starter.unload(Some(&save_path))?;
    let slave_1_starter_in = slave_1_starter.unload(Some(&save_path))?;
    let slave_2_starter_in = slave_2_starter.unload(Some(&save_path))?;

    // ======= RUNNING NETWORK
    println!("Start starters");
    // run slave1 in a spawned process
    let handle_slave_1 = spawn(async move { 
        // NOTA: main starter should be launched FIRST, because it plays as server and TCP stream connection may be blocking
        sleep(Duration::from_millis(100)).await; // so, sleep awhile so that main starts first
        match slave_1_starter_in.run().await {
            Ok(()) => println!("Done handle_slave_1"), Err(e) => println!("Failed to run starter: {}",e),
        }
    });
    // run slave2 in a spawned process
    let handle_slave_2 = spawn(async move { 
        // NOTA: main starter should be launched FIRST, because it plays as server and TCP stream connection may be blocking
        sleep(Duration::from_millis(100)).await; // so, sleep awhile so that main starts first
        match slave_2_starter_in.run().await {
            Ok(()) => println!("Done handle_slave_2"), Err(e) => println!("Failed to run starter: {}",e),
        }
    });
    // run main process
    match main_starter_in.run().await { Ok(()) => println!("Done main"), Err(e) => println!("Failed to run starter: {}",e), };
    // and await other threads to end
    if handle_slave_1.await.is_err() { println!("ERROR slave_starter_2: Handle failure!"); }
    if handle_slave_2.await.is_err() { println!("ERROR slave_starter_2: Handle failure!"); }

    // ======= RUNNING NETWORK BY LOADING IT
    println!("\n");
    println!("=======================================================");
    println!("==                                                   ==");
    println!("==   New run based on definitions loaded from disk   ==");
    println!("==                                                   ==");
    println!("=======================================================");
    println!();
    // load from disk and run slave1 in a spawned process
    let handle_slave_1 = spawn({
        let save_path = save_path.to_str().unwrap().to_string();
        let slave_1_starter_path = slave_1_starter_path.to_string();
        async move {
            // NOTA: main starter should be launched FIRST, because it plays as server and TCP stream connection may be blocking
            sleep(Duration::from_millis(100)).await; // so, sleep awhile so that main starts first
            exp_load_start(&slave_1_starter_path, &save_path).await
        }
    });
    // load from disk and run slave2 in a spawned process
    let handle_slave_2 = spawn({
        let save_path = save_path.to_str().unwrap().to_string();
        let slave_2_starter_path = slave_2_starter_path.to_string();
        async move {
            // NOTA: main starter should be launched FIRST, because it plays as server and TCP stream connection may be blocking
            sleep(Duration::from_millis(100)).await; // so, sleep awhile so that main starts first
            exp_load_start(&slave_2_starter_path, &save_path).await
        }
    });
    // load from disk, run main and wait for result; set message error (empty if no error)
    let result0 = match exp_load_start(&main_starter_path, save_path.to_str().unwrap()).await {
        Ok(()) => String::new(),
        Err(e) => format!("main_starter error: {e}"),
    };
    // wait for result of slave1; chain message erreor if any
    let result1 = match handle_slave_1.await { 
        Ok(Ok(())) => result0,
        Ok(Err(e)) => format!("{result0}\nslave_starter_1 error: {e}"),
        Err(e) => format!("{result0}\nslave_starter_1 handle error: {e}"),
    };
    // wait for result of slave2; chain message erreor if any
    let result2 = match handle_slave_2.await { 
        Ok(Ok(())) => result1,
        Ok(Err(e)) => format!("{result1}\nslave_starter_2 error: {e}"),
        Err(e) => format!("{result1}\nslave_starter_2 handle error: {e}"),
    };
    // publish arrors if any
    if result2.is_empty() { Ok(()) } else { Err(result2) }
}

/// network test example with vector data (1 cluster)
/// network test example with scalar data (1 cluster)
///   1. Starter is first defined from scratch and saved on disk
///   2. The built starter is run
///     * numbers from 1 to 20 are generated (time-spaced by 1s)
///     * these numbers are raised to the power of 2
///     * results are summed
///   3. The saved starter is loaded and run again
pub async fn exp_silx_vec_mono() -> Result<(),String> {
    // ======= BUILDING NETWORK FROM SCRATCH AND RUNNING IT
    println!("=====================================================");
    println!("==                                                 ==");
    println!("==   (a) Starters defined from scratch             ==");
    println!("==   (b) Starters definitions saved on disk        ==");
    println!("==   (c) Running defined network                   ==");
    println!("==                                                 ==");
    println!("=====================================================");
    println!();

    // ======= STARTERS DEFINITIONS
    // define net address of cluster main
    let main_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8180);
    // set starter path for main
    let main_starter_path = "silx/vec-mono/starter/main.yaml";
    // build the starters loaders (actually only main starter)
    let mut starters = build_my_starter_vec_mono(
        main_addr, main_starter_path,
    ).unwrap();
    // get main starter loader
    let mut main_starter = starters.remove(&main_addr).unwrap();
    // define save path for the network
    let save_path = PathBuf::from("main_examples_data/saved_data/");
    // save the starter loader on disk and get the starters
    let main_starter_in = main_starter.unload(Some(&save_path))?;

    // ======= RUNNING NETWORK
    println!("Start starters");
    // run main process
    match main_starter_in.run().await { Ok(()) => println!("Done main"), Err(e) => println!("Failed to run starter: {}",e), };

    // ======= RUNNING NETWORK BY LOADING IT
    println!("\n");
    println!("=======================================================");
    println!("==                                                   ==");
    println!("==   New run based on definitions loaded from disk   ==");
    println!("==                                                   ==");
    println!("=======================================================");
    println!();
    // load from disk, run main and wait for result; set message error (empty if no error)
    match exp_load_start(&main_starter_path, save_path.to_str().unwrap()).await {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("main_starter error: {e}")),
    }
}
