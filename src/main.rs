mod cli;
mod cmd;
mod directory_manager;
mod docker_compose_manager;

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

                let subdirectories = ["block_producer_keys", "libp2p_keys", "nodes"];

                directory_manager
                    .create_subdirectories(cmd.network_id(), &subdirectories)
                    .expect("Failed to create subdirectories");

                directory_manager
                    .chmod_network_subdirectories(cmd.network_id(), &subdirectories, 0o700)
                    .expect("Failed to set permissions for subdirectories");

                // pattern match on &cmd.topology
                match &cmd.topology {
                    Some(topology) => {
                        info!(
                            "Copying topology from '{}' to network directory.",
                            topology.display()
                        );
                    }
                    None => {
                        //generate key-pairs for default topology
                        //generate default topology
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
                        //generate default genesis ledger
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
