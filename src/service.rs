//! # Service Module
//!
//! This module provides structures and methods to hold and manage configurations for different Mina daemons.
//! With these configurations, docker-compose files can be dynamically generated to deploy and manage nodes in the network.

use log::warn;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub enum ServiceType {
    #[serde(rename = "Seed_node")]
    Seed,
    #[default]
    #[serde(rename = "Block_producer")]
    BlockProducer,
    #[serde(rename = "Snark_worker")]
    SnarkWorker,
    #[serde(rename = "Snark_coordinator")]
    SnarkCoordinator,
    #[serde(rename = "Archive_node")]
    ArchiveNode,
}

#[derive(Debug, Clone, Default)]
pub struct ServiceConfig {
    pub service_type: ServiceType,
    pub service_name: String,
    pub docker_image: String,
    pub client_port: Option<u16>,
    pub public_key: Option<String>,
    pub public_key_path: Option<String>,
    pub private_key: Option<String>,
    /// Path to the privkey file used by `mina daemon --block-producer-key KEYFILE ...`
    pub private_key_path: Option<PathBuf>,
    pub libp2p_keypair: Option<String>,
    /// Path to the libp2p keyfile used by `mina daemon --libp2p-keypair KEYFILE ...`
    pub libp2p_keypair_path: Option<PathBuf>,
    pub peers: Option<Vec<String>>,
    /// Path to the file used by `mina daemon --peer-list-file PATH ...`
    pub peers_list_path: Option<PathBuf>,

    //snark coordinator specific
    pub snark_coordinator_port: Option<u16>,
    pub snark_coordinator_fees: Option<String>,

    //snark worker specific
    pub snark_worker_proof_level: Option<String>,
}

impl ServiceConfig {
    // helper function to generate peers list based on libp2p keypair list and external port
    pub fn generate_peers(libp2p_keypairs: Vec<String>, external_port: u16) -> Vec<String> {
        libp2p_keypairs
            .into_iter()
            .filter_map(|s| s.split(',').last().map(|s| s.to_string()))
            .map(|last_key| format!("/ip4/127.0.0.1/tcp/{}/p2p/{}", external_port, last_key))
            .collect()
    }

    // generate base daemon command common for most mina services
    pub fn generate_base_command(&self) -> Vec<String> {
        let client_port = self.client_port.unwrap_or(3100);
        let rest_port = client_port + 1;
        let external_port = rest_port + 1;
        let metrics_port = external_port + 1;
        let libp2p_metrics_port = metrics_port + 1;

        vec![
            "daemon".to_string(),
            "-client-port".to_string(),
            client_port.to_string(),
            "-rest-port".to_string(),
            rest_port.to_string(),
            "-insecure-rest-server".to_string(),
            "-external-port".to_string(),
            external_port.to_string(),
            "-metrics-port".to_string(),
            metrics_port.to_string(),
            "-libp2p-metrics-port".to_string(),
            libp2p_metrics_port.to_string(),
            "-config-file".to_string(),
            "/local-network/genesis_ledger.json".to_string(),
            "-log-json".to_string(),
            "-log-level".to_string(),
            "Trace".to_string(),
            "-file-log-level".to_string(),
            "Trace".to_string(),
            "-config-directory".to_string(),
            format!("/config-directory/{}", self.service_name),
        ]
    }

    // generate command for seed node
    pub fn generate_seed_command(&self) -> String {
        assert_eq!(self.service_type, ServiceType::Seed);

        let mut base_command = self.generate_base_command();
        base_command.push("-seed".to_string());

        self.add_libp2p_command(&mut base_command);
        base_command.join(" ")
    }

    // generate command for block producer node
    pub fn generate_block_producer_command(&self) -> String {
        assert_eq!(self.service_type, ServiceType::BlockProducer);

        let mut base_command = self.generate_base_command();

        // Handling multiple peers
        self.add_peers_command(&mut base_command);

        if let Some(private_key_path) = &self.private_key_path {
            base_command.push("-block-producer-key".to_string());
            base_command.push(private_key_path.to_str().unwrap().to_string());
        } else if let Some(public_key_path) = &self.public_key_path {
            base_command.push("-block-producer-key".to_string());
            base_command.push(public_key_path.clone());
        } else {
            warn!(
                "No public or private key path provided for block producer node '{}'. This is not recommended.",
                self.service_name
            );
        }

        self.add_libp2p_command(&mut base_command);
        base_command.join(" ")
    }

    // generate command for snark coordinator node
    pub fn generate_snark_coordinator_command(&self) -> String {
        assert_eq!(self.service_type, ServiceType::SnarkCoordinator);

        let mut base_command = self.generate_base_command();

        base_command.push("-work-selection".to_string());
        base_command.push("seq".to_string());

        self.add_peers_command(&mut base_command);

        if let Some(snark_worker_fees) = &self.snark_coordinator_fees {
            base_command.push("-snark-worker-fee".to_string());
            base_command.push(snark_worker_fees.clone());
        } else {
            warn!(
                "No snark worker fees provided for snark coordinator node '{}'. This is not recommended.",
                self.service_name
            );
        }

        if let Some(public_key) = &self.public_key {
            base_command.push("-run-snark-coordinator".to_string());
            base_command.push(public_key.clone());
        } else {
            warn!(
                "No public key provided for snark coordinator node '{}'. This is not recommended.",
                self.service_name
            );
        }

        self.add_libp2p_command(&mut base_command);
        base_command.join(" ")
    }

    // generate command for snark worker node
    pub fn generate_snark_worker_command(&self) -> String {
        assert_eq!(self.service_type, ServiceType::SnarkWorker);
        let mut base_command = vec![
            "internal".to_string(),
            "snark-worker".to_string(),
            "-shutdown-on-disconnect".to_string(),
            "false".to_string(),
            "-config-directory".to_string(),
            format!("/config-directory/{}", self.service_name),
        ];

        if let Some(snark_coordinator_port) = &self.snark_coordinator_port {
            base_command.push("-daemon-address".to_string());
            base_command.push(format!("localhost:{}", snark_coordinator_port));
        } else {
            warn!(
                "No snark coordinator port provided for snark worker node '{}'. This is not recommended.",
                self.service_name
            );
        }

        if let Some(proof_level) = &self.snark_worker_proof_level {
            base_command.push("-proof-level".to_string());
            base_command.push(proof_level.clone());
        } else {
            warn!(
                "No proof level provided for snark worker node '{}'. This is not recommended.",
                self.service_name
            );
        }

        base_command.join(" ")
    }

    fn add_peers_command(&self, base_command: &mut Vec<String>) {
        if let Some(ref peers_list_path) = self.peers_list_path {
            base_command.push("-peer-list-file".to_string());
            base_command.push(peers_list_path.to_str().unwrap().to_string());
        } else if let Some(ref peers) = self.peers {
            for peer in peers.iter() {
                base_command.push("-peer".to_string());
                base_command.push(peer.clone());
            }
        } else {
            warn!(
                "No peers provided for block producer node '{}'. This is not recommended.",
                self.service_name
            );
        }
    }

    fn add_libp2p_command(&self, base_command: &mut Vec<String>) {
        if let Some(libp2p_keypair_path) = &self.libp2p_keypair_path {
            base_command.push("-libp2p-keypair".to_string());
            base_command.push(libp2p_keypair_path.to_str().unwrap().to_string());
        } else if let Some(libp2p_keypair) = &self.libp2p_keypair {
            base_command.push("-libp2p-keypair".to_string());
            base_command.push(libp2p_keypair.clone());
        } else {
            warn!(
                "No libp2p keypair provided for node '{}'. This is not recommended.",
                self.service_name
            );
        }
    }
}
