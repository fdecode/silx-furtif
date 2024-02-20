/// Definition of servants and starters for the experiments; definition of the experimentations commands
pub mod crate_main; 
use crate::crate_main::exp_load_start;

use self::crate_main::{ exp_silx_scalar, exp_silx_scalar_mono, exp_silx_vec, exp_silx_vec_mono, };



#[tokio::main]
/// Main method for the tests of silx examples. Options are available as command arguments
/// * exp_load_start STARTER_PATH COMPONENTS_DIR : execution of asynchroneous network loaded from file STARTER_PATH within directory COMPONENTS_DIR
///   - example: command `cargo run --bin silx-examples --release -- exp_load_start silx\vec-mono\starter\main.yaml  .\main_examples_data\saved_data\` 
///     will load and run cluster defined in file `.\main_examples_data\saved_data\silx\vec-mono\starter\main.yaml` 
/// * exp_silx_vec : (default) execution of asynchroneous network for computing vectorial sequence
/// * exp_silx_vec_mono : execution of one-cluster asynchroneous network for computing vectorial sequence
/// * exp_silx_scalar : execution of asynchroneous network for computing scalar (`f64slx`) sequence
/// * exp_silx_scalar_mono : execution of one-cluster asynchroneous network for computing scalar (`f64slx`) sequence
pub async fn main() {
    println!("Available paralelism -> {:?}",std::thread::available_parallelism());
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        1 => exp_silx_vec().await.unwrap(), 
        2 => {
            match args[1].as_str() {
                "exp_silx_scalar" => exp_silx_scalar().await.unwrap(),
                "exp_silx_scalar_mono" => exp_silx_scalar_mono().await.unwrap(),
                "exp_silx_vec" => exp_silx_vec().await.unwrap(),
                "exp_silx_vec_mono" => exp_silx_vec_mono().await.unwrap(),
                _ => panic!("bad argument"),
            }    
        }, 
        4 => {
            match (args[1].as_str(),args[2].as_str(),args[3].as_str()) {
                ("exp_load_start", starter_path, components_dir) => {
                    exp_load_start(starter_path, components_dir).await.unwrap();
                },
                _ => panic!("Bad arguments"),        
            }
        }   
        _ => panic!("Bad arguments")        
    }
}
