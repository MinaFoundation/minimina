//! # Utils module
//!
//! This module provides utility functions to run external commands
//! and fetch the UID and GID of the current user.

use log::{debug, error};
use std::{
    io::{self, ErrorKind},
    process::{Command, Output},
};

/// Run an external command and capture its output.
/// Logs the command, its output, and any potential errors.
///
/// # Arguments
///
/// * `cmd` - A string slice that holds the name of the command.
/// * `args` - A slice of string slices that contain the arguments to the command.
///
/// # Returns
///
/// * `io::Result<Output>` - The output from the command execution.
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

/// Fetch the UID and GID of the current user.
///
/// # Returns
///
/// * `Option<String>` - A formatted string "UID:GID", or `None` if unable to retrieve.
pub fn get_current_user_uid_gid() -> Option<String> {
    let current_user = users::get_current_uid();
    let current_group = users::get_current_gid();

    Some(format!("{}:{}", current_user, current_group))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_command() {
        let output = run_command("echo", &["hello", "world"]).unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), "hello world\n");
    }

    #[test]
    fn test_get_current_user_uid_gid() {
        let uid_gid = get_current_user_uid_gid().unwrap();
        assert!(uid_gid.contains(':'));
    }
}
