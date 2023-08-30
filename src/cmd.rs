use log::{debug, error};
use std::{
    io::{self, ErrorKind},
    process::{Command, Output},
};

pub fn run_command(cmd: &str, args: &[&str]) -> io::Result<Output> {
    debug!("Running command: {} {}", cmd, args.join(" "));
    let output = Command::new(cmd).args(args).output();

    match output {
        Ok(output) => {
            debug!("status: {}", output.status);
            debug!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            debug!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            Ok(output)
        }
        Err(e) => {
            error!("Failed to run command: {}", e);
            Err(io::Error::new(ErrorKind::Other, e))
        }
    }
}
