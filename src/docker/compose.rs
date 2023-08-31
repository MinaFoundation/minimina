//! # Docker Compose Module
//!
//! This module facilitates the generation contents of `docker-compose.yaml` for
//! deploying various Mina services in a Docker environment.

use log::{debug, error};
use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use serde_yaml;
use std::collections::HashMap;
use std::path::Path;

use crate::service::{ServiceConfig, ServiceType};
use crate::utils::get_current_user_uid_gid;

#[derive(Serialize)]
pub(crate) struct DockerCompose {
    version: String,
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
    network_mode: String,
    user: String,
    entrypoint: Vec<String>,
    volumes: Vec<String>,
    environment: Environment,
}

#[derive(Serialize)]
struct Environment {
    mina_privkey_pass: String,
    mina_libp2p_pass: String,
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

        let uid_gid = match get_current_user_uid_gid() {
            Some(uid_gid) => uid_gid,
            None => {
                error!("Unable to retrieve UID and GID of current user");
                String::new()
            }
        };
        let compose = DockerCompose {
            version: "3.8".to_string(),
            x_defaults: Defaults {
                network_mode: "host".to_string(),
                user: uid_gid,
                entrypoint: vec!["mina".to_string()],
                volumes: vec![format!("{}:/local-network", networt_path_string)],
                environment: Environment {
                    mina_privkey_pass: "naughty blue worm".to_string(),
                    mina_libp2p_pass: "naughty blue worm".to_string(),
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
        ];
        let network_path = Path::new("/tmp");
        let docker_compose = DockerCompose::generate(configs, network_path);
        println!("{:?}", docker_compose);
        assert!(docker_compose.contains("seed"));
        assert!(docker_compose.contains("block-producer"));
        assert!(docker_compose.contains("snark-coordinator"));
        assert!(docker_compose.contains("snark-worker"));
    }
}
