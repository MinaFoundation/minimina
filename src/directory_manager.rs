//! # DirectoryManager Module
//!
//! This module provides functionalities related to managing directories for the local network.
//! The directory structure will be placed in the user's home directory under `~/.minimina/{network_id}`.
//! The directory structure will contain the following subdirectories and files:
//! - `network-keypairs`: Contains the key pairs for the block producer service.
//! - `libp2p-keypairs`: Contains the key pairs for the libp2p service.
//! - `genesis_ledger.json`: Contains the genesis ledger for the network.
//! - `docker-compose.yml`: Contains the docker compose file for the network.
//! - `network.json`: Contains the network topology representation in JSON format.
//! - `peer_list_file.txt`: Contains the list of libp2p peers for the network.

use crate::service::ServiceConfig;
use dirs::home_dir;
use log::info;
use std::os::unix::fs::PermissionsExt;
use std::{
    fs,
    io::Result,
    path::{Path, PathBuf},
};

pub struct DirectoryManager {
    pub base_path: PathBuf,
    pub subdirectories: [&'static str; 2],
}

impl DirectoryManager {
    pub fn new() -> Self {
        let mut base_path = home_dir().expect("Home directory not found");
        base_path.push(".minimina");
        DirectoryManager {
            base_path,
            subdirectories: Self::subdirectories(),
        }
    }

    // for testing purposes
    pub fn _new_with_base_path(base_path: PathBuf) -> Self {
        DirectoryManager {
            base_path,
            subdirectories: Self::subdirectories(),
        }
    }

    pub fn _base_path(&self) -> &PathBuf {
        &self.base_path
    }

    // return path to network directory
    pub fn network_path(&self, network_id: &str) -> PathBuf {
        let network_path = self.base_path.clone();
        network_path.join(network_id)
    }

    // list of all subdirectories that needs to be created for the network
    fn subdirectories() -> [&'static str; 2] {
        ["network-keypairs", "libp2p-keypairs"]
    }

    pub fn generate_dir_structure(&self, network_id: &str) -> Result<PathBuf> {
        info!(
            "Creating directory structure for network-id '{}'",
            network_id
        );
        self.create_network_directory(network_id)?;
        self.create_subdirectories(network_id)?;
        self.set_subdirectories_permissions(network_id, 0o700)?;
        let np = self.network_path(network_id);
        Ok(np)
    }

    // return paths to all subdirectories for given network
    fn subdirectories_paths(&self, network_id: &str) -> Vec<PathBuf> {
        let mut subdirectories_paths = vec![];
        for subdirectory in &self.subdirectories {
            let mut subdirectory_path = self.base_path.clone();
            subdirectory_path.push(network_id);
            subdirectory_path.push(subdirectory);
            subdirectories_paths.push(subdirectory_path);
        }
        subdirectories_paths
    }

    pub fn network_path_exists(&self, network_id: &str) -> bool {
        let network_path = self.network_path(network_id);
        network_path.exists()
    }

    pub fn create_network_directory(&self, network_id: &str) -> Result<()> {
        let network_path = self.network_path(network_id);
        fs::create_dir_all(network_path)
    }

    pub fn delete_network_directory(&self, network_id: &str) -> Result<()> {
        let network_path = self.network_path(network_id);
        fs::remove_dir_all(network_path)
    }

    pub fn list_network_directories(&self) -> Result<Vec<String>> {
        let mut networks = vec![];
        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(network_id) = entry.file_name().to_str() {
                    networks.push(network_id.to_string());
                }
            }
        }
        Ok(networks)
    }

    fn create_subdirectories(&self, network_id: &str) -> Result<()> {
        for subdirectory in self.subdirectories_paths(network_id) {
            fs::create_dir_all(subdirectory)?;
        }
        Ok(())
    }

    fn set_subdirectories_permissions(&self, network_id: &str, mode: u32) -> Result<()> {
        for subdirectory in self.subdirectories_paths(network_id) {
            fs::set_permissions(subdirectory, fs::Permissions::from_mode(mode))?;
        }
        Ok(())
    }

    /// Copies all network and libp2p keypairs from service paths to the appropriate
    /// network subdirectories and sets permissions
    pub fn copy_all_network_keys(
        &self,
        network_id: &str,
        services: &Vec<ServiceConfig>,
    ) -> Result<()> {
        let network_keys = self.network_path(network_id).join(self.subdirectories[0]);
        let libp2p_keys = self.network_path(network_id).join(self.subdirectories[1]);

        for service in services {
            // copy network keypairs + permissions
            if let Some(network_key_path) = &service.private_key_path {
                let service_network_key = network_keys
                    .clone()
                    .join(format!("{}.json", &service.service_name));

                fs::copy(network_key_path, &service_network_key)?;
                set_key_file_permissions(&service_network_key)?;
            }

            // copy libp2p keypairs + permissions
            if let Some(libp2p_key_path) = &service.libp2p_keypair_path {
                let service_libp2p_key = libp2p_keys
                    .clone()
                    .join(format!("{}.json", &service.service_name));

                fs::copy(libp2p_key_path, &service_libp2p_key)?;
                set_key_file_permissions(&service_libp2p_key)?;
            }
        }

        Ok(())
    }

    pub fn peer_list_file(&self, network_id: &str) -> PathBuf {
        self.network_path(network_id).join("peer_list_file.txt")
    }

    pub fn create_peer_list_file(&self, network_id: &str, peers: &[&ServiceConfig]) -> Result<()> {
        use std::io::Write;

        let peer_list_path = self.peer_list_file(network_id);
        let mut file = fs::File::create(peer_list_path)?;

        for peer in peers {
            let peer_hostname = format!("{}-{}", peer.service_name, network_id);
            let external_port = peer.client_port.unwrap() + 2;
            let libp2p_key = peer.libp2p_peerid.clone().unwrap();
            writeln!(
                file,
                "/dns4/{}/tcp/{}/p2p/{}",
                peer_hostname, external_port, libp2p_key
            )?;
        }

        Ok(())
    }

    /// Checks whether the genesis timestamp is too far in the past.
    pub fn check_genesis_timestamp(&self, network_id: &str) -> Result<()> {
        use chrono::{prelude::*, Duration};

        let network_path = self.network_path(network_id);
        let genesis_ledger_path = network_path.join("genesis_ledger.json");
        let contents = fs::read_to_string(genesis_ledger_path)?;
        let json: serde_json::Value = serde_json::from_str(&contents)?;
        let genesis = json
            .get("genesis")
            .expect("'genesis' field should be present in genesis ledger");
        let genesis_timestamp = DateTime::parse_from_rfc3339(
            genesis
                .get("genesis_state_timestamp")
                .expect("'genesis_state_timestamp' should be a field in 'genesis' object")
                .as_str()
                .unwrap(),
        )
        .unwrap();

        // use k genesis parameter to calculate cutoff time
        // in case k is not present, use default value of 20
        let k = match genesis.get("k") {
            Some(k) => k.to_string().parse::<u32>().unwrap(),
            None => 20 as u32,
        };
        let cutoff = Local::now()
            .checked_sub_signed(Duration::minutes((k / 2 * 3) as i64))
            .unwrap();

        // if we're outside of the first half of the first transition frontier,
        // we throw an error
        if cutoff > genesis_timestamp {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Genesis timestamp '{genesis_timestamp}' may be outdated."),
            ));
        }

        Ok(())
    }

    /// Copies the genesis ledger at `genesis_ledger_path` to the network directory
    pub fn copy_genesis_ledger(&self, network_id: &str, genesis_ledger_path: &Path) -> Result<()> {
        let network_genesis_path = self.genesis_ledger_path(network_id);
        fs::copy(genesis_ledger_path, network_genesis_path).map(|_| ())
    }

    pub fn overwrite_genesis_timestamp(
        &self,
        network_id: &str,
        genesis_ledger_path: &Path,
    ) -> Result<()> {
        use crate::genesis_ledger::current_timestamp;
        use fs::{read_to_string, write};

        let contents = read_to_string(genesis_ledger_path)?;
        let mut ledger: serde_json::Value = serde_json::from_str(&contents)?;
        let genesis = ledger.get_mut("genesis").unwrap();
        let timestamp = genesis.get_mut("genesis_state_timestamp").unwrap();

        *timestamp = serde_json::Value::String(current_timestamp());

        let contents = serde_json::to_string_pretty(&ledger)?;
        write(self.genesis_ledger_path(network_id), contents)
    }

    /// Returns the genesis ledger path for the given network
    pub fn genesis_ledger_path(&self, network_id: &str) -> PathBuf {
        self.network_path(network_id).join("genesis_ledger.json")
    }

    /// Returns the network file path for the given network
    pub fn network_file_path(&self, network_id: &str) -> PathBuf {
        self.network_path(network_id).join("network.json")
    }

    /// Returns the topology file path for the given network
    pub fn topology_file_path(&self, network_id: &str) -> PathBuf {
        self.network_path(network_id).join("topology.json")
    }
}

fn set_key_file_permissions(file: &Path) -> Result<()> {
    fs::set_permissions(file, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_delete_network_directory() {
        let dir_manager = DirectoryManager::_new_with_base_path(
            "/tmp/test_create_and_delete_network_directory-testing".into(),
        );
        let network_id = "test_network";

        // Create the network directory
        dir_manager.create_network_directory(network_id).unwrap();
        let network_path = dir_manager.network_path(network_id);
        assert!(network_path.exists());

        // Delete the network directory
        dir_manager.delete_network_directory(network_id).unwrap();
        assert!(!network_path.exists());
    }

    #[test]
    fn test_create_subdirectories() {
        let dir_manager =
            DirectoryManager::_new_with_base_path("/tmp/test_create_subdirectories-testing".into());
        let network_id = "test_network";
        let subdirectories = dir_manager.subdirectories;

        // Create the network and subdirectories
        dir_manager.create_network_directory(network_id).unwrap();
        dir_manager.create_subdirectories(network_id).unwrap();

        for subdir in &subdirectories {
            let mut subdir_path = dir_manager._base_path().clone();
            subdir_path.push(network_id);
            subdir_path.push(subdir);
            assert!(subdir_path.exists());
        }

        // Clean up
        dir_manager.delete_network_directory(network_id).unwrap();
    }

    #[test]
    fn test_list_networks() {
        let dir_manager =
            DirectoryManager::_new_with_base_path("/tmp/test_list_networks-testing".into());

        let network_ids = ["test_network1", "test_network2"];

        // Create some network directories
        for network_id in &network_ids {
            dir_manager.create_network_directory(network_id).unwrap();
        }

        // Check that all network directories are listed
        let listed_networks = dir_manager.list_network_directories().unwrap();
        for network_id in &network_ids {
            assert!(listed_networks.contains(&network_id.to_string()));
        }

        // Clean up
        for network_id in &network_ids {
            dir_manager.delete_network_directory(network_id).unwrap();
        }
    }

    #[test]
    fn test_chmod_network_subdirectories() {
        let dir_manager = DirectoryManager::_new_with_base_path(
            "/tmp/test_chmod_network_subdirectories-testing".into(),
        );
        let network_id = "test_network";
        let subdirectories = dir_manager.subdirectories;

        // Create the network and subdirectories
        dir_manager.create_network_directory(network_id).unwrap();
        dir_manager.create_subdirectories(network_id).unwrap();
        // Set readonly permissions
        dir_manager
            .set_subdirectories_permissions(network_id, 0o444)
            .unwrap();

        // Check that the subdirectories have readonly permissions
        for subdir in &subdirectories {
            let mut subdir_path = dir_manager._base_path().clone();
            subdir_path.push(network_id);
            subdir_path.push(subdir);
            let metadata = fs::metadata(subdir_path).unwrap();
            assert!(metadata.permissions().readonly());
        }

        // Clean up
        dir_manager.delete_network_directory(network_id).unwrap();
    }

    #[test]
    fn test_network_subdirectories_paths() {
        let dir_manager = DirectoryManager::_new_with_base_path(
            "/tmp/test_network_subdirectories_paths-testing".into(),
        );
        let network_id = "test_network";
        let subdirectories = dir_manager.subdirectories;
        let paths = dir_manager.subdirectories_paths(network_id);

        for (path, subdir) in paths.iter().zip(&subdirectories) {
            let mut subdir_path = dir_manager._base_path().clone();
            subdir_path.push(network_id);
            subdir_path.push(subdir);
            assert_eq!(path, &subdir_path);
        }
    }

    #[test]
    fn test_check_genesis_timestamp() -> Result<()> {
        use chrono::{prelude::*, Duration};

        let base_path = "/tmp/test_check_genesis_timestamp";
        let network_id = "test_network";
        let dir_manager = DirectoryManager::_new_with_base_path(base_path.into());
        let genesis_ledger_path = dir_manager
            .network_path(network_id)
            .join("genesis_ledger.json");
        fs::create_dir_all(PathBuf::from(base_path).join(network_id))?;

        let k = 20;
        let now = Local::now();
        let old_time = now
            .checked_sub_signed(Duration::minutes(k / 2 * 3 + 1))
            .unwrap()
            .format("%Y-%m-%dT%H:%M:%S%.6f%Z");
        let recent_time = now
            .checked_sub_signed(Duration::minutes(k / 2 * 3 - 1))
            .unwrap()
            .format("%Y-%m-%dT%H:%M:%S%.6f%Z");

        let old_genesis = format!(
            "{{
                \"genesis\": {{
                    \"k\": {k},
                    \"genesis_state_timestamp\": \"{old_time}\"
                }}
            }}"
        );
        let recent_genesis = format!(
            "{{
                \"genesis\": {{
                    \"k\": {k},
                    \"genesis_state_timestamp\": \"{recent_time}\"
                }}
            }}",
        );

        println!("Old:    {old_time}");
        println!("Recent: {recent_time}");

        // genesis ledger is too old so the timestamp will be overwritten
        fs::write(genesis_ledger_path.clone(), old_genesis.clone())?;
        assert!(dir_manager.check_genesis_timestamp(network_id).is_err());

        // genesis ledger is recent enough so the timestamp will not be overwritten
        fs::write(genesis_ledger_path.clone(), recent_genesis.clone())?;
        assert!(dir_manager.check_genesis_timestamp(network_id).is_ok());

        dir_manager.delete_network_directory(network_id)?;

        Ok(())
    }
}
