use std::{collections::HashMap, path::Path};

use log::info;

use crate::cmd::run_command;

pub struct Keys;

impl Keys {
    // generate bp key pair for single service
    pub fn generate_bp_key_pair(
        network_path: &Path,
        service_name: &str,
    ) -> std::io::Result<String> {
        let mut bp_dir = network_path.to_path_buf();
        bp_dir.push("block_producer_keys");

        info!(
            "Creating block producer keys for: {:?}/{}",
            bp_dir, service_name
        );

        let volume_path = format!("{}:/keys", bp_dir.to_str().unwrap());
        let pkey_path = format!("/keys/{}", service_name);
        let args = vec![
            "run",
            "--rm",
            "--env",
            "MINA_PRIVKEY_PASS=naughty blue worm",
            "--entrypoint",
            "mina",
            "-v",
            &volume_path,
            "gcr.io/o1labs-192920/mina-daemon:2.0.0rampup3-bfd1009-buster-berkeley",
            "advanced",
            "generate-keypair",
            "-privkey-path",
            &pkey_path,
        ];

        let output = run_command("docker", &args)?;

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let public_key_line = stdout_str
            .lines()
            .find(|line| line.contains("Public key: "))
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "Public key not found")
            })?;

        let public_key = public_key_line
            .split(": ")
            .nth(1)
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Public key format is incorrect",
                )
            })?
            .to_string();

        Ok(public_key)
    }

    // generate bp key pairs for multiple services
    pub fn generate_bp_key_pairs(
        network_path: &Path,
        service_names: &[&str],
    ) -> std::io::Result<HashMap<String, String>> {
        let mut public_keys = HashMap::new();
        for &service_name in service_names {
            let public_key = Self::generate_bp_key_pair(network_path, service_name)?;
            public_keys.insert(service_name.to_string(), public_key);
        }
        Ok(public_keys)
    }

    // generate libp2p key pair for single service
    pub fn generate_libp2p_key_pair(
        network_path: &Path,
        service_name: &str,
    ) -> std::io::Result<String> {
        let mut libp2p_dir = network_path.to_path_buf();
        libp2p_dir.push("libp2p_keys");

        info!(
            "Creating libp2p keys for: {:?}/{}",
            libp2p_dir, service_name
        );

        let volume_path = format!("{}:/keys", libp2p_dir.to_str().unwrap());
        let pkey_path = format!("/keys/{}", service_name);

        let args = vec![
            "run",
            "--rm",
            "--env",
            "MINA_LIBP2P_PASS=naughty blue worm",
            "--entrypoint",
            "mina",
            "-v",
            &volume_path,
            "gcr.io/o1labs-192920/mina-daemon:2.0.0rampup3-bfd1009-buster-berkeley",
            "libp2p",
            "generate-keypair",
            "-privkey-path",
            &pkey_path,
        ];

        let output = run_command("docker", &args)?;

        // Extract the full keypair
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let keypair = stdout_str.replace("libp2p keypair:", "").trim().to_string();
        Ok(keypair)
    }

    // generate libp2p key pairs for multiple services
    pub fn generate_libp2p_key_pairs(
        network_path: &Path,
        service_names: &[&str],
    ) -> std::io::Result<HashMap<String, String>> {
        let mut keypairs = HashMap::new();
        for &service_name in service_names {
            let keypair = Self::generate_libp2p_key_pair(network_path, service_name)?;
            keypairs.insert(service_name.to_string(), keypair);
        }
        Ok(keypairs)
    }
}
