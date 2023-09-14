//! # Genesis Ledger Module
//!
//! This module provides functionalities to generate a default genesis ledger for a given network.
//! The generated ledger contains basic account structures populated with information from provided service keys.
//! It handles the serialization of the ledger to a formatted JSON structure and saves it as `genesis_ledger.json`.

extern crate chrono;
use chrono::prelude::*;

use log::{debug, info};
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use crate::keys::NodeKey;

#[derive(Serialize)]
struct GenesisLedger {
    genesis: Genesis,
    ledger: Ledger,
}

#[derive(Serialize)]
struct Genesis {
    genesis_state_timestamp: String,
}

#[derive(Serialize)]
struct Ledger {
    name: Option<String>,
    num_accounts: Option<u32>,
    accounts: Vec<Account>,
}

#[derive(Serialize)]
struct Account {
    pk: String,
    sk: Option<String>,
    balance: String,
    delegate: Option<String>,
}

pub mod default {
    use super::*;

    pub struct LedgerGenerator;

    impl LedgerGenerator {
        pub fn generate(
            network_path: &Path,
            bp_keys: &HashMap<String, NodeKey>,
        ) -> std::io::Result<()> {
            info!("Generating default genesis ledger.");
            let accounts: Vec<Account> = bp_keys
                .values()
                .map(|key_info| Account {
                    pk: key_info.key_string.clone(),
                    sk: None,
                    balance: "11550000.000000000".into(),
                    delegate: None,
                })
                .collect();

            let ledger = Ledger {
                name: Some("release".into()),
                num_accounts: Some(250),
                accounts,
            };

            let genesis = Genesis {
                genesis_state_timestamp: current_timestamp(),
            };

            let genesis_ledger = GenesisLedger { genesis, ledger };

            let content = serde_json::to_string_pretty(&genesis_ledger)?;
            debug!("Generated genesis ledger: {}", content);

            // Construct the path to file
            let path = network_path.to_path_buf();
            let path = path.join("genesis_ledger.json");

            // Write content to the output file.
            let mut file = File::create(path)?;
            file.write_all(content.as_bytes())?;

            Ok(())
        }
    }
}

pub fn current_timestamp() -> String {
    let datetime = Local::now();
    datetime.format("%Y-%m-%dT%H:%M:%S%z").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn test_generate_default_ledger() {
        let network_path = PathBuf::from("/tmp");
        let mut bp_keys_map: HashMap<String, NodeKey> = HashMap::new();
        let service_key = NodeKey {
            key_string: "test_key".to_string(),
            key_path_docker: "test_key_path".to_string(),
        };
        bp_keys_map.insert("node0".to_string(), service_key);
        let result = default::LedgerGenerator::generate(&network_path, &bp_keys_map);
        println!("{:?}", result);
        assert!(result.is_ok());

        let path = network_path.to_path_buf();
        let path = path.join("genesis_ledger.json");
        assert!(path.exists());
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("genesis_state_timestamp"));
        assert!(content.contains("release"));
        assert!(content.contains("test_key"));
    }
}
