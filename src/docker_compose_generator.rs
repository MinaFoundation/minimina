use std::path::PathBuf;

use crate::directory_manager::DirectoryManager;
use std::fs::File;
use std::io::Write;

pub struct DockerComposeGenerator {
    directory_manager: DirectoryManager,
}

impl DockerComposeGenerator {
    pub fn new(directory_manager: DirectoryManager) -> Self {
        DockerComposeGenerator { directory_manager }
    }

    pub fn generate_docker_compose(
        &self,
        network_id: &str,
        _topology: &PathBuf,
    ) -> std::io::Result<()> {
        // TODO: Implement actual generation logic based on topology
        let contents = "version: '3.5'\nservices:\n  block-producer:\n    image: gcr.io/o1labs-192920/mina-daemon:2.0.0rampup3-bfd1009-buster-berkeley\n";

        let mut file_path = self.directory_manager.network_path(network_id);
        file_path.push("docker-compose.yaml");

        let mut file = File::create(file_path)?;
        file.write_all(contents.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_generate_docker_compose() {
        let dir_manager = DirectoryManager::_new_with_base_path(
            "/tmp/minimina-testing-docker-compose-gen".into(),
        );
        let network_id = "test_network";

        // Create the network directory
        dir_manager.create_network_directory(network_id).unwrap();

        // Generate a docker-compose file
        let generator = DockerComposeGenerator::new(dir_manager);
        let topology = std::path::PathBuf::from("path/to/topology");
        generator
            .generate_docker_compose(network_id, &topology)
            .unwrap();

        // Check that the file was created and has the correct contents
        let mut file_path = generator.directory_manager._base_path().clone();
        file_path.push(network_id);
        file_path.push("docker-compose.yaml");

        let contents = fs::read_to_string(file_path).unwrap();
        assert!(contents.contains("version: '3.5'"));
        assert!(contents.contains("services:\n  block-producer:"));

        // Clean up
        generator
            .directory_manager
            .delete_network_directory(network_id)
            .unwrap();
    }
}
