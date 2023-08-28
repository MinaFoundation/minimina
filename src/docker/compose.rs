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
    pub public_key: Option<String>,
    pub public_key_path: Option<String>,
    pub libp2p_keypair: Option<String>,
    pub peers: Option<Vec<String>>,
    pub client_port: Option<u16>,
}

impl ServiceConfig {
    fn generate_seed_command(&self) -> String {
        assert_eq!(self.service_type, ServiceType::Seed);

        let client_port = self.client_port.unwrap_or(3100);
        let rest_port = client_port + 1;
        let external_port = rest_port + 1;
        let metrics_port = external_port + 1;
        let libp2p_metrics_port = metrics_port + 1;

        let client_port_str = client_port.to_string();
        let rest_port_str = rest_port.to_string();
        let external_port_str = external_port.to_string();
        let metrics_port_str = metrics_port.to_string();
        let libp2p_metric_port = libp2p_metrics_port.to_string();
        let base_command = vec![
            "daemon",
            "-client-port",
            &client_port_str,
            "-rest-port",
            &rest_port_str,
            "-insecure-rest-server",
            "-external-port",
            &external_port_str,
            "-metrics-port",
            &metrics_port_str,
            "-libp2p-metrics-port",
            &libp2p_metric_port,
            "-config-file",
            "/local-network/genesis_ledger.json",
            "-log-json",
            "-log-level",
            "Trace",
            "-file-log-level",
            "Trace",
            "-seed",
        ];

        let libp2p_keypair = match &self.libp2p_keypair {
            Some(libp2p_keypair) => {
                let libp2p_keypair = format!("-libp2p-keypair {}", libp2p_keypair);
                libp2p_keypair
            }
            None => {
                warn!("No libp2p keypair provided for seed node. This is not recommended.");
                "".to_string()
            }
        };

        let config_directory = format!(
            "-config-directory /local-network/nodes/{}",
            self.service_name
        );

        [base_command, vec![&libp2p_keypair, &config_directory]]
            .concat()
            .join(" ")
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
