extern crate chrono;
use chrono::prelude::*;

use log::{debug, info};
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use crate::keys::ServiceKeys;

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
    name: &'static str,
    num_accounts: u32,
    accounts: Vec<Account>,
}

#[derive(Serialize)]
struct Account {
    pk: String,
    sk: Option<()>,
    balance: &'static str,
    delegate: Option<()>,
}

pub struct DefaultLedgerGenerator;

impl DefaultLedgerGenerator {
    fn generate_current_timestamp() -> String {
        let datetime = Local::now();
        datetime.format("%Y-%m-%dT%H:%M:%S%z").to_string()
    }

    pub fn generate(
        network_path: &Path,
        bp_keys: &HashMap<String, ServiceKeys>,
    ) -> std::io::Result<()> {
        info!("Generating default genesis ledger.");
        let accounts: Vec<Account> = bp_keys
            .values()
            .map(|key_info| Account {
                pk: key_info.key_string.clone(),
                sk: None,
                balance: "11550000.000000000",
                delegate: None,
            })
            .collect();

        let ledger = Ledger {
            name: "release",
            num_accounts: 250,
            accounts,
        };

        let genesis = Genesis {
            genesis_state_timestamp: Self::generate_current_timestamp(),
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
