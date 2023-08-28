/// This is the `manager` module for Docker operations.
///
/// It provides functionalities for starting, stopping the docker-compose as well as generating it to disk.
use std::path::{Path, PathBuf};

use crate::cmd::run_command;
use std::fs::File;
use std::io::Write;

use super::compose::{DockerCompose, ServiceConfig};

pub struct DockerManager {
    pub network_path: PathBuf,
    pub docker_compose_path: PathBuf,
}

impl DockerManager {
    pub fn new(network_path: &Path) -> Self {
        let docker_compose_path = network_path.join("docker-compose.yaml");
        DockerManager {
            network_path: network_path.to_path_buf(),
            docker_compose_path,
        }
    }

    pub fn compose_generate_file(&self, configs: Vec<ServiceConfig>) -> std::io::Result<()> {
        let mut file = File::create(&self.docker_compose_path)?;
        let contents = DockerCompose::generate(configs, &self.network_path);
        file.write_all(contents.as_bytes())?;
        Ok(())
    }

    pub fn compose_up(&self) -> std::io::Result<()> {
        self.run_docker_compose(&["up", "-d"])
    }

    pub fn compose_down(&self) -> std::io::Result<()> {
        self.run_docker_compose(&["down"])
    }

    fn run_docker_compose(&self, subcommands: &[&str]) -> std::io::Result<()> {
        let network_id = self
            .network_path
            .file_name()
            .expect("Failed to extract file name")
            .to_str()
            .expect("Failed to convert OsStr to str");

        let base_args = &[
            "compose",
            "-f",
            self.docker_compose_path
                .to_str()
                .expect("Failed to convert file path to str"),
            "-p",
            network_id,
        ];

        let mut args: Vec<&str> = base_args.to_vec();
        args.extend_from_slice(subcommands);

        let _ = run_command("docker", &args)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
