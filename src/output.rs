use crate::service::{ServiceConfig, ServiceType};
use std::collections::HashMap;

pub mod network {
    use serde::Serialize;

    use crate::docker::manager::{ComposeInfo, ContainerInfo};

    #[derive(Debug, Serialize)]
    pub struct Create {
        pub network_id: String,
        pub node_map: std::collections::HashMap<String, super::node::Info>,
    }

    #[derive(Debug, Serialize)]
    pub struct Start {
        pub network_id: String,
    }

    #[derive(Debug, Serialize)]
    pub struct ListInfo {
        pub network_id: String,
        pub config_dir: String,
    }

    #[derive(Debug, Serialize)]
    pub struct List {
        pub networks: Vec<ListInfo>,
    }

    impl List {
        pub fn new() -> Self {
            List { networks: vec![] }
        }

        pub fn update(&mut self, networks: Vec<String>, base_dir: &str) {
            for network in networks {
                let config_dir = format!("{}/{}", base_dir, network);
                self.add_network(network, config_dir.as_str());
            }
        }

        pub fn add_network(&mut self, network_id: String, config_dir: &str) {
            self.networks.push(ListInfo {
                network_id,
                config_dir: config_dir.to_string(),
            });
        }
    }

    #[derive(Debug, Serialize)]
    pub struct Stop {
        pub network_id: String,
    }

    #[derive(Debug, Serialize)]
    pub struct Status {
        pub network_id: String,
        pub status: String,
        pub docker_compose_file: String,
        pub nodes: Vec<super::node::Status>,
    }

    impl Status {
        pub fn new(network_id: &str) -> Self {
            Status {
                network_id: network_id.to_string(),
                status: "unknown".to_string(),
                docker_compose_file: "unknown".to_string(),
                nodes: vec![],
            }
        }

        /// Parse the output of `docker compose ls --format json` to get the status of the network
        pub fn update_from_compose_ls(
            &mut self,
            ls_out: Vec<ComposeInfo>,
            compose_file_path: &str,
        ) {
            // get status and config_files of network for compose info where name == network_id
            let network_status = ls_out
                .iter()
                .find(|compose_info| compose_info.name == self.network_id)
                .map(|compose_info| compose_info.status.clone())
                .unwrap_or_else(|| "not_running".to_string());

            let config_files = ls_out
                .iter()
                .find(|compose_info| compose_info.name == self.network_id)
                .map(|compose_info| compose_info.config_files.clone())
                .unwrap_or_else(|| compose_file_path.to_string());

            self.status = network_status;
            self.docker_compose_file = config_files;
        }

        /// Parse the output of `docker compose ps --format json` to get the status of the nodes
        pub fn update_from_compose_ps(&mut self, ps_out: Vec<ContainerInfo>) {
            ps_out.iter().for_each(|container| {
                let node_id = container.name.clone();
                let status = container.status.clone();
                let command = container.command.clone();
                let docker_image = container.image.clone();
                let state = container.state.clone();
                self.nodes.push(super::node::Status {
                    node_id,
                    state,
                    status,
                    command,
                    docker_image,
                });
            });
        }
    }

    #[derive(Debug, Serialize)]
    pub struct Delete {
        pub network_id: String,
    }
}

pub mod node {
    // Import ServiceType from service module
    use super::ServiceType;
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    pub struct Info {
        pub node_id: String,
        pub client_port: Option<u16>,
        pub graphql_uri: Option<String>,
        pub public_key: Option<String>,
        pub libp2p_keypair: Option<String>,
        pub node_type: ServiceType,
    }

    #[derive(Debug, Serialize)]
    pub struct Status {
        pub node_id: String,
        pub state: String,
        pub status: String,
        pub command: String,
        pub docker_image: String,
    }

    #[derive(Debug, Serialize)]
    pub struct Start {
        pub fresh_state: bool,
        pub network_id: String,
        pub node_id: String,
    }

    #[derive(Debug, Serialize)]
    pub struct Stop {
        pub network_id: String,
        pub node_id: String,
    }

    #[derive(Debug, Serialize)]
    pub struct ArchiveData {
        pub data: String,
        pub network_id: String,
        pub node_id: String,
    }

    #[derive(Debug, Serialize)]
    pub struct MinaLogs {
        pub logs: String,
        pub network_id: String,
        pub node_id: String,
    }

    #[derive(Debug, Serialize)]
    pub struct PrecomputedBlocks {
        pub blocks: String,
        pub network_id: String,
        pub node_id: String,
    }

    #[derive(Debug, Serialize)]
    pub struct ReplayerLogs {
        pub logs: String,
        pub network_id: String,
        pub node_id: String,
    }
}

#[derive(Debug, serde::Serialize)]
pub struct Error {
    pub error_message: String,
}

impl ServiceConfig {
    pub fn to_node_info(&self) -> node::Info {
        node::Info {
            node_id: self.service_name.clone(),
            client_port: self.client_port,
            graphql_uri: self
                .client_port
                .map(|port| format!("http://localhost:{}/graphql", port + 1)),
            public_key: self.public_key.clone(),
            libp2p_keypair: self.libp2p_keypair.clone(),
            node_type: self.service_type.clone(),
        }
    }
}

pub fn generate_network_info(services: Vec<ServiceConfig>, network_id: &str) -> network::Create {
    let mut node_map: HashMap<String, node::Info> = HashMap::new();
    for (i, service) in services.iter().enumerate() {
        let node_name = format!("node{}", i);
        node_map.insert(node_name, service.to_node_info());
    }

    network::Create {
        network_id: network_id.to_string(),
        node_map,
    }
}

macro_rules! impl_display {
    ($name:path) => {
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", serde_json::to_string_pretty(self).unwrap())?;
                Ok(())
            }
        }
    };
}

impl_display!(network::Create);
impl_display!(network::Start);
impl_display!(network::Stop);
impl_display!(network::Status);
impl_display!(network::ListInfo);
impl_display!(network::List);
impl_display!(network::Delete);
impl_display!(node::Start);
impl_display!(node::Stop);
impl_display!(node::ArchiveData);
impl_display!(node::MinaLogs);
impl_display!(node::PrecomputedBlocks);
impl_display!(node::ReplayerLogs);
impl_display!(node::Status);
impl_display!(Error);
