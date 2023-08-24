mod cli;
mod cmd;
mod default_ledger_generator;
mod directory_manager;
mod docker_compose_manager;
mod keys;

use crate::{default_ledger_generator::DefaultLedgerGenerator, keys::Keys};
use clap::Parser;
use cli::{Cli, Command, NetworkCommand, NodeCommand};
use directory_manager::DirectoryManager;
use docker_compose_manager::DockerComposeManager;
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
                directory_manager
                    .create_network_directory(cmd.network_id())
                    .expect("Failed to create network directory");

                directory_manager
                    .create_subdirectories(cmd.network_id())
                    .expect("Failed to create subdirectories");

                directory_manager
                    .set_subdirectories_permissions(cmd.network_id(), 0o700)
                    .expect("Failed to set permissions for subdirectories");

                let network_path = directory_manager.network_path(cmd.network_id());

                // pattern match on &cmd.topology
                match &cmd.topology {
                    Some(topology) => {
                        info!(
                            "Copying topology from '{}' to network directory.",
                            topology.display()
                        );
                    }
                    None => {
                        info!("Topology not provided. Generating default topology.");
                        //generate default docker compose file
                    }
                }

                // pattern match on &cmd.genesis_ledger
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
                        let block_producers = vec!["mina-bp-1", "mina-bp-2"];
                        let bp_keys = Keys::generate_bp_key_pairs(&network_path, &block_producers)
                            .expect("Failed to generate key pairs for block producers.");

                        let _libp2p_keys =
                            Keys::generate_libp2p_key_pairs(&network_path, &block_producers)
                                .expect("Failed to generate libp2p key pairs for block producers.");

                        //generate default genesis ledger
                        match DefaultLedgerGenerator::generate(&network_path, bp_keys) {
                            Ok(()) => info!("Successfully generated ledger!"),
                            Err(e) => error!("Error generating ledger: {}", e),
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
                let docker_compose_manager = DockerComposeManager::new(directory_manager);
                match docker_compose_manager.run_docker_compose(&cmd.network_id, &["up", "-d"]) {
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
                let docker_compose_manager = DockerComposeManager::new(directory_manager);
                match docker_compose_manager.run_docker_compose(&cmd.network_id, &["down"]) {
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
