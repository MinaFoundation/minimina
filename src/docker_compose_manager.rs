use std::path::Path;

use crate::cmd::run_command;
use crate::directory_manager::DirectoryManager;
use std::fs::File;
use std::io::Write;

pub struct DockerComposeManager {
    directory_manager: DirectoryManager,
}

impl DockerComposeManager {
    pub fn new(directory_manager: DirectoryManager) -> Self {
        DockerComposeManager { directory_manager }
    }

    pub fn generate_docker_compose(
        &self,
        network_id: &str,
        _topology: &Path,
    ) -> std::io::Result<()> {
        // TODO: Implement actual generation logic based on topology
        let contents = "version: '3.5'\nservices:\n  block-producer:\n    image: gcr.io/o1labs-192920/mina-daemon:2.0.0rampup3-bfd1009-buster-berkeley\n";

        let mut file_path = self.directory_manager.network_path(network_id);
        file_path.push("docker-compose.yaml");

        let mut file = File::create(file_path)?;
        file.write_all(contents.as_bytes())?;
        Ok(())
    }

    pub fn run_docker_compose(
        &self,
        network_id: &str,
        subcommands: &[&str],
    ) -> std::io::Result<()> {
        let mut file_path = self.directory_manager.network_path(network_id);
        file_path.push("docker-compose.yaml");

        let base_args = &[
            "compose",
            "-f",
            file_path
                .to_str()
                .expect("Failed to convert file path to str"),
            "-p",
            network_id,
        ];

        let mut args: Vec<&str> = base_args.iter().cloned().collect();
        args.extend_from_slice(subcommands);

        let output = run_command("docker", &args)?;

        println!("status: {}", output.status);
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

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
        let generator = DockerComposeManager::new(dir_manager);
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
