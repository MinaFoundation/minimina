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

use dirs::home_dir;
use log::info;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

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
        let mut network_path = self.base_path.clone();
        network_path.push(network_id);
        network_path
    }

    // list of all subdirectories that needs to be created for the network
    fn subdirectories() -> [&'static str; 2] {
        ["network-keypairs", "libp2p-keypairs"]
    }

    pub fn generate_dir_structure(&self, network_id: &str) -> std::io::Result<PathBuf> {
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

    pub fn create_network_directory(&self, network_id: &str) -> std::io::Result<()> {
        let network_path = self.network_path(network_id);
        std::fs::create_dir_all(network_path)
    }

    pub fn delete_network_directory(&self, network_id: &str) -> std::io::Result<()> {
        let network_path = self.network_path(network_id);
        std::fs::remove_dir_all(network_path)
    }

    pub fn list_network_directories(&self) -> std::io::Result<Vec<String>> {
        let mut networks = vec![];
        for entry in std::fs::read_dir(&self.base_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(network_id) = entry.file_name().to_str() {
                    networks.push(network_id.to_string());
                }
            }
        }
        Ok(networks)
    }

    fn create_subdirectories(&self, network_id: &str) -> std::io::Result<()> {
        for subdirectory in self.subdirectories_paths(network_id) {
            std::fs::create_dir_all(subdirectory)?;
        }
        Ok(())
    }

    fn set_subdirectories_permissions(&self, network_id: &str, mode: u32) -> std::io::Result<()> {
        for subdirectory in self.subdirectories_paths(network_id) {
            std::fs::set_permissions(subdirectory, std::fs::Permissions::from_mode(mode))?;
        }
        Ok(())
    }
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
            let metadata = std::fs::metadata(subdir_path).unwrap();
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
}
