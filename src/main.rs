mod cli;
mod directory_manager;
mod docker_compose_generator;

use clap::Parser;
use cli::{Cli, Command, NetworkCommand, NodeCommand};
use directory_manager::DirectoryManager;
use docker_compose_generator::DockerComposeGenerator;

fn main() {
    let cli: Cli = Cli::parse();
    let directory_manager = DirectoryManager::new();

    match cli.command {
        Command::Network(net_cmd) => match net_cmd {
            NetworkCommand::Create(cmd) => {
                println!("Creating network with network-id = {}.", cmd.network_id());
                directory_manager
                    .create_network_directory(cmd.network_id())
                    .expect("Failed to create network directory");
                directory_manager
                    .create_subdirectories(
                        cmd.network_id(),
                        &[
                            "fish_keys",
                            "libp2p_keys",
                            "nodes",
                            "service-keys",
                            "snark_coordinator_keys",
                            "whale_keys",
                            "zkapp_keys",
                        ],
                    )
                    .expect("Failed to create subdirectories");

                let docker_compose_generator = DockerComposeGenerator::new(directory_manager);
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
                println!("Network delete command with network_id {}.", cmd.network_id);
            }
            NetworkCommand::Start(cmd) => {
                println!("Network start command with network_id {}.", cmd.network_id);
            }
            NetworkCommand::Stop(cmd) => {
                println!("Network stop command with network_id {}.", cmd.network_id);
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
