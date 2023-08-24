use std::path::{Path, PathBuf};

use crate::cmd::run_command;
use std::fs::File;
use std::io::Write;

pub struct DockerCompose {
    pub network_path: PathBuf,
    pub docker_compose_path: PathBuf,
}

impl DockerCompose {
    pub fn new(network_path: &Path) -> Self {
        let docker_compose_path = network_path.join("docker-compose.yaml");
        DockerCompose {
            network_path: network_path.to_path_buf(),
            docker_compose_path,
        }
    }

    pub fn _generate_docker_compose(&self) -> std::io::Result<()> {
        // TODO: Implement actual generation logic based on topology
        let contents = "version: '3.5'\nservices:\n  block-producer:\n    image: gcr.io/o1labs-192920/mina-daemon:2.0.0rampup3-bfd1009-buster-berkeley\n";

        let mut file = File::create(&self.docker_compose_path)?;
        file.write_all(contents.as_bytes())?;
        Ok(())
    }

    pub fn run_docker_compose(&self, subcommands: &[&str]) -> std::io::Result<()> {
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
