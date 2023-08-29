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
    pub struct Stop {
        pub network_id: String,
    }

    #[derive(Debug, Serialize)]
    pub struct Status {
        pub network_id: String,
        pub status: String,
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
    pub message: String,
}

impl ServiceConfig {
    pub fn to_node_info(&self) -> node::Info {
        node::Info {
            node_id: self.service_name.clone(),
            client_port: self.client_port,
            graphql_uri: self
                .client_port
                .map(|port| format!("http://localhost:{}/graphql", port + 2)),
            public_key: self.public_key.clone(),
            libp2p_keypair: self.libp2p_keypair.clone(),
            node_type: self.service_type.clone(),
        }
    }
}

pub fn generate_network_create(services: Vec<ServiceConfig>, network_id: &str) -> network::Create {
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
impl_display!(node::Start);
impl_display!(node::Stop);
impl_display!(node::ArchiveData);
impl_display!(node::MinaLogs);
impl_display!(node::PrecomputedBlocks);
impl_display!(node::ReplayerLogs);
impl_display!(Error);
