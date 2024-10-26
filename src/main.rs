use std::thread;

use log::{error, info};
use relayer::common::node_configs::NodeConfiguration;
use relayer::monitor::Monitor;
use relayer::filter::Filter;
use relayer::utils::log_util::{init_logger, LogOutput};

use clap::{App, Arg};


fn main() {
    let config_file_path_arg = Arg::with_name("config_file_path")
        .long("config")
        .value_name("CONFIG_FILE_PATH")
        .takes_value(true)  // 表示接收值
        .required(true)
        .default_value("application.yaml")
        .help("Configuration file to use");

    let log_arg = Arg::with_name("log")
        .long("log")
        .takes_value(false) // 表示不接收值
        .help("Log mode: stream the validator log");

    let matches = App::new("fraud-proof")
        .about("Fraud Proof")
        .version("0.1.0")
        .arg(config_file_path_arg)
        .arg(log_arg)
        .get_matches();

    let output = if matches.is_present("log") {
        LogOutput::Log
    } else {
        LogOutput::None
    };

    init_logger(output);

    let config_file = matches.value_of("config_file_path");
    let default_config_file = String::from("application.yaml");
    let config_file_path = config_file.unwrap_or(&default_config_file);
    info!("config_file_path: {:?}", config_file_path);
    let cfg_result = &NodeConfiguration::load_from_file(config_file_path);
    info!("cfg_result: {:?}", cfg_result.to_owned());

    match cfg_result {
        Err(err) => {
            error!("Load config error {:#?}", &err);
        }
        Ok(cfg) => {
            let store = cfg.store.clone();
            let chain = cfg.chain.clone();
            let contract = cfg.contract.clone();
            // let monitor_store = store.clone();
            // let monitor_chain = chain.clone();

            // let _ = thread::spawn(move || {
            //     let mut monitor = Monitor::new()
            //         .load_chain_config(&monitor_chain)
            //         .load_store_config(&monitor_store);

            //     let _ = monitor.start();
            // });

            let mut filter = Filter::new()
                .store(&store)
                .contract(&contract);

            filter.start();
        }
    }
}



