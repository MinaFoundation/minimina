//! # `minimina` Command-Line Interface (CLI)

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    author,
    version,
    about = "MiniMina - A Command-line Tool for Spinning up Local Mina Networks"
)]
#[command(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Manage local network
    #[clap(subcommand)]
    Network(NetworkCommand),

    /// Manage a single node
    #[clap(subcommand)]
    Node(NodeCommand),
}

#[derive(Subcommand)]
pub enum NetworkCommand {
    /// Create a local network
    Create(CreateNetworkArgs),
    /// Delete a local network
    Delete(NetworkId),
    /// List local networks
    List,
    /// Get status of a local network
    Status(NetworkId),
    /// Get details of a local network
    Info(NetworkId),
    /// Start a local network
    Start(StartNetworkArgs),
    /// Stop a local network
    Stop(NetworkId),
}

#[derive(Args, Debug, Clone)]
pub struct NetworkId {
    /// Network identifier
    #[clap(short, long, default_value = "default")]
    pub network_id: String,
}

#[derive(Args, Clone)]
pub struct CreateNetworkArgs {
    /// Path to the (JSON) topology file
    #[clap(short = 't', long)]
    pub topology: Option<std::path::PathBuf>,

    /// Path to the (JSON) genesis ledger/runtime config
    #[clap(short = 'g', long)]
    pub genesis_ledger: Option<std::path::PathBuf>,

    /// Network identifier
    #[clap(flatten)]
    pub network_id: NetworkId,
}

#[derive(Args, Clone)]
pub struct StartNetworkArgs {
    /// Network identifier
    #[clap(flatten)]
    pub network_id: NetworkId,

    /// Network identifier
    #[clap(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum NodeCommand {
    /// Start a node
    Start(NodeCommandArgs),
    /// Stop a node
    Stop(NodeCommandArgs),
    /// Dump the node's logs to stdout
    Logs(NodeCommandArgs),
    /// Dump the node's precomputed blocks to stdout
    DumpPrecomputedBlocks(NodeCommandArgs),
    /// Dump an archive node's data
    DumpArchiveData(NodeCommandArgs),
    /// Run the replayer on an archive node's db
    RunReplayer(ReplayerArgs),
}

#[derive(Args, Debug)]
pub struct NodeId {
    /// Node identifier
    #[clap(short = 'i', long)]
    pub node_id: String,
}

#[derive(Args, Debug)]
pub struct GlobalSlot {
    /// Global slot since genesis
    #[clap(short = 's', long)]
    pub global_slot: u32,
}

#[derive(Args, Debug)]
pub struct NodeCommandArgs {
    /// Node identifier
    #[clap(flatten)]
    pub node_id: NodeId,

    /// Network identifier
    #[clap(flatten)]
    pub network_id: NetworkId,

    /// Log level filter
    #[clap(short = 'l', long, default_value = "warn")]
    pub log_level: String,
}

#[derive(Args, Debug)]
pub struct ReplayerArgs {
    /// Node identifier
    #[clap(flatten)]
    pub node_id: NodeId,

    /// Network identifier
    #[clap(flatten)]
    pub network_id: NetworkId,

    /// Global slot since genesis
    #[clap(flatten)]
    pub start_slot_since_genesis: GlobalSlot,

    /// Log level filter
    #[clap(short = 'l', long, default_value = "warn")]
    pub log_level: String,
}

pub trait DefaultLogLevel {
    fn log_level(&self) -> &str;
}

trait LogLevel {
    fn log_level(&self) -> &str;
}

pub trait CommandWithNetworkId {
    fn network_id(&self) -> &str;
}

pub trait CommandWithNodeId {
    fn node_id(&self) -> &str;
}

impl ReplayerArgs {
    pub fn start_slot_since_genesis(&self) -> u32 {
        self.start_slot_since_genesis.global_slot
    }
}

macro_rules! network_id {
    ($name:path) => {
        impl CommandWithNetworkId for $name {
            fn network_id(&self) -> &str {
                &self.network_id.network_id
            }
        }
    };
}

macro_rules! node_id {
    ($name:path) => {
        impl CommandWithNodeId for $name {
            fn node_id(&self) -> &str {
                &self.node_id.node_id
            }
        }
    };
}

network_id!(StartNetworkArgs);
network_id!(CreateNetworkArgs);
network_id!(NodeCommandArgs);
network_id!(ReplayerArgs);

node_id!(NodeCommandArgs);
node_id!(ReplayerArgs);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_create_command() {
        let args = vec![
            "minimina",
            "network",
            "create",
            "--topology",
            "/path/to/file",
            "--genesis-ledger",
            "/path/to/dir",
            "--network-id",
            "test",
        ];

        let cli = Cli::parse_from(args);

        match cli.command {
            Command::Network(NetworkCommand::Create(args)) => {
                assert_eq!(
                    args.topology,
                    Some(std::path::PathBuf::from("/path/to/file"))
                );
                assert_eq!(
                    args.genesis_ledger,
                    Some(std::path::PathBuf::from("/path/to/dir"))
                );
                assert_eq!(args.network_id(), "test");
            }
            _ => panic!("Unexpected command parsed"),
        }
    }

    #[test]
    fn test_network_delete_command() {
        let args = vec!["minimina", "network", "delete", "--network-id", "test"];

        let cli = Cli::parse_from(args);

        match cli.command {
            Command::Network(NetworkCommand::Delete(args)) => {
                assert_eq!(args.network_id, "test");
            }
            _ => panic!("Unexpected command parsed"),
        }
    }

    #[test]
    fn test_network_list_command() {
        let args = vec!["minimina", "network", "list"];

        let cli = Cli::parse_from(args);

        match cli.command {
            Command::Network(NetworkCommand::List) => {}
            _ => panic!("Unexpected command parsed"),
        }
    }

    #[test]
    fn test_network_start_command() {
        let args = vec!["minimina", "network", "start", "--network-id", "test"];

        let cli = Cli::parse_from(args);

        match cli.command {
            Command::Network(NetworkCommand::Start(args)) => {
                assert_eq!(args.network_id(), "test");
            }
            _ => panic!("Unexpected command parsed"),
        }
    }

    #[test]
    fn test_network_stop_command() {
        let args = vec!["minimina", "network", "stop", "--network-id", "test"];

        let cli = Cli::parse_from(args);

        match cli.command {
            Command::Network(NetworkCommand::Stop(args)) => {
                assert_eq!(args.network_id, "test");
            }
            _ => panic!("Unexpected command parsed"),
        }
    }

    #[test]
    fn test_node_start_command() {
        let args = vec!["minimina", "node", "start", "--node-id", "test"];

        let cli = Cli::parse_from(args);

        match cli.command {
            Command::Node(NodeCommand::Start(args)) => {
                assert_eq!(args.node_id(), "test");
                assert_eq!(args.network_id(), "default");
            }
            _ => panic!("Unexpected command parsed"),
        }
    }

    #[test]
    fn test_node_stop_command() {
        let args = vec![
            "minimina",
            "node",
            "stop",
            "--node-id",
            "test",
            "--network-id",
            "banana",
        ];

        let cli = Cli::parse_from(args);

        match cli.command {
            Command::Node(NodeCommand::Stop(args)) => {
                assert_eq!(args.node_id(), "test");
                assert_eq!(args.network_id(), "banana");
            }
            _ => panic!("Unexpected command parsed"),
        }
    }

    #[test]
    fn test_node_logs_command() {
        let args = vec!["minimina", "node", "logs", "--node-id", "test"];

        let cli = Cli::parse_from(args);

        match cli.command {
            Command::Node(NodeCommand::Logs(args)) => {
                assert_eq!(args.node_id(), "test");
                assert_eq!(args.network_id(), "default");
            }
            _ => panic!("Unexpected command parsed"),
        }
    }
}
