use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about = "MiniMina - A Command-line Tool for Spinning up Mina Network Locally", long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[clap(subcommand)]
    Network(NetworkCommand),

    #[clap(subcommand)]
    Node(NodeCommand),
}

#[derive(Subcommand)]
pub enum NetworkCommand {
    Create(CreateNetworkArgs),
    Deploy(NetworkId),
    Destroy(NetworkId),
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
    Start(NodeId),
    Stop(NodeId),
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
    fn test_network_deploy_command() {
        let args = vec![
            "minimina",
            "network",
            "deploy",
            "--network-id",
            "test",
        ];

        let cli = Cli::parse_from(args);

        match cli.command {
            Command::Network(NetworkCommand::Deploy(args)) => {
                assert_eq!(args.network_id, "test");
            }
            _ => panic!("Unexpected command parsed"),
        }
    }

    #[test]
    fn test_network_destroy_command() {
        let args = vec![
            "minimina",
            "network",
            "destroy",
            "--network-id",
            "test",
        ];

        let cli = Cli::parse_from(args);

        match cli.command {
            Command::Network(NetworkCommand::Destroy(args)) => {
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