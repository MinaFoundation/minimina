mod cli;

use clap::Parser;
use cli::{Cli, Command, NetworkCommand, NodeCommand};

fn main() {
    let cli: Cli = Cli::parse();
    match cli.command {
        Command::Network(net_cmd) => match net_cmd {
            NetworkCommand::Create(cmd) => {
                println!("Network create command with topology {}, genesis_ledger {}, and network_id {}.", cmd.topology.display(), cmd.genesis_ledger.display(), cmd.network_id());
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
