mod cli;
mod default_ledger_generator;
mod directory_manager;
mod docker;
mod keys;
mod output;
mod service;
mod topology;
mod utils;

use std::collections::HashMap;

use crate::{
    default_ledger_generator::DefaultLedgerGenerator,
    keys::{KeysManager, NodeKey},
    output::{
        network::{self},
        node,
    },
    service::{ServiceConfig, ServiceType},
};
use clap::Parser;
use cli::{Cli, Command, NetworkCommand, NodeCommand};
use directory_manager::DirectoryManager;
use docker::manager::{ContainerState, DockerManager};
use env_logger::{Builder, Env};
use log::{error, info, warn};

fn network_not_exists(network_id: &str) -> bool {
    let directory_manager = DirectoryManager::new();
    if directory_manager.network_path_exists(network_id) {
        false
    } else {
        let error_message = format!("Network with network_id '{}' does not exist.", network_id);
        let network_path = directory_manager.network_path(network_id);
        let error = format!(
            "Network directory '{}' does not exist.",
            network_path.display()
        );
        print_error(&error_message, &error);
        true
    }
}

fn print_error(error_message: &str, error: &str) {
    error!("{}: {}", error_message, error);
    println!(
        "{}",
        output::Error {
            error_message: error_message.to_string(),
            error: error.trim().to_string()
        }
    );
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
                        if cmd.topology.is_none() {
                            error!(
                                "Must provide a topology file with a genesis ledger, \
                                 keys will be incompatible otherwise."
                            );

                            directory_manager
                                .delete_network_directory(cmd.network_id())
                                .unwrap();
                            std::process::exit(1);
                        }

                        info!(
                            "Copying genesis ledger from '{}' to network directory.",
                            genesis_ledger.display()
                        );

                        let genesis_path = &network_path.join("genesis_ledger.json");
                        std::fs::copy(genesis_ledger, genesis_path).unwrap();
                    }
                    None => generate_default_genesis_ledger(
                        &mut bp_keys_opt,
                        &mut libp2p_keys_opt,
                        &network_path,
                        docker_image,
                    ),
                }

                // generate docker-compose.yaml based on topology
                let services = match &cmd.topology {
                    Some(topology_path) => {
                        if cmd.genesis_ledger.is_none() {
                            error!(
                                "Must provide a genesis ledger with a topology file, \
                                 keys will be incompatible otherwise."
                            );

                            directory_manager
                                .delete_network_directory(cmd.network_id())
                                .unwrap();
                            std::process::exit(1);
                        }
                        info!(
                            "Generating docker-compose based on provided topology '{}'.",
                            topology_path.display()
                        );

                        // peers list is based on the network seeds for now
                        match topology::Topology::new(topology_path) {
                            Ok(topology) => {
                                let peers = topology.seeds();
                                let peer_list_file = directory_manager
                                    .create_peer_list_file(cmd.network_id(), &peers, 3102)
                                    .unwrap();
                                topology.services(&peer_list_file)
                            }
                            Err(err) => {
                                error!(
                                    "Error occured while parsing the topology file:\n\
                                     path: {}\n\
                                     error: {err}",
                                    topology_path.display()
                                );
                                std::process::exit(1)
                            }
                        }
                    }
                    None => {
                        info!("Topology not provided. Generating docker-compose based on default topology.");

                        if let (Some(bp_keys), Some(libp2p_keys)) =
                            (&bp_keys_opt.as_ref(), &libp2p_keys_opt.as_ref())
                        {
                            generate_default_topology(
                                bp_keys,
                                libp2p_keys,
                                &cmd,
                                &docker,
                                docker_image,
                            )
                        } else {
                            error!("Failed to generate docker-compose.yaml. Keys not generated.");
                            return;
                        }
                    }
                };

                // copy libp2p + network keys
                if let Err(e) = directory_manager.copy_all_network_keys(cmd.network_id(), &services)
                {
                    error!("Failed to copy keys with error: {e}");
                    std::process::exit(1);
                }

                // generate docker compose
                if let Err(e) = docker.compose_generate_file(&services) {
                    error!("Failed to generate docker-compose.yaml with error: {e}");
                    std::process::exit(1);
                }

                //create network
                match docker.compose_create() {
                    Ok(_) => {
                        info!(
                            "Successfully created network with id: '{}'!",
                            cmd.network_id()
                        );
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
                            "Failed to register network with 'docker compose create' with network_id '{}'",
                            cmd.network_id()
                        );
                        print_error(&error_message, e.to_string().as_str());
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
                        print_error(&error_message, e.to_string().as_str());
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
                            "Failed to get status from docker compose ls for network with network_id '{}'",
                            cmd.network_id
                        );
                        print_error(&error_message, e.to_string().as_str());

                        return;
                    }
                };

                let ps_out = match docker.compose_ps(None) {
                    Ok(out) => out,
                    Err(e) => {
                        let error_message = format!(
                            "Failed to get status from docker compose ps for network with network_id '{}'",
                            cmd.network_id
                        );
                        print_error(&error_message, e.to_string().as_str());

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
                                "Failed to delete network directory for network_id '{}'",
                                cmd.network_id
                            );
                            print_error(&error_message, e.to_string().as_str());
                        }
                    },
                    Err(e) => {
                        let error_message = format!(
                            "Failed to delete network with network_id '{}'",
                            cmd.network_id
                        );
                        print_error(&error_message, e.to_string().as_str());
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
                if network_not_exists(&cmd.network_id) {
                    return;
                };
                let network_path = directory_manager.network_path(&cmd.network_id);
                let docker = DockerManager::new(&network_path);
                match docker.compose_start_all() {
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
                            "Failed to start network with network_id '{}'",
                            cmd.network_id
                        );
                        print_error(&error_message, e.to_string().as_str());
                    }
                }
            }

            NetworkCommand::Stop(cmd) => {
                if network_not_exists(&cmd.network_id) {
                    return;
                };
                let network_path = directory_manager.network_path(&cmd.network_id);
                let docker = DockerManager::new(&network_path);
                match docker.compose_stop_all() {
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
                            "Failed to stop network with network_id '{}'",
                            cmd.network_id
                        );
                        print_error(&error_message, e.to_string().as_str());
                    }
                }
            }
        },
        Command::Node(node_cmd) => match node_cmd {
            NodeCommand::Start(cmd) => {
                let network_path = directory_manager.network_path(cmd.network_id());
                let docker = DockerManager::new(&network_path);

                let nodes = docker.compose_ps(None).unwrap();
                let node = docker.filter_container_by_name(nodes, cmd.node_id());
                let mut fresh_state = false;
                match node {
                    Some(node) => match node.state {
                        ContainerState::Running => {
                            warn!("Node with node_id '{}' is already running.", cmd.node_id());
                        }
                        ContainerState::Created => {
                            fresh_state = true;
                        }
                        _ => {}
                    },
                    None => {
                        let error_message: String =
                            format!("Failed to start node with node_id '{}'", cmd.node_id());
                        let error = format!(
                            "Node with node_id '{}' does not exist in the network '{}'",
                            cmd.node_id(),
                            cmd.network_id()
                        );
                        print_error(&error_message, error.to_string().as_str());
                        return;
                    }
                }

                fn handle_start_error(node_id: &str, error: impl ToString) {
                    let error_message: String =
                        format!("Failed to start node with node_id '{}'", node_id);
                    print_error(&error_message, error.to_string().as_str());
                }
                match docker.compose_start(vec![cmd.node_id()]) {
                    Ok(out) => {
                        if out.status.success() {
                            println!(
                                "{}",
                                node::Start {
                                    fresh_state,
                                    node_id: cmd.node_id().to_string(),
                                    network_id: cmd.network_id().to_string()
                                }
                            )
                        } else {
                            handle_start_error(cmd.node_id(), String::from_utf8_lossy(&out.stderr));
                        }
                    }
                    Err(e) => handle_start_error(cmd.node_id(), e),
                }
            }
            NodeCommand::Stop(cmd) => {
                let network_path = directory_manager.network_path(cmd.network_id());
                let docker = DockerManager::new(&network_path);
                fn handle_stop_error(node_id: &str, error: impl ToString) {
                    let error_message = format!("Failed to stop node with node_id '{}'", node_id,);
                    print_error(&error_message, error.to_string().as_str());
                }
                match docker.compose_stop(vec![cmd.node_id()]) {
                    Ok(out) => {
                        if out.status.success() {
                            println!(
                                "{}",
                                node::Stop {
                                    node_id: cmd.node_id().to_string(),
                                    network_id: cmd.network_id().to_string()
                                }
                            )
                        } else {
                            handle_stop_error(cmd.node_id(), String::from_utf8_lossy(&out.stderr));
                        }
                    }
                    Err(e) => handle_stop_error(cmd.node_id(), e),
                }
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

fn generate_default_genesis_ledger(
    bp_keys_opt: &mut Option<HashMap<String, NodeKey>>,
    libp2p_keys_opt: &mut Option<HashMap<String, NodeKey>>,
    network_path: &std::path::Path,
    docker_image: &str,
) {
    info!("Genesis ledger not provided. Generating default genesis ledger.");

    // set default services to generate keys for
    let seeds = vec!["mina-seed-1"];
    let block_producers = vec!["mina-bp-1", "mina-bp-2"];
    let snark_coordinators = vec!["mina-snark-coordinator"];
    let snark_workers = vec!["mina-snark-worker-1"];
    let all_services = [seeds, block_producers, snark_coordinators, snark_workers].concat();

    //generate key-pairs for default services
    let keys_manager = KeysManager::new(network_path, docker_image);
    *bp_keys_opt = Some(
        keys_manager
            .generate_bp_key_pairs(&all_services)
            .expect("Failed to generate key pairs for mina services."),
    );
    *libp2p_keys_opt = Some(
        keys_manager
            .generate_libp2p_key_pairs(&all_services)
            .expect("Failed to generate libp2p key pairs for mina services."),
    );

    //generate default genesis ledger
    match DefaultLedgerGenerator::generate(network_path, bp_keys_opt.as_ref().unwrap()) {
        Ok(()) => {}
        Err(e) => error!("Error generating default ledger: {}", e),
    }
}

fn generate_default_topology(
    bp_keys: &HashMap<String, NodeKey>,
    libp2p_keys: &HashMap<String, NodeKey>,
    cmd: &cli::CreateNetworkArgs,
    docker: &DockerManager,
    docker_image: &str,
) -> Vec<service::ServiceConfig> {
    let seed_name = "mina-seed-1";
    let peers =
        ServiceConfig::generate_peers([libp2p_keys[seed_name].key_string.clone()].to_vec(), 3102);
    let seed = ServiceConfig {
        service_type: ServiceType::Seed,
        service_name: format!["{}-{}", &cmd.network_id(), seed_name],
        docker_image: Some(docker_image.into()),
        git_build: None,
        client_port: Some(3100),
        public_key: None,
        public_key_path: None,
        private_key: None,
        private_key_path: None,
        libp2p_keypair: Some(libp2p_keys[seed_name].key_string.clone()),
        libp2p_keypair_path: None,
        peers: None,
        peers_list_path: None,
        snark_coordinator_fees: None,
        snark_coordinator_port: None,
        snark_worker_proof_level: None,
    };

    let bp_1_name = "mina-bp-1";
    let bp_1 = ServiceConfig {
        service_type: ServiceType::BlockProducer,
        service_name: format!["{}-{}", &cmd.network_id(), bp_1_name],
        docker_image: Some(docker_image.into()),
        git_build: None,
        client_port: Some(4000),
        public_key: Some(bp_keys[bp_1_name].key_string.clone()),
        public_key_path: Some(bp_keys[bp_1_name].key_path_docker.clone()),
        private_key: None,
        private_key_path: None,
        libp2p_keypair: Some(libp2p_keys[bp_1_name].key_string.clone()),
        libp2p_keypair_path: None,
        peers: Some(peers.clone()),
        peers_list_path: None,
        snark_coordinator_fees: None,
        snark_coordinator_port: None,
        snark_worker_proof_level: None,
    };

    let bp_2_name = "mina-bp-2";
    let bp_2 = ServiceConfig {
        service_type: ServiceType::BlockProducer,
        service_name: format!["{}-{}", &cmd.network_id(), bp_2_name],
        docker_image: Some(docker_image.into()),
        git_build: None,
        client_port: Some(4005),
        public_key: Some(bp_keys[bp_2_name].key_string.clone()),
        public_key_path: Some(bp_keys[bp_2_name].key_path_docker.clone()),
        private_key: None,
        private_key_path: None,
        libp2p_keypair: Some(libp2p_keys[bp_2_name].key_string.clone()),
        libp2p_keypair_path: None,
        peers: Some(peers.clone()),
        peers_list_path: None,
        snark_coordinator_fees: None,
        snark_coordinator_port: None,
        snark_worker_proof_level: None,
    };

    let snark_coordinator_name = "mina-snark-coordinator";
    let snark_coordinator = ServiceConfig {
        service_type: ServiceType::SnarkCoordinator,
        service_name: format!["{}-{}", &cmd.network_id(), snark_coordinator_name],
        docker_image: Some(docker_image.into()),
        git_build: None,
        client_port: Some(7000),
        public_key: Some(bp_keys[snark_coordinator_name].key_string.clone()),
        public_key_path: None,
        private_key: None,
        private_key_path: None,
        libp2p_keypair: Some(libp2p_keys[snark_coordinator_name].key_string.clone()),
        libp2p_keypair_path: None,
        peers: Some(peers),
        peers_list_path: None,
        snark_coordinator_fees: Some("0.001".into()),
        snark_coordinator_port: None,
        snark_worker_proof_level: None,
    };

    let snark_worker_1_name = "mina-snark-worker-1";
    let snark_worker_1 = ServiceConfig {
        service_type: ServiceType::SnarkWorker,
        service_name: format!["{}-{}", &cmd.network_id(), snark_worker_1_name],
        docker_image: Some(docker_image.into()),
        git_build: None,
        client_port: None,
        public_key: None,
        public_key_path: None,
        private_key: None,
        private_key_path: None,
        libp2p_keypair: None,
        libp2p_keypair_path: None,
        peers: None,
        peers_list_path: None,
        snark_coordinator_fees: None,
        snark_coordinator_port: Some(7000),
        snark_worker_proof_level: Some("none".into()),
    };

    let services = vec![seed, bp_1, bp_2, snark_coordinator, snark_worker_1];
    match docker.compose_generate_file(&services) {
        Ok(()) => info!("Successfully generated docker-compose.yaml!"),
        Err(e) => error!("Error generating docker-compose.yaml: {}", e),
    }

    services
}
