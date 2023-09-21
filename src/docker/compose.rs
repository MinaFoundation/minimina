//! # Docker Compose Module
//!
//! This module facilitates the generation contents of `docker-compose.yaml` for
//! deploying various Mina services in a Docker environment.

use crate::{
    service::{ServiceConfig, ServiceType},
    DEFAULT_ARCHIVE_DOCKER_IMAGE, DEFAULT_DAEMON_DOCKER_IMAGE,
};
use log::debug;
use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use serde_yaml;
use std::collections::HashMap;
use std::path::Path;

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
    environment: Environment,
}

#[derive(Serialize)]
struct Environment {
    mina_privkey_pass: String,
    mina_libp2p_pass: String,
    mina_client_trustlist: String,
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
const POSTGRES_DATA: &str = "postgres-data";

impl DockerCompose {
    pub fn generate(configs: &[ServiceConfig], network_path: &Path) -> String {
        let network_path_string = network_path
            .to_str()
            .expect("Failed to convert network path to str");
        let network_name = network_path.file_name().unwrap().to_str().unwrap();

        let archive_data = configs.iter().find_map(|config| {
            if config.service_type == ServiceType::ArchiveNode {
                let archive_host = format!("{}-{network_name}", config.service_name.clone());
                Some((archive_host, config.archive_port.unwrap()))
            } else {
                None
            }
        });

        //insert volumes for each service
        let mut volumes = configs.iter().fold(HashMap::new(), |mut acc, config| {
            let service_name = format!("{}-{network_name}", config.service_name.clone());
            acc.insert(service_name, None);
            acc
        });

        let mut services: HashMap<String, Service> = configs
            .iter()
            .filter_map(|config| {
                match config.service_type {
                    // We'll handle ArchiveNode outside of this map operation
                    // because it requires adding additional service: postgres
                    ServiceType::ArchiveNode => None,
                    _ => {
                        let service_name =
                            format!("{}-{network_name}", config.service_name.clone());
                        let service = Service {
                            merge: Some("*default-attributes"),
                            container_name: service_name.clone(),
                            entrypoint: Some(vec!["mina".to_string()]),
                            volumes: Some(vec![
                                format!("{}:/local-network", network_path_string),
                                format!("{}:/{}", service_name, CONFIG_DIRECTORY),
                            ]),
                            image: config
                                .docker_image
                                .clone()
                                .unwrap_or(DEFAULT_DAEMON_DOCKER_IMAGE.into()),
                            command: Some(match config.service_type {
                                ServiceType::Seed => config.generate_seed_command(),
                                ServiceType::BlockProducer => {
                                    config.generate_block_producer_command(archive_data.clone())
                                }
                                ServiceType::SnarkCoordinator => {
                                    config.generate_snark_coordinator_command()
                                }
                                ServiceType::SnarkWorker => {
                                    config.generate_snark_worker_command(network_name.to_string())
                                }
                                _ => String::new(),
                            }),
                            ports: match config.client_port {
                                Some(port) => {
                                    let gql_port = port + 1;
                                    let external_port = port + 2;
                                    Some(vec![
                                        format!("{}:{}", gql_port, gql_port),
                                        port.to_string(),
                                        external_port.to_string(),
                                    ])
                                }
                                None => None,
                            },
                            ..Default::default()
                        };
                        Some((
                            format!("{}-{network_name}", config.service_name.clone()),
                            service,
                        ))
                    }
                }
            })
            .collect();

        if let Some(archive_config) = configs
            .iter()
            .find(|config| config.service_type == ServiceType::ArchiveNode)
        {
            // Add postres volume
            volumes.insert(POSTGRES_DATA.to_string(), None);

            let mut postgres_environment = HashMap::new();
            postgres_environment.insert("POSTGRES_PASSWORD".to_string(), "postgres".to_string());

            let postgres_name = format!("postgres-{network_name}");
            services.insert(
                postgres_name.clone(),
                Service {
                    container_name: postgres_name.clone(),
                    image: "postgres".to_string(),
                    environment: Some(postgres_environment),
                    volumes: Some(vec![format!("{}:/var/lib/postgresql/data", POSTGRES_DATA)]),
                    ports: Some(vec!["5432".to_string()]),
                    ..Default::default()
                },
            );

            let archive_name = format!("{}-{network_name}", archive_config.service_name.clone());
            let (_, archive_port) = archive_data.unwrap();
            let archive_command = format!("mina-archive run --postgres-uri postgres://postgres:postgres@{}:5432/archive --server-port {}", postgres_name, archive_port);

            services.insert(
                archive_name.clone(),
                Service {
                    container_name: archive_name.clone(),
                    image: archive_config
                        .docker_image
                        .clone()
                        .unwrap_or(DEFAULT_ARCHIVE_DOCKER_IMAGE.into()),
                    command: Some(archive_command),
                    volumes: Some(vec![
                        format!("{}:/data", archive_name),
                        format!("{}:/local-network", network_path_string),
                    ]),
                    ports: Some(vec![archive_port.to_string()]),
                    depends_on: Some(vec![postgres_name]),
                    ..Default::default()
                },
            );
        }

        let compose = DockerCompose {
            version: "3.8".to_string(),
            x_defaults: Defaults {
                environment: Environment {
                    mina_privkey_pass: "naughty blue worm".to_string(),
                    mina_libp2p_pass: "naughty blue worm".to_string(),
                    mina_client_trustlist: "0.0.0.0/0".to_string(),
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
        .replace("mina_client_trustlist", "MINA_CLIENT_TRUSTLIST")
        .replace("null", "")
    }
}

#[cfg(test)]
mod tests {
    use tempdir::TempDir;

    use super::*;
    use crate::service::ServiceType;

    #[test]
    fn test_generate() {
        let configs = vec![
            ServiceConfig {
                service_name: "seed".to_string(),
                service_type: ServiceType::Seed,
                docker_image: Some("seed-image".into()),
                client_port: Some(8300),
                ..Default::default()
            },
            ServiceConfig {
                service_name: "block-producer".to_string(),
                service_type: ServiceType::BlockProducer,
                docker_image: Some("bp-image".into()),
                client_port: Some(8301),
                ..Default::default()
            },
            ServiceConfig {
                service_name: "snark-coordinator".to_string(),
                service_type: ServiceType::SnarkCoordinator,
                docker_image: Some("snark-image".into()),
                client_port: Some(8302),
                ..Default::default()
            },
            ServiceConfig {
                service_name: "snark-worker".to_string(),
                service_type: ServiceType::SnarkWorker,
                docker_image: Some("worker-image".into()),
                client_port: Some(8303),
                ..Default::default()
            },
            ServiceConfig {
                service_name: "mina-archive555".to_string(),
                service_type: ServiceType::ArchiveNode,
                docker_image: Some("archive-image".into()),
                archive_port: Some(8304),
                ..Default::default()
            },
        ];
        let network_path = Path::new("/not-a-real-path");
        let docker_compose = DockerCompose::generate(&configs, network_path);
        println!("{:?}", docker_compose);
        assert!(docker_compose.contains("seed"));
        assert!(docker_compose.contains("block-producer"));
        assert!(docker_compose.contains("snark-coordinator"));
        assert!(docker_compose.contains("snark-worker"));
        assert!(docker_compose.contains("mina-archive555"));
        assert!(docker_compose.contains("postgres"));
        assert!(docker_compose.contains("postgres-data"));
        assert!(docker_compose.contains("-archive-address"));
    }

    #[test]
    fn test_generate_without_archive_node() {
        let configs = vec![
            ServiceConfig {
                service_name: "seed".to_string(),
                service_type: ServiceType::Seed,
                docker_image: Some("seed-image".into()),
                client_port: Some(8300),
                ..Default::default()
            },
            ServiceConfig {
                service_name: "block-producer".to_string(),
                service_type: ServiceType::BlockProducer,
                docker_image: Some("bp-image".into()),
                client_port: Some(8301),
                ..Default::default()
            },
        ];
        let network_path = Path::new("/not-a-real-path");
        let docker_compose = DockerCompose::generate(&configs, network_path);
        println!("{}", docker_compose);
        assert!(docker_compose.contains("seed"));
        assert!(docker_compose.contains("block-producer"));
        assert!(!docker_compose.contains("mina-archive"));
        assert!(!docker_compose.contains("postgres"));
        assert!(!docker_compose.contains("postgres-data"));
        assert!(!docker_compose.contains("archive-data"));
        assert!(!docker_compose.contains("-archive-address"));
    }

    #[test]
    fn test_generate_compose_from_topology() -> std::io::Result<()> {
        use crate::{topology::Topology, DirectoryManager};
        let tempdir = TempDir::new("test_generate_compose_from_topology")
            .expect("Cannot create temporary directory");
        let tmp_network_path = tempdir.path();
        let dir_manager = DirectoryManager::_new_with_base_path(tmp_network_path.to_path_buf());
        let network_id = "test_network";
        let network_path = dir_manager.network_path(network_id);
        dir_manager.generate_dir_structure(network_id)?;

        let file = std::path::PathBuf::from("./tests/data/large_network/topology.json");
        let contents = std::fs::read_to_string(file)?;
        let topology: Topology = serde_json::from_str(&contents)?;
        let peers_file = dir_manager.peer_list_file(network_id);
        let services = topology.services(&peers_file);
        let compose_contents = DockerCompose::generate(&services, &network_path);

        assert!(compose_contents.contains("snark-node"));
        assert!(compose_contents.contains("archive-node"));
        assert!(compose_contents.contains("receiver"));
        assert!(compose_contents.contains("empty_node-1"));
        assert!(compose_contents.contains("empty_node-2"));
        assert!(compose_contents.contains("observer"));
        assert!(compose_contents.contains("seed-0"));
        assert!(compose_contents.contains("seed-1"));
        assert!(compose_contents.contains("-archive-address"));

        dir_manager.delete_network_directory(network_id)?;

        Ok(())
    }
}
