use crate::service::{ServiceConfig, ServiceType};
use std::collections::HashMap;

pub mod network {
    use serde::Serialize;

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
    }

    impl Status {
        pub fn new(network_id: &str) -> Self {
            Status {
                network_id: network_id.to_string(),
                status: "unknown".to_string(),
            }
        }

        /// Parse the output of `docker compose ls` to get the status of the network
        /// Output:
        /// NAME                STATUS              CONFIG FILES
        /// default             running(5)          /home/piotr/.minimina/default/docker-compose.yaml
        /// test5               running(1)          /home/piotr/.minimina/test5/docker-compose.yaml
        pub fn set_status_from_output(&self, output: &str) -> Option<Status> {
            // Split the output into lines
            let lines: Vec<&str> = output.lines().collect();

            // Search for the line that starts with the given network_id
            let data_line = lines
                .iter()
                .find(|&&line| line.split_whitespace().next() == Some(&self.network_id))?;

            let parts: Vec<&str> = data_line.split_whitespace().collect();

            // Make sure the line has enough parts to parse
            if parts.len() < 3 {
                return None;
            }

            // Extract the status from the line
            let status = parts[1].to_string();

            Some(Status {
                network_id: self.network_id.to_string(),
                status,
            })
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
impl_display!(node::Start);
impl_display!(node::Stop);
impl_display!(node::ArchiveData);
impl_display!(node::MinaLogs);
impl_display!(node::PrecomputedBlocks);
impl_display!(node::ReplayerLogs);
impl_display!(Error);
