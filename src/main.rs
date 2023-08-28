mod cli;
mod cmd;
mod default_ledger_generator;
mod directory_manager;
mod docker;
mod keys;

use crate::{
    default_ledger_generator::DefaultLedgerGenerator,
    docker::compose::{ServiceConfig, ServiceType},
    keys::KeysManager,
};
use clap::Parser;
use cli::{Cli, Command, NetworkCommand, NodeCommand};
use directory_manager::DirectoryManager;
use docker::manager::DockerManager;
use env_logger::{Builder, Env};
use log::{error, info, warn};

fn main() {
    Builder::from_env(Env::default().default_filter_or("info")).init();
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

                let docker_manager = DockerManager::new(&network_path);

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
                        //generate key-pairs for default topology
                        let keys_manager = KeysManager::new(&network_path, docker_image);
                        let block_producers = vec!["mina-bp-1", "mina-bp-2"];
                        let bp_keys = keys_manager
                            .generate_bp_key_pairs(&block_producers)
                            .expect("Failed to generate key pairs for block producers.");

                        let _libp2p_keys = keys_manager
                            .generate_libp2p_key_pairs(&block_producers)
                            .expect("Failed to generate libp2p key pairs for block producers.");

                        //generate default genesis ledger
                        match DefaultLedgerGenerator::generate(&network_path, bp_keys) {
                            Ok(()) => info!("Successfully generated ledger!"),
                            Err(e) => error!("Error generating ledger: {}", e),
                        }
                    }
                }

                // generate docker-compose.yaml
                match &cmd.topology {
                    Some(topology) => {
                        info!(
                            "Generating docker-compose based on provided topology '{}'.",
                            topology.display()
                        );
                    }
                    None => {
                        info!("Topology not provided. Generating docker-compose based on default topology.");
                        let seed = ServiceConfig {
                            service_type: ServiceType::Seed,
                            service_name: "mina-seed-1".into(),
                            docker_image: docker_image.into(),
                            public_key: None,
                            public_key_path: None,
                            libp2p_keypair: Some("CAESQNf7ldToowe604aFXdZ76GqW/XVlDmnXmBT+otorvIekBmBaDWu/6ZwYkZzqfr+3IrEh6FLbHQ3VSmubV9I9Kpc=,CAESIAZgWg1rv+mcGJGc6n6/tyKxIehS2x0N1Uprm1fSPSqX,12D3KooWAFFq2yEQFFzhU5dt64AWqawRuomG9hL8rSmm5vxhAsgr".into()),
                            peers: None,
                            client_port: Some(3100),
                        };
                        let bp_1 = ServiceConfig {
                            service_type: ServiceType::BlockProducer,
                            service_name: "mina-bp-1".into(),
                            docker_image: docker_image.into(),
                            public_key: None,
                            public_key_path: None,
                            libp2p_keypair: Some("CAES".into()),
                            peers: None,
                            client_port: Some(4000),
                        };
                        let services = vec![seed, bp_1];

                        match docker_manager.compose_generate_file(services) {
                            Ok(()) => info!("Successfully generated docker-compose.yaml!"),
                            Err(e) => error!("Error generating docker-compose.yaml: {}", e),
                        }
                    }
                }
            }
            NetworkCommand::Delete(cmd) => {
                match directory_manager.delete_network_directory(&cmd.network_id) {
                    Ok(_) => {}
                    Err(e) => {
                        error!(
                            "Failed to delete network directory for network_id '{}' with error = {}",
                            cmd.network_id, e
                        );
                        return;
                    }
                }

                info!("Network '{}' deleted successfully.", cmd.network_id);
            }
            NetworkCommand::List => {
                let networks = directory_manager
                    .list_network_directories()
                    .expect("Failed to list networks");

                if networks.is_empty() {
                    println!("No networks found.");
                    return;
                }

                println!("Available networks:");

                for network in networks {
                    println!("  {}", network);
                }
            }
            NetworkCommand::Start(cmd) => {
                let network_path = directory_manager.network_path(&cmd.network_id);
                let docker_manager = DockerManager::new(&network_path);
                match docker_manager.compose_up() {
                    Ok(_) => {}
                    Err(e) => {
                        error!(
                            "Failed to start network with network_id '{}' with error = {}",
                            cmd.network_id, e
                        );
                    }
                }
            }
            NetworkCommand::Stop(cmd) => {
                let network_path = directory_manager.network_path(&cmd.network_id);
                let docker_manager = DockerManager::new(&network_path);
                match docker_manager.compose_down() {
                    Ok(_) => {}
                    Err(e) => {
                        error!(
                            "Failed to stop network with network_id '{}' with error = {}",
                            cmd.network_id, e
                        );
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
