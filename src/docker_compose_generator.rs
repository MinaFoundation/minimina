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
