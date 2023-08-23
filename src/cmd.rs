use log::info;
use std::process::{Command, Output};

pub fn run_command(cmd: &str, args: &[&str]) -> std::io::Result<Output> {
    info!("Running command: {} {}", cmd, args.join(" "));
    Command::new(cmd).args(args).output()
}
