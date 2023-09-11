//! # Docker Manager Module
//!
//! Provides an interface for managing Docker operations within the application.
//! Specifically, it offers functionalities to:
//! - Generate a `docker-compose.yaml` file from provided service configurations.
//! - Start up services using the generated Docker Compose file.
//! - Shut down active services.
//! - Handle interactions with the Docker CLI.

use std::path::{Path, PathBuf};
use std::process::Output;

use serde::{Deserialize, Serialize};

use crate::service::ServiceConfig;
use crate::utils::run_command;
use std::fs::File;
use std::io::Write;

use super::compose::DockerCompose;

#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerInfo {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Image")]
    pub image: String,
    #[serde(rename = "Command")]
    pub command: String,
    #[serde(rename = "Project")]
    pub project: String,
    #[serde(rename = "Service")]
    pub service: String,
    #[serde(rename = "Created")]
    pub created: u64,
    #[serde(rename = "State")]
    pub state: String,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "Health")]
    pub health: String,
    #[serde(rename = "ExitCode")]
    pub exit_code: u8,
    #[serde(rename = "Publishers")]
    pub publishers: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComposeInfo {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "ConfigFiles")]
    pub config_files: String,
}

pub struct DockerManager {
    pub network_path: PathBuf,
    pub compose_path: PathBuf,
}

impl DockerManager {
    pub fn new(network_path: &Path) -> Self {
        let compose_path = network_path.join("docker-compose.yaml");
        DockerManager {
            network_path: network_path.to_path_buf(),
            compose_path,
        }
    }

    pub fn compose_generate_file(&self, configs: &[ServiceConfig]) -> std::io::Result<()> {
        let mut file = File::create(&self.compose_path)?;
        let contents = DockerCompose::generate(configs, &self.network_path);
        file.write_all(contents.as_bytes())?;
        Ok(())
    }

    pub fn _compose_up(&self) -> std::io::Result<Output> {
        self.run_docker_compose(&["up", "-d"])
    }

    pub fn compose_down(&self) -> std::io::Result<Output> {
        self.run_docker_compose(&["down", "--volumes", "--remove-orphans", "--rmi", "all"])
    }

    pub fn compose_create(&self) -> std::io::Result<Output> {
        self.run_docker_compose(&["create"])
    }

    pub fn compose_start(&self) -> std::io::Result<Output> {
        self.run_docker_compose(&["start"])
    }

    pub fn compose_stop(&self) -> std::io::Result<Output> {
        self.run_docker_compose(&["stop"])
    }

    pub fn compose_ls(&self) -> std::io::Result<Vec<ComposeInfo>> {
        let output = self.run_docker_compose(&["ls", "--format", "json"])?;
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let compose_info = serde_json::from_str(&stdout_str)?;
        Ok(compose_info)
    }

    pub fn compose_ps(&self) -> std::io::Result<Vec<ContainerInfo>> {
        let output = self.run_docker_compose(&["ps", "--format", "json"])?;
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let containers: Vec<ContainerInfo> = serde_json::from_str(&stdout_str)?;
        Ok(containers)
    }

    fn run_docker_compose(&self, subcommands: &[&str]) -> std::io::Result<Output> {
        let network_id = self
            .network_path
            .file_name()
            .expect("Failed to extract file name")
            .to_str()
            .expect("Failed to convert OsStr to str");

        let base_args = &[
            "compose",
            "-f",
            self.compose_path
                .to_str()
                .expect("Failed to convert file path to str"),
            "-p",
            network_id,
        ];

        let mut args: Vec<&str> = base_args.to_vec();
        args.extend_from_slice(subcommands);

        let out = run_command("docker", &args)?;
        Ok(out)
    }
}
