use crate::service::{ServiceConfig, ServiceType};
use serde::Deserialize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

/// Topology info for an archive node
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ArchiveTopologyInfo {
    pub pk: String,
    pub sk: String,
    #[serde(rename(deserialize = "role"))]
    pub service_type: ServiceType,
    pub docker_image: String,
    pub schema_file: PathBuf,
    pub zkapp_table: PathBuf,
}

/// Topology info for a block producer or seed node
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct NodeTopologyInfo {
    pub pk: String,
    pub sk: String,
    #[serde(rename(deserialize = "role"))]
    pub service_type: ServiceType,
    pub docker_image: String,
    pub libp2p_pass: String,
    pub libp2p_keyfile: PathBuf,
    pub libp2p_peerid: String,
}

/// Topology info for a snark coordinator
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SnarkCoordinatorTopologyInfo {
    pub pk: String,
    pub sk: String,
    #[serde(rename(deserialize = "role"))]
    pub service_type: ServiceType,
    pub docker_image: String,
    pub worker_nodes: u16,
    pub snark_worker_fee: String,
}

/// Each node variant's topology info
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum TopologyInfo {
    Archive(ArchiveTopologyInfo),
    Node(NodeTopologyInfo),
    SnarkCoordinator(SnarkCoordinatorTopologyInfo),
}

/// Full network topology
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Topology {
    #[serde(flatten)]
    pub topology: HashMap<String, TopologyInfo>,
}

impl TopologyInfo {
    fn to_service_config(
        &self,
        service_name: String,
        peer_list_file: &Path,
        client_port: u16,
    ) -> ServiceConfig {
        match self {
            TopologyInfo::Archive(archive_info) => ServiceConfig {
                service_type: ServiceType::ArchiveNode,
                service_name,
                docker_image: archive_info.docker_image.clone(),
                client_port: None,
                public_key: Some(archive_info.pk.clone()),
                public_key_path: None,
                private_key: Some(archive_info.sk.clone()),
                private_key_path: None,
                libp2p_keypair: None,
                libp2p_keypair_path: None,
                peers: None,
                peers_list_path: Some(peer_list_file.to_path_buf()),
                snark_coordinator_port: None,
                snark_coordinator_fees: None,
                snark_worker_proof_level: None,
            },
            TopologyInfo::Node(node_info) => ServiceConfig {
                service_type: node_info.service_type.clone(),
                service_name,
                docker_image: node_info.docker_image.clone(),
                client_port: Some(client_port),
                public_key: Some(node_info.pk.clone()),
                public_key_path: None,
                private_key: Some(node_info.sk.clone()),
                private_key_path: None,
                libp2p_keypair: None,
                libp2p_keypair_path: Some(node_info.libp2p_keyfile.clone()),
                peers: None,
                peers_list_path: Some(peer_list_file.to_path_buf()),
                snark_coordinator_port: None,
                snark_coordinator_fees: None,
                snark_worker_proof_level: None,
            },
            TopologyInfo::SnarkCoordinator(snark_info) => ServiceConfig {
                service_type: snark_info.service_type.clone(),
                service_name,
                docker_image: snark_info.docker_image.clone(),
                client_port: None,
                public_key: Some(snark_info.pk.clone()),
                public_key_path: None,
                private_key: Some(snark_info.sk.clone()),
                private_key_path: None,
                libp2p_keypair: None,
                libp2p_keypair_path: None,
                peers: None,
                peers_list_path: Some(peer_list_file.to_path_buf()),
                snark_coordinator_port: Some(7000),
                snark_coordinator_fees: Some(snark_info.snark_worker_fee.clone()),
                snark_worker_proof_level: Some("full".to_string()),
            },
        }
    }
}

impl Topology {
    pub fn new(path: &Path) -> Result<Self, serde_json::Error> {
        let contents = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&contents)
    }

    pub fn services(&self, peer_list_file: &Path) -> Vec<ServiceConfig> {
        let mut client_port = 7070;
        self.topology
            .iter()
            .map(|(service_name, service_info)| {
                client_port += 1;
                service_info.to_service_config(service_name.clone(), peer_list_file, client_port)
            })
            .collect()
    }

    pub fn seeds(&self) -> Vec<NodeTopologyInfo> {
        self.topology
            .values()
            .filter_map(|info| {
                if let TopologyInfo::Node(node_info) = info.clone() {
                    if let ServiceType::Seed = node_info.service_type.clone() {
                        return Some(node_info);
                    }
                }
                None
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_info() {
        let pk = "pub_key".to_string();
        let sk = "priv_key".to_string();
        let role = "Archive_node".to_string();
        let docker_image = "archive-image".to_string();
        let schema_file = "path/to/create_schame.sql".to_string();
        let zkapp_table = "path/to/zkapp_table.sql".to_string();

        let expect: ArchiveTopologyInfo = serde_json::from_str(&format!(
            "{{
                \"pk\": \"{pk}\",
                \"sk\": \"{sk}\",
                \"role\": \"{role}\",
                \"docker_image\": \"{docker_image}\",
                \"schema_file\": \"{schema_file}\",
                \"zkapp_table\": \"{zkapp_table}\"
            }}"
        ))
        .unwrap();

        assert_eq!(
            expect,
            ArchiveTopologyInfo {
                pk,
                sk,
                docker_image,
                service_type: ServiceType::ArchiveNode,
                schema_file: PathBuf::from(schema_file),
                zkapp_table: PathBuf::from(zkapp_table),
            }
        );
    }

    #[test]
    fn test_deserialize_topology() {
        let bp_name = "bp".to_string();
        let pk = "pk0".to_string();
        let sk = "sk0".to_string();
        let service_type = ServiceType::BlockProducer;
        let docker_image = "bp-image".to_string();
        let libp2p_pass = "pwd0".to_string();
        let libp2p_keyfile = PathBuf::from("path/to/bp_keyfile.json");
        let libp2p_peerid = "bp_peerid".to_string();
        let bp_node = NodeTopologyInfo {
            pk,
            sk,
            service_type,
            docker_image,
            libp2p_pass,
            libp2p_keyfile,
            libp2p_peerid,
        };

        let seed_name = "seed".to_string();
        let pk = "pk1".to_string();
        let sk = "sk1".to_string();
        let service_type = ServiceType::Seed;
        let docker_image = "seed-image".to_string();
        let libp2p_pass = "pwd1".to_string();
        let libp2p_keyfile = PathBuf::from("path/to/seed_keyfile.json");
        let libp2p_peerid = "seed_peerid".to_string();
        let seed_node = NodeTopologyInfo {
            pk,
            sk,
            service_type,
            docker_image,
            libp2p_pass,
            libp2p_keyfile,
            libp2p_peerid,
        };

        let snark_name = "snark".to_string();
        let pk = "pk2".to_string();
        let sk = "sk2".to_string();
        let service_type = ServiceType::SnarkCoordinator;
        let docker_image = "snark-image".to_string();
        let worker_nodes = 42;
        let snark_worker_fee = "0.01".to_string();
        let snark_node = SnarkCoordinatorTopologyInfo {
            pk,
            sk,
            service_type,
            docker_image,
            worker_nodes,
            snark_worker_fee,
        };

        let expect: Topology = serde_json::from_str(
            "{
                \"bp\": {
                    \"pk\": \"pk0\",
                    \"sk\": \"sk0\",
                    \"role\": \"Block_producer\",
                    \"docker_image\": \"bp-image\",
                    \"libp2p_pass\": \"pwd0\",
                    \"libp2p_keyfile\": \"path/to/bp_keyfile.json\",
                    \"libp2p_keypair\": \"bp_keypair\",
                    \"libp2p_peerid\": \"bp_peerid\"
                },
                \"seed\": {
                    \"pk\": \"pk1\",
                    \"sk\": \"sk1\",
                    \"role\": \"Seed_node\",
                    \"docker_image\": \"seed-image\",
                    \"libp2p_pass\": \"pwd1\",
                    \"libp2p_keyfile\": \"path/to/seed_keyfile.json\",
                    \"libp2p_keypair\": \"seed_keypair\",
                    \"libp2p_peerid\": \"seed_peerid\"
                },
                \"snark\": {
                    \"pk\": \"pk2\",
                    \"sk\": \"sk2\",
                    \"role\": \"Snark_coordinator\",
                    \"docker_image\": \"snark-image\",
                    \"worker_nodes\": 42,
                    \"snark_worker_fee\": \"0.01\"
                }
            }",
        )
        .unwrap();

        let topology = Topology {
            topology: HashMap::from([
                (bp_name, TopologyInfo::Node(bp_node)),
                (seed_name, TopologyInfo::Node(seed_node)),
                (snark_name, TopologyInfo::SnarkCoordinator(snark_node)),
            ]),
        };

        assert_eq!(expect, topology);
    }

    #[test]
    fn test_deserialize_topology_file() {
        let path = PathBuf::from("./tests/data/example_topology.json");
        let contents = std::fs::read_to_string(path).unwrap();
        let _: Topology = serde_json::from_str(&contents).unwrap();
    }
}
