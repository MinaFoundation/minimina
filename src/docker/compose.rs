//! # Docker Compose Module
//!
//! This module facilitates the generation contents of `docker-compose.yaml` for
//! deploying various Mina services in a Docker environment.

use log::debug;
use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use serde_yaml;
use std::collections::HashMap;
use std::path::Path;

use crate::service::{ServiceConfig, ServiceType};

#[derive(Serialize)]
pub(crate) struct DockerCompose {
    version: String,
    #[serde(
        rename = "x-defaults",
        serialize_with = "serialize_defaults_with_anchor"
    )]
    x_defaults: Defaults,
    volumes: HashMap<String, Option<String>>,
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
    volumes: Vec<String>,
    environment: Environment,
}

#[derive(Serialize)]
struct Environment {
    mina_privkey_pass: String,
    mina_libp2p_pass: String,
}

#[derive(Default, Serialize)]
struct Service {
    #[serde(rename = "<<", skip_serializing_if = "Option::is_none")]
    merge: Option<&'static str>,
    container_name: String,
    image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    network_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    entrypoint: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    volumes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    environment: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ports: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    depends_on: Option<Vec<String>>,
}

const CONFIG_DIRECTORY: &str = "config-directory";
const ARCHIVE_DATA: &str = "archive-data";
const POSTGRES_DATA: &str = "postgres-data";

impl DockerCompose {
    pub fn generate(configs: &[ServiceConfig], network_path: &Path) -> String {
        let network_path_string = network_path
            .to_str()
            .expect("Failed to convert network path to str");
        let mut volumes = HashMap::new();
        volumes.insert(CONFIG_DIRECTORY.to_string(), None);

        let mut services: HashMap<String, Service> = configs
            .iter()
            .filter_map(|config| {
                match config.service_type {
                    // We'll handle ArchiveNode outside of this map operation
                    // because it requires adding additional service: postgres
                    ServiceType::ArchiveNode => None,
                    _ => {
                        let service = Service {
                            merge: Some("*default-attributes"),
                            container_name: config.service_name.clone(),
                            network_mode: Some("host".to_string()),
                            entrypoint: Some(vec!["mina".to_string()]),
                            image: config.docker_image.to_string(),
                            command: Some(match config.service_type {
                                ServiceType::Seed => config.generate_seed_command(),
                                ServiceType::BlockProducer => {
                                    config.generate_block_producer_command()
                                }
                                ServiceType::SnarkCoordinator => {
                                    config.generate_snark_coordinator_command()
                                }
                                ServiceType::SnarkWorker => config.generate_snark_worker_command(),
                                _ => String::new(),
                            }),
                            ..Default::default()
                        };
                        Some((config.service_name.clone(), service))
                    }
                }
            })
            .collect();

        if configs
            .iter()
            .any(|config| config.service_type == ServiceType::ArchiveNode)
        {
            let archive_config = configs
                .iter()
                .find(|config| config.service_type == ServiceType::ArchiveNode)
                .expect("Failed to find archive config");

            // Add archive and postres volumes
            volumes.insert(POSTGRES_DATA.to_string(), None);
            volumes.insert(ARCHIVE_DATA.to_string(), None);

            let mut postgres_environment = HashMap::new();
            postgres_environment.insert("POSTGRES_PASSWORD".to_string(), "postgres".to_string());

            services.insert(
                "postgres".to_string(),
                Service {
                    container_name: "postgres".to_string(),
                    image: "postgres".to_string(),
                    environment: Some(postgres_environment),
                    volumes: Some(vec![format!("{}:/var/lib/postgresql/data", POSTGRES_DATA)]),
                    ports: Some(vec!["6451:5432".to_string()]),
                    ..Default::default()
                },
            );

            services.insert(
                    "mina-archive".to_string(),
                    Service {
                        container_name: archive_config.service_name.clone(),
                        image: archive_config.docker_image.to_string(),
                        command: Some(
                            "mina-archive run --postgres-uri postgres://postgres:postgres@postgres:5432/archive --server-port 3086".to_string()
                        ),
                        volumes: Some(vec![format!("{}:/data", ARCHIVE_DATA)]),
                        ports: Some(vec!["3086:3086".to_string()]),
                        depends_on: Some(vec!["postgres".to_string()]),
                        ..Default::default()
                    },
                );
        }

        let compose = DockerCompose {
            version: "3.8".to_string(),
            x_defaults: Defaults {
                volumes: vec![
                    format!("{}:/local-network", network_path_string),
                    format!("{}:/{}", CONFIG_DIRECTORY, CONFIG_DIRECTORY),
                ],
                environment: Environment {
                    mina_privkey_pass: "naughty blue worm".to_string(),
                    mina_libp2p_pass: "naughty blue worm".to_string(),
                },
            },
            volumes,
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
        .replace("null", "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::ServiceType;

    #[test]
    fn test_generate() {
        let configs = vec![
            ServiceConfig {
                service_name: "seed".to_string(),
                service_type: ServiceType::Seed,
                client_port: Some(8300),
                ..Default::default()
            },
            ServiceConfig {
                service_name: "block-producer".to_string(),
                service_type: ServiceType::BlockProducer,
                client_port: Some(8301),
                ..Default::default()
            },
            ServiceConfig {
                service_name: "snark-coordinator".to_string(),
                service_type: ServiceType::SnarkCoordinator,
                client_port: Some(8302),
                ..Default::default()
            },
            ServiceConfig {
                service_name: "snark-worker".to_string(),
                service_type: ServiceType::SnarkWorker,
                client_port: Some(8303),
                ..Default::default()
            },
            ServiceConfig {
                service_name: "mina-archive555".to_string(),
                service_type: ServiceType::ArchiveNode,
                client_port: Some(8303),
                ..Default::default()
            },
        ];
        let network_path = Path::new("/tmp");
        let docker_compose = DockerCompose::generate(&configs, network_path);
        println!("{:?}", docker_compose);
        assert!(docker_compose.contains("seed"));
        assert!(docker_compose.contains("block-producer"));
        assert!(docker_compose.contains("snark-coordinator"));
        assert!(docker_compose.contains("snark-worker"));
        assert!(docker_compose.contains("mina-archive555"));
        assert!(docker_compose.contains("postgres"));
        assert!(docker_compose.contains("postgres-data"));
        assert!(docker_compose.contains("archive-data"));
    }

    #[test]
    fn test_generate_without_archive_node() {
        let configs = vec![
            ServiceConfig {
                service_name: "seed".to_string(),
                service_type: ServiceType::Seed,
                client_port: Some(8300),
                ..Default::default()
            },
            ServiceConfig {
                service_name: "block-producer".to_string(),
                service_type: ServiceType::BlockProducer,
                client_port: Some(8301),
                ..Default::default()
            },
        ];
        let network_path = Path::new("/tmp2");
        let docker_compose = DockerCompose::generate(&configs, network_path);
        println!("{}", docker_compose);
        assert!(docker_compose.contains("seed"));
        assert!(docker_compose.contains("block-producer"));
        assert!(!docker_compose.contains("mina-archive"));
        assert!(!docker_compose.contains("postgres"));
        assert!(!docker_compose.contains("postgres-data"));
        assert!(!docker_compose.contains("archive-data"));
    }
}
