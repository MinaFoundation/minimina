mod cli;
mod cmd;
mod directory_manager;
mod docker_compose_manager;

use clap::Parser;
use cli::{Cli, Command, NetworkCommand, NodeCommand};
use directory_manager::DirectoryManager;
use docker_compose_manager::DockerComposeManager;

fn main() {
    let cli: Cli = Cli::parse();
    let directory_manager = DirectoryManager::new();

    match cli.command {
        Command::Network(net_cmd) => match net_cmd {
            NetworkCommand::Create(cmd) => {
                println!("Creating network with network-id '{}'.", cmd.network_id());
                directory_manager
                    .create_network_directory(cmd.network_id())
                    .expect("Failed to create network directory");

                let subdirectories = ["block_producer_keys", "libp2p_keys", "nodes"];

                directory_manager
                    .create_subdirectories(cmd.network_id(), &subdirectories)
                    .expect("Failed to create subdirectories");

                directory_manager
                    .chmod_network_subdirectories(cmd.network_id(), &subdirectories, 0o700)
                    .expect("Failed to chmod subdirectories");

                let docker_compose_generator = DockerComposeManager::new(directory_manager);
                docker_compose_generator
                    .generate_docker_compose(cmd.network_id(), &cmd.topology)
                    .expect("Failed to generate docker-compose.yaml");

                println!(
                    "Network '{}' created successfully using topology '{}' and genesis ledger '{}'.",
                    cmd.network_id(),
                    cmd.topology.display(),
                    cmd.genesis_ledger.display()
                );
            }
            NetworkCommand::Delete(cmd) => {
                match directory_manager.delete_network_directory(&cmd.network_id) {
                    Ok(_) => {}
                    Err(e) => {
                        println!(
                            "Failed to delete network directory for network_id '{}' with error = {}",
                            cmd.network_id, e
                        );
                        return;
                    }
                }

                println!("Network '{}' deleted successfully.", cmd.network_id);
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
                        println!(
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
                        println!(
                            "Failed to stop network with network_id '{}' with error = {}",
                            cmd.network_id, e
                        );
                    }
                }
            }
        },
        Command::Node(node_cmd) => match node_cmd {
            NodeCommand::Start(cmd) => {
                println!(
                    "Node start command with node_id {}, network_id {}.",
                    cmd.node_id(),
                    cmd.network_id()
                );
            }
            NodeCommand::Stop(cmd) => {
                println!(
                    "Node stop command with node_id {}, network_id {}.",
                    cmd.node_id(),
                    cmd.network_id()
                );
            }
            NodeCommand::Logs(cmd) => {
                println!(
                    "Node logs command with node_id {}, network_id {}.",
                    cmd.node_id(),
                    cmd.network_id()
                );
            }
        },
    }
}
