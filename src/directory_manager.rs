use dirs::home_dir;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

pub struct DirectoryManager {
    base_path: PathBuf,
}

impl DirectoryManager {
    pub fn new() -> Self {
        let mut base_path = home_dir().expect("Home directory not found");
        base_path.push(".minimina");

        DirectoryManager { base_path }
    }

    // for testing purposes
    pub fn _new_with_base_path(base_path: PathBuf) -> Self {
        DirectoryManager { base_path }
    }

    pub fn _base_path(&self) -> &PathBuf {
        &self.base_path
    }

    pub fn network_path(&self, network_id: &str) -> PathBuf {
        let mut network_path = self.base_path.clone();
        network_path.push(network_id);
        network_path
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

    pub fn create_subdirectories(
        &self,
        network_id: &str,
        subdirectories: &[&str],
    ) -> std::io::Result<()> {
        for subdirectory in subdirectories {
            let mut subdirectory_path = self.base_path.clone();
            subdirectory_path.push(network_id);
            subdirectory_path.push(subdirectory);
            std::fs::create_dir_all(subdirectory_path)?;
        }
        Ok(())
    }

    pub fn chmod_network_subdirectories(
        &self,
        network_id: &str,
        subdirectories: &[&str],
        mode: u32,
    ) -> std::io::Result<()> {
        for subdirectory in subdirectories {
            let mut subdirectory_path = self.base_path.clone();
            subdirectory_path.push(network_id);
            subdirectory_path.push(subdirectory);
            std::fs::set_permissions(subdirectory_path, std::fs::Permissions::from_mode(mode))?;
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
        let subdirectories = ["subdir1", "subdir2"];

        // Create the network and subdirectories
        dir_manager.create_network_directory(network_id).unwrap();
        dir_manager
            .create_subdirectories(network_id, &subdirectories)
            .unwrap();

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
        let subdirectories = ["subdir1", "subdir2"];

        // Create the network and subdirectories
        dir_manager.create_network_directory(network_id).unwrap();
        dir_manager
            .create_subdirectories(network_id, &subdirectories)
            .unwrap();
        // Set readonly permissions
        dir_manager
            .chmod_network_subdirectories(network_id, &subdirectories, 0o444)
            .unwrap();

        // Check that the subdirectories have readonly permissions
        for subdir in &subdirectories {
            let mut subdir_path = dir_manager._base_path().clone();
            subdir_path.push(network_id);
            subdir_path.push(subdir);
            let metadata = std::fs::metadata(subdir_path).unwrap();
            assert_eq!(metadata.permissions().readonly(), true);
        }

        // Clean up
        dir_manager.delete_network_directory(network_id).unwrap();
    }
}
