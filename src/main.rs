mod cli;
mod default_ledger_generator;
mod directory_manager;
mod docker;
mod keys;
mod output;
mod service;
mod utils;

use std::collections::HashMap;

use crate::{
    default_ledger_generator::DefaultLedgerGenerator,
    keys::{KeysManager, NodeKey},
    output::network::{self},
    service::{ServiceConfig, ServiceType},
};
use clap::Parser;
use cli::{Cli, Command, NetworkCommand, NodeCommand};
use directory_manager::DirectoryManager;
use docker::manager::DockerManager;
use env_logger::{Builder, Env};
use log::{error, info, warn};

fn network_not_exists(network_id: &str) -> bool {
    let directory_manager = DirectoryManager::new();
    if directory_manager.network_path_exists(network_id) {
        false
    } else {
        let error_message = format!("Network with network_id '{}' does not exist.", network_id);
        error!("{}", error_message);
        println!("{}", output::Error { error_message });
        true
    }
}

fn main() {
    Builder::from_env(Env::default().default_filter_or("warn")).init();
    let cli: Cli = Cli::parse();
    let directory_manager = DirectoryManager::new();

    match cli.command {
        Command::Network(net_cmd) => match net_cmd {
            NetworkCommand::Create(cmd) => {
                if directory_manager.network_path_exists(cmd.network_id()) {
                    warn!(
                        "Network with network-id '{}' already exists. Overwiting!",
                        cmd.network_id()
                    );
                }
                info!("Creating network with network-id '{}'.", cmd.network_id());
                // create directory structure for network
                let network_path = match directory_manager.generate_dir_structure(cmd.network_id())
                {
                    Ok(np) => np,
                    Err(e) => {
                        error!(
                            "Failed to set up network directory structure for network_id '{}' with error = {}",
                            cmd.network_id(), e
                        );
                        return;
                    }
                };

                // hardcode docker image for now
                let docker_image =
                    "gcr.io/o1labs-192920/mina-daemon:2.0.0rampup3-bfd1009-buster-berkeley";

                // create docker manager
                let docker = DockerManager::new(&network_path);

                // key-pairs for block producers and libp2p keys for all services
                let mut bp_keys_opt: Option<HashMap<String, NodeKey>> = None;
                let mut libp2p_keys_opt: Option<HashMap<String, NodeKey>> = None;

                // generate genesis ledger
                match &cmd.genesis_ledger {
                    Some(genesis_ledger) => {
                        info!(
                            "Copying genesis ledger from '{}' to network directory.",
                            genesis_ledger.display()
                        );
                    }
                    None => {
                        info!("Genesis ledger not provided. Generating default genesis ledger.");
                        // set default services to generate keys for
                        let seeds = vec!["mina-seed-1"];
                        let block_producers = vec!["mina-bp-1", "mina-bp-2"];
                        let snark_coordinators = vec!["mina-snark-coordinator"];
                        let snark_workers = vec!["mina-snark-worker-1"];
                        let all_services =
                            [seeds, block_producers, snark_coordinators, snark_workers].concat();

                        //generate key-pairs for default services
                        let keys_manager = KeysManager::new(&network_path, docker_image);
                        bp_keys_opt = Some(
                            keys_manager
                                .generate_bp_key_pairs(&all_services)
                                .expect("Failed to generate key pairs for mina services."),
                        );
                        libp2p_keys_opt = Some(
                            keys_manager
                                .generate_libp2p_key_pairs(&all_services)
                                .expect("Failed to generate libp2p key pairs for mina services."),
                        );

                        //generate default genesis ledger
                        match DefaultLedgerGenerator::generate(
                            &network_path,
                            bp_keys_opt.as_ref().unwrap(),
                        ) {
                            Ok(()) => {}
                            Err(e) => error!("Error generating default ledger: {}", e),
                        }
                    }
                }

                // generate docker-compose.yaml based on topology
                let services = match &cmd.topology {
                    Some(topology) => {
                        info!(
                            "Generating docker-compose based on provided topology '{}'.",
                            topology.display()
                        );
                        vec![]
                    }
                    None => {
                        info!("Topology not provided. Generating docker-compose based on default topology.");
                        if let (Some(bp_keys), Some(libp2p_keys)) =
                            (&bp_keys_opt.as_ref(), &libp2p_keys_opt.as_ref())
                        {
                            let seed_name = "mina-seed-1";
                            let seed = ServiceConfig {
                                service_type: ServiceType::Seed,
                                service_name: format!["{}-{}", &cmd.network_id(), seed_name],
                                docker_image: docker_image.into(),
                                client_port: Some(3100),
                                public_key: None,
                                public_key_path: None,
                                libp2p_keypair: Some(libp2p_keys[seed_name].key_string.clone()),
                                peers: None,
                                snark_coordinator_fees: None,
                                snark_coordinator_port: None,
                                snark_worker_proof_level: None,
                            };
                            let peers = ServiceConfig::generate_peers(
                                [libp2p_keys[seed_name].key_string.clone()].to_vec(),
                                3102,
                            );

                            let bp_1_name = "mina-bp-1";
                            let bp_1 = ServiceConfig {
                                service_type: ServiceType::BlockProducer,
                                service_name: format!["{}-{}", &cmd.network_id(), bp_1_name],
                                docker_image: docker_image.into(),
                                client_port: Some(4000),
                                public_key: Some(bp_keys[bp_1_name].key_string.clone()),
                                public_key_path: Some(bp_keys[bp_1_name].key_path_docker.clone()),
                                libp2p_keypair: Some(libp2p_keys[bp_1_name].key_string.clone()),
                                peers: Some(peers.clone()),
                                snark_coordinator_fees: None,
                                snark_coordinator_port: None,
                                snark_worker_proof_level: None,
                            };

                            let bp_2_name = "mina-bp-2";
                            let bp_2 = ServiceConfig {
                                service_type: ServiceType::BlockProducer,
                                service_name: format!["{}-{}", &cmd.network_id(), bp_2_name],
                                docker_image: docker_image.into(),
                                client_port: Some(4005),
                                public_key: Some(bp_keys[bp_2_name].key_string.clone()),
                                public_key_path: Some(bp_keys[bp_2_name].key_path_docker.clone()),
                                libp2p_keypair: Some(libp2p_keys[bp_2_name].key_string.clone()),
                                peers: Some(peers.clone()),
                                snark_coordinator_fees: None,
                                snark_coordinator_port: None,
                                snark_worker_proof_level: None,
                            };

                            let snark_coordinator_name = "mina-snark-coordinator";
                            let snark_coordinator = ServiceConfig {
                                service_type: ServiceType::SnarkCoordinator,
                                service_name: format![
                                    "{}-{}",
                                    &cmd.network_id(),
                                    snark_coordinator_name
                                ],
                                docker_image: docker_image.into(),
                                client_port: Some(7000),
                                public_key: Some(
                                    bp_keys[snark_coordinator_name].key_string.clone(),
                                ),
                                public_key_path: None,
                                libp2p_keypair: Some(
                                    libp2p_keys[snark_coordinator_name].key_string.clone(),
                                ),
                                peers: Some(peers),
                                snark_coordinator_fees: Some("0.001".into()),
                                snark_coordinator_port: None,
                                snark_worker_proof_level: None,
                            };
                            let snark_worker_1_name = "mina-snark-worker-1";
                            let snark_worker_1 = ServiceConfig {
                                service_type: ServiceType::SnarkWorker,
                                service_name: format![
                                    "{}-{}",
                                    &cmd.network_id(),
                                    snark_worker_1_name
                                ],
                                docker_image: docker_image.into(),
                                client_port: None,
                                public_key: None,
                                public_key_path: None,
                                libp2p_keypair: None,
                                peers: None,
                                snark_coordinator_fees: None,
                                snark_coordinator_port: Some(7000),
                                snark_worker_proof_level: Some("none".into()),
                            };

                            let services =
                                vec![seed, bp_1, bp_2, snark_coordinator, snark_worker_1];

                            match docker.compose_generate_file(services.clone()) {
                                Ok(()) => info!("Successfully generated docker-compose.yaml!"),
                                Err(e) => error!("Error generating docker-compose.yaml: {}", e),
                            }
                            services
                        } else {
                            error!("Failed to generate docker-compose.yaml. Keys not generated.");
                            return;
                        }
                    }
                };
                //create network
                match docker.compose_create() {
                    Ok(_) => {
                        info!("Successfully created network!");
                        // generate command output
                        let result = output::generate_network_info(services, cmd.network_id());
                        println!("{}", result);
                        let json_data = format!("{}", result);
                        let json_path = network_path.join("network.json");
                        match std::fs::write(json_path, json_data) {
                            Ok(()) => {}
                            Err(e) => error!("Error generating network.json: {}", e),
                        }
                    }
                    Err(e) => {
                        let error_message = format!(
                            "Failed to register network with 'docker compose create' with network_id '{}' with error = {}",
                            cmd.network_id(),
                            e
                        );
                        error!("{}", error_message);
                        println!("{}", output::Error { error_message })
                    }
                }
            }

            NetworkCommand::Info(cmd) => {
                if network_not_exists(&cmd.network_id) {
                    return;
                };
                let network_path = directory_manager.network_path(&cmd.network_id);
                let json_path = network_path.join("network.json");
                match std::fs::read_to_string(json_path) {
                    Ok(json_data) => {
                        println!("{}", json_data);
                    }
                    Err(e) => {
                        let error_message = format!(
                            "Failed to get info for network with network_id '{}' with error = {}",
                            cmd.network_id, e
                        );
                        error!("{}", error_message);
                        println!("{}", output::Error { error_message })
                    }
                }
            }

            NetworkCommand::Status(cmd) => {
                if network_not_exists(&cmd.network_id) {
                    return;
                };
                let network_path = directory_manager.network_path(&cmd.network_id);
                let docker = DockerManager::new(&network_path);
                let ls_out = match docker.compose_ls() {
                    Ok(out) => out,
                    Err(e) => {
                        let error_message = format!(
                            "Failed to get status from docker compose ls for network with network_id '{}' with error = {}",
                            cmd.network_id, e
                        );
                        error!("{}", error_message);
                        println!("{}", output::Error { error_message });
                        return;
                    }
                };

                let ps_out = match docker.compose_ps() {
                    Ok(out) => out,
                    Err(e) => {
                        let error_message = format!(
                            "Failed to get status from docker compose ps for network with network_id '{}' with error = {}",
                            cmd.network_id, e
                        );
                        error!("{}", error_message);
                        println!("{}", output::Error { error_message });
                        return;
                    }
                };

                let compose_file_path = docker.compose_path.as_path().to_str().unwrap();
                let mut status = network::Status::new(&cmd.network_id);
                status.update_from_compose_ls(ls_out, compose_file_path);
                status.update_from_compose_ps(ps_out);

                println!("{}", status);
            }

            NetworkCommand::Delete(cmd) => {
                if network_not_exists(&cmd.network_id) {
                    return;
                };
                let docker = DockerManager::new(&directory_manager.network_path(&cmd.network_id));
                match docker.compose_down() {
                    Ok(_) => match directory_manager.delete_network_directory(&cmd.network_id) {
                        Ok(_) => {
                            println!(
                                "{}",
                                network::Delete {
                                    network_id: cmd.network_id
                                }
                            )
                        }
                        Err(e) => {
                            let error_message = format!(
                                    "Failed to delete network directory for network_id '{}' with error = {}",
                                    cmd.network_id, e
                                );
                            error!("{}", error_message);
                            println!("{}", output::Error { error_message });
                        }
                    },
                    Err(e) => {
                        let error_message = format!(
                            "Failed to delete network with network_id '{}' with error = {}",
                            cmd.network_id, e
                        );
                        error!("{}", error_message);
                        println!("{}", output::Error { error_message })
                    }
                }
            }

            NetworkCommand::List => {
                let networks = directory_manager
                    .list_network_directories()
                    .expect("Failed to list networks");

                let mut list = network::List::new();
                if networks.is_empty() {
                    println!("{}", list);
                } else {
                    list.update(
                        networks,
                        directory_manager.base_path.as_path().to_str().unwrap(),
                    );
                    println!("{}", list);
                }
            }

            NetworkCommand::Start(cmd) => {
                let network_path = directory_manager.network_path(&cmd.network_id);
                let docker = DockerManager::new(&network_path);
                match docker.compose_start() {
                    Ok(_) => {
                        println!(
                            "{}",
                            network::Start {
                                network_id: cmd.network_id
                            }
                        )
                    }
                    Err(e) => {
                        let error_message = format!(
                            "Failed to start network with network_id '{}' with error = {}",
                            cmd.network_id, e
                        );
                        error!("{}", error_message);
                        println!("{}", output::Error { error_message })
                    }
                }
            }

            NetworkCommand::Stop(cmd) => {
                let network_path = directory_manager.network_path(&cmd.network_id);
                let docker = DockerManager::new(&network_path);
                match docker.compose_stop() {
                    Ok(_) => {
                        println!(
                            "{}",
                            network::Stop {
                                network_id: cmd.network_id
                            }
                        )
                    }
                    Err(e) => {
                        let error_message = format!(
                            "Failed to stop network with network_id '{}' with error = {}",
                            cmd.network_id, e
                        );
                        error!("{}", error_message);
                        println!("{}", output::Error { error_message })
                    }
                }
            }
        },
        Command::Node(node_cmd) => match node_cmd {
            NodeCommand::Start(cmd) => {
                info!(
                    "Node start command with node_id {}, network_id {}.",
                    cmd.node_id(),
                    cmd.network_id()
                );
            }
            NodeCommand::Stop(cmd) => {
                info!(
                    "Node stop command with node_id {}, network_id {}.",
                    cmd.node_id(),
                    cmd.network_id()
                );
            }
            NodeCommand::Logs(cmd) => {
                info!(
                    "Node logs command with node_id {}, network_id {}.",
                    cmd.node_id(),
                    cmd.network_id()
                );
            }
        },
    }
}
