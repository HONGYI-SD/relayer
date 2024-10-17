use config::{Config, ConfigError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeConfiguration {
    pub chain: ChainConfiguration,
    pub store: StoreConfiguration,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChainConfiguration {
    pub url: String,
    pub fraud_proof_native_program_id: String,
    // keypair base58 string
    pub execute_keypair: String,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoreConfiguration {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub schema: String,
}


impl NodeConfiguration {
    pub fn load_from_file(file_name: &str) -> Result<NodeConfiguration, ConfigError> {
        let config = Config::builder()
            .add_source(config::File::with_name(file_name))
            .build()?;

        config.try_deserialize::<NodeConfiguration>()
    }
}

#[test]
fn test_null_path() {
    let config_file = "";
    let cfg_result = NodeConfiguration::load_from_file(config_file.clone());
    println!("cfg_result: {:?}", cfg_result);
    assert!(cfg_result.is_err());
}
