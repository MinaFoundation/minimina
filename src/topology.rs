use crate::service::ServiceType;
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};

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
}

/// Topology info for a snark worker
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SnarkWorkerTopologyInfo {
    pub pk: String,
    pub sk: String,
    #[serde(rename(deserialize = "role"))]
    pub service_type: ServiceType,
    pub docker_image: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum TopologyInfo {
    Archive(ArchiveTopologyInfo),
    Node(NodeTopologyInfo),
    SnarkCoordinator(SnarkCoordinatorTopologyInfo),
    SnarkWorker(SnarkWorkerTopologyInfo),
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Topology {
    #[serde(flatten)]
    pub topology: HashMap<String, TopologyInfo>,
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
        let bp_node = NodeTopologyInfo {
            pk,
            sk,
            service_type,
            docker_image,
            libp2p_pass,
            libp2p_keyfile,
        };

        let seed_name = "seed".to_string();
        let pk = "pk1".to_string();
        let sk = "sk1".to_string();
        let service_type = ServiceType::Seed;
        let docker_image = "seed-image".to_string();
        let libp2p_pass = "pwd1".to_string();
        let libp2p_keyfile = PathBuf::from("path/to/seed_keyfile.json");
        let seed_node = NodeTopologyInfo {
            pk,
            sk,
            service_type,
            docker_image,
            libp2p_pass,
            libp2p_keyfile,
        };

        let snark_name = "snark".to_string();
        let pk = "pk2".to_string();
        let sk = "sk2".to_string();
        let service_type = ServiceType::SnarkCoordinator;
        let docker_image = "snark-image".to_string();
        let worker_nodes = 42;
        let snark_node = SnarkCoordinatorTopologyInfo {
            pk,
            sk,
            service_type,
            docker_image,
            worker_nodes,
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
                    \"worker_nodes\": 42
                }
            }",
        )
        .unwrap();

        assert_eq!(
            expect,
            Topology {
                topology: HashMap::from([
                    (bp_name, TopologyInfo::Node(bp_node)),
                    (seed_name, TopologyInfo::Node(seed_node)),
                    (snark_name, TopologyInfo::SnarkCoordinator(snark_node)),
                ]),
            }
        );
    }
}
