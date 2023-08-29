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
