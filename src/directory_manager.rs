use dirs::home_dir;
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

    pub fn network_path(&self, network_id: &str) -> PathBuf {
        let mut network_path = self.base_path.clone();
        network_path.push(network_id);
        network_path
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
}
