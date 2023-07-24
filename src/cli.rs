use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about = "MiniMina - A Command-line Tool for Spinning up Mina Network Locally", long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub commands: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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