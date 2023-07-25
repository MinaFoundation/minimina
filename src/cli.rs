use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about = "MiniMina - A Command-line Tool for Spinning up Mina Network Locally")]
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
    /// Start a local network
    Start(NetworkId),
    /// Stop a local network
    Stop(NetworkId),
}

#[derive(Args, Debug)]
pub struct NetworkId {
    #[clap(short, long, default_value = "default")]
    pub network_id: String,
}

#[derive(Args)]
pub struct CreateNetworkArgs {
    #[clap(short = 't', long)]
    pub topology: std::path::PathBuf,
    
    #[clap(short = 'g', long)]
    pub genesis_ledger: std::path::PathBuf,
    
    #[clap(flatten)]
    pub network_id: NetworkId,
}

#[derive(Subcommand)]
pub enum NodeCommand {
    /// Start a node
    Start(NodeId),
    /// Stop a node
    Stop(NodeId),
    /// Get logs from a node
    Logs(NodeId),
}

#[derive(Args, Debug)]
pub struct NodeId {
    #[clap(short, long)]
    pub node_id: String,
}

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
                assert_eq!(args.topology, std::path::PathBuf::from("/path/to/file"));
                assert_eq!(args.genesis_ledger, std::path::PathBuf::from("/path/to/dir"));
                assert_eq!(args.network_id.network_id, "test");
            }
            _ => panic!("Unexpected command parsed"),
        }
    }

    #[test]
    fn test_network_delete_command() {
        let args = vec![
            "minimina",
            "network",
            "delete",
            "--network-id",
            "test",
        ];

        let cli = Cli::parse_from(args);

        match cli.command {
            Command::Network(NetworkCommand::Delete(args)) => {
                assert_eq!(args.network_id, "test");
            }
            _ => panic!("Unexpected command parsed"),
        }
    }

    #[test]
    fn test_network_start_command() {
        let args = vec![
            "minimina",
            "network",
            "start",
            "--network-id",
            "test",
        ];

        let cli = Cli::parse_from(args);

        match cli.command {
            Command::Network(NetworkCommand::Start(args)) => {
                assert_eq!(args.network_id, "test");
            }
            _ => panic!("Unexpected command parsed"),
        }
    }

    #[test]
    fn test_network_stop_command() {
        let args = vec![
            "minimina",
            "network",
            "stop",
            "--network-id",
            "test",
        ];

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
        let args = vec![
            "minimina",
            "node",
            "start",
            "--node-id",
            "test",
        ];

        let cli = Cli::parse_from(args);

        match cli.command {
            Command::Node(NodeCommand::Start(args)) => {
                assert_eq!(args.node_id, "test");
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
        ];

        let cli = Cli::parse_from(args);

        match cli.command {
            Command::Node(NodeCommand::Stop(args)) => {
                assert_eq!(args.node_id, "test");
            }
            _ => panic!("Unexpected command parsed"),
        }
    }

    #[test]
    fn test_node_logs_command() {
        let args = vec![
            "minimina",
            "node",
            "logs",
            "--node-id",
            "test",
        ];

        let cli = Cli::parse_from(args);

        match cli.command {
            Command::Node(NodeCommand::Logs(args)) => {
                assert_eq!(args.node_id, "test");
            }
            _ => panic!("Unexpected command parsed"),
        }
    }
}