mod cli;

use clap::Parser;
use cli::{Cli, Commands, NetworkCommand, NodeCommand};

fn main() {
    let cli: Cli = Cli::parse();
    match cli.commands {
        Commands::Network(net_cmd) => match net_cmd {
            NetworkCommand::Create(cmd) => {
                println!("Network create command with topology {}, genesis_ledger {}, and network_id {:?}.", cmd.topology.display(), cmd.genesis_ledger.display(), cmd.network_id.network_id);
            },
            NetworkCommand::Deploy(cmd) => {
                println!("Network deploy command with network_id {:?}.", cmd.network_id);
            },
            NetworkCommand::Destroy(cmd) => {
                println!("Network destroy command with network_id {:?}.", cmd.network_id);
            },
        },
        Commands::Node(node_cmd) => match node_cmd {
            NodeCommand::Start(cmd) => {
                println!("Node start command with node_id {:?}.", cmd.node_id);
            },
            NodeCommand::Stop(cmd) => {
                println!("Node stop command with node_id {:?}.", cmd.node_id);
            },
            NodeCommand::Logs(cmd) => {
                println!("Node logs command with node_id {:?}.", cmd.node_id);
            },
        },
    }
}
