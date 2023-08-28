use log::{debug, warn};
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ServiceType {
    Seed,
    BlockProducer,
    SnarkWorker,
    SnarkCoordinator,
    ArchiveNode,
}

#[derive(Debug)]
pub struct ServiceConfig {
    pub service_type: ServiceType,
    pub service_name: String,
    pub docker_image: String,
    pub client_port: Option<u16>,
    pub public_key: Option<String>,
    pub public_key_path: Option<String>,
    pub libp2p_keypair: Option<String>,
    pub peers: Option<Vec<String>>,

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
    fn generate_base_command(&self) -> Vec<String> {
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
            format!("/local-network/nodes/{}", self.service_name),
        ]
    }

    // generate command for seed node
    fn generate_seed_command(&self) -> String {
        assert_eq!(self.service_type, ServiceType::Seed);

        let mut base_command = self.generate_base_command();
        base_command.push("-seed".to_string());

        if let Some(libp2p_keypair) = &self.libp2p_keypair {
            base_command.push("-libp2p-keypair".to_string());
            base_command.push(libp2p_keypair.clone());
        } else {
            warn!(
                "No libp2p keypair provided for seed node '{}'. This is not recommended.",
                self.service_name
            );
        }

        base_command.join(" ")
    }

    // generate command for block producer node
    fn generate_block_producer_command(&self) -> String {
        assert_eq!(self.service_type, ServiceType::BlockProducer);

        let mut base_command = self.generate_base_command();

        // Handling multiple peers
        if let Some(ref peers) = self.peers {
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

        if let Some(public_key_path) = &self.public_key_path {
            base_command.push("-block-producer-key".to_string());
            base_command.push(public_key_path.clone());
        } else {
            warn!(
                "No public key path provided for block producer node '{}'. This is not recommended.",
                self.service_name
            );
        }

        if let Some(libp2p_keypair) = &self.libp2p_keypair {
            base_command.push("-libp2p-keypair".to_string());
            base_command.push(libp2p_keypair.clone());
        } else {
            warn!(
                "No libp2p keypair provided for block producer node '{}'. This is not recommended.",
                self.service_name
            );
        }

        base_command.join(" ")
    }

    // generate command for snark coordinator node
    fn generate_snark_coordinator_command(&self) -> String {
        assert_eq!(self.service_type, ServiceType::SnarkCoordinator);

        let mut base_command = self.generate_base_command();

        base_command.push("-work-selection".to_string());
        base_command.push("seq".to_string());

        if let Some(peers) = &self.peers {
            for peer in peers.iter() {
                base_command.push("-peer".to_string());
                base_command.push(peer.clone());
            }
        } else {
            warn!(
                "No peers provided for snark coordinator node '{}'. This is not recommended.",
                self.service_name
            );
        }

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

        if let Some(libp2p_keypair) = &self.libp2p_keypair {
            base_command.push("-libp2p-keypair".to_string());
            base_command.push(libp2p_keypair.clone());
        } else {
            warn!(
                "No libp2p keypair provided for snark coordinator node '{}'. This is not recommended.",
                self.service_name
            );
        }

        base_command.join(" ")
    }

    fn generate_snark_worker_command(&self) -> String {
        // implement this command:
        //
        // internal snark-worker
        // -proof-level none
        // -shutdown-on-disconnect false
        // -daemon-address localhost:7000
        // -config-directory /local-network/nodes/snark_workers/mina-snark-worker-1

        assert_eq!(self.service_type, ServiceType::SnarkWorker);
        let mut base_command = vec![
            "internal".to_string(),
            "snark-worker".to_string(),
            "-shutdown-on-disconnect".to_string(),
            "false".to_string(),
            "-config-directory".to_string(),
            format!("/local-network/nodes/{}", self.service_name),
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
}

#[derive(Serialize)]
pub(crate) struct DockerCompose {
    version: &'static str,
    #[serde(
        rename = "x-defaults",
        serialize_with = "serialize_defaults_with_anchor"
    )]
    x_defaults: Defaults,
    services: HashMap<String, Service>,
}

fn serialize_defaults_with_anchor<S>(defaults: &Defaults, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut state = s.serialize_struct("Defaults", 1)?;
    state.serialize_field("&default-attributes", defaults)?;
    state.end()
}

#[derive(Serialize)]
struct Defaults {
    network_mode: &'static str,
    entrypoint: Vec<&'static str>,
    volumes: Vec<String>,
    environment: Environment,
}

#[derive(Serialize)]
struct Environment {
    mina_privkey_pass: &'static str,
    mina_libp2p_pass: &'static str,
}

#[derive(Serialize)]
struct Service {
    #[serde(rename = "<<", serialize_with = "no_quotes_merge")]
    merge: &'static str,
    container_name: String,
    image: String,
    command: String,
}

fn no_quotes_merge<S>(merge: &&'static str, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(merge)
}

impl DockerCompose {
    pub fn generate(configs: Vec<ServiceConfig>, network_path: &Path) -> String {
        let networt_path_string = network_path
            .to_str()
            .expect("Failed to convert network path to str");
        let services: HashMap<String, Service> = configs
            .iter()
            .map(|config| {
                let service = Service {
                    merge: "*default-attributes",
                    container_name: config.service_name.clone(),
                    image: config.docker_image.to_string(),
                    command: match config.service_type {
                        ServiceType::Seed => config.generate_seed_command(),
                        ServiceType::BlockProducer => config.generate_block_producer_command(),
                        ServiceType::SnarkCoordinator => {
                            config.generate_snark_coordinator_command()
                        }
                        ServiceType::SnarkWorker => config.generate_snark_worker_command(),
                        _ => String::new(),
                    },
                };
                (config.service_name.clone(), service)
            })
            .collect();

        let compose = DockerCompose {
            version: "3.8",
            x_defaults: Defaults {
                network_mode: "host",
                entrypoint: vec!["mina"],
                volumes: vec![format!("{}:/local-network", networt_path_string)],
                environment: Environment {
                    mina_privkey_pass: "naughty blue worm",
                    mina_libp2p_pass: "naughty blue worm",
                },
            },
            services,
        };

        let yaml_output = serde_yaml::to_string(&compose).unwrap();
        let generated_file = Self::post_process_yaml(yaml_output);
        debug!("Generated docker-compose.yaml: {}", generated_file);
        generated_file
    }

    // fix the format of the yaml output
    fn post_process_yaml(yaml: String) -> String {
        yaml.replace(
            "x-defaults:\n  '&default-attributes':",
            "x-defaults: &default-attributes",
        )
        .replace("<<: '*default-attributes'", "<<: *default-attributes")
        .replace("mina_privkey_pass", "MINA_PRIVKEY_PASS")
        .replace("mina_libp2p_pass", "MINA_LIBP2P_PASS")
    }
}
