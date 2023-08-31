use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use log::{debug, info};

use crate::utils::{get_current_user_uid_gid, run_command};

#[derive(Debug)]
pub struct ServiceKeys {
    pub key_string: String,
    pub key_path_docker: String,
}

pub struct KeysManager {
    pub network_path: PathBuf,
    pub docker_image: String,
}

impl KeysManager {
    pub fn new(network_path: &Path, docker_image: &str) -> Self {
        KeysManager {
            network_path: network_path.to_path_buf(),
            docker_image: docker_image.to_string(),
        }
    }
    // generate bp key pair for single service
    pub fn generate_bp_key_pair(&self, service_name: &str) -> std::io::Result<ServiceKeys> {
        info!("Creating block producer keys for: {}", service_name);
        let uid_gid = match get_current_user_uid_gid() {
            Some(uid_gid) => uid_gid,
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Unable to retrieve UID and GID of current user",
                ))
            }
        };

        let key_subdir = "block_producer_keys";
        let volume_path = format!("{}:/local-network", self.network_path.to_str().unwrap());
        let pkey_path = format!("/local-network/{}/{}", key_subdir, service_name);
        let args = vec![
            "run",
            "--rm",
            "--user",
            uid_gid.as_str(),
            "--env",
            "MINA_PRIVKEY_PASS=naughty blue worm",
            "--entrypoint",
            "mina",
            "-v",
            &volume_path,
            self.docker_image.as_str(),
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

        let keys = ServiceKeys {
            key_string: public_key,
            key_path_docker: pkey_path,
        };
        debug!("Generated keypair: {:?}", keys);
        Ok(keys)
    }

    // generate bp key pairs for multiple services
    pub fn generate_bp_key_pairs(
        &self,
        service_names: &[&str],
    ) -> std::io::Result<HashMap<String, ServiceKeys>> {
        let mut public_keys = HashMap::new();
        for &service_name in service_names {
            let public_key = self.generate_bp_key_pair(service_name)?;
            public_keys.insert(service_name.to_string(), public_key);
        }
        Ok(public_keys)
    }

    // generate libp2p key pair for single service
    pub fn generate_libp2p_key_pair(&self, service_name: &str) -> std::io::Result<ServiceKeys> {
        info!("Creating libp2p keys for: {}", service_name);

        let key_subdir = "libp2p_keys";
        let volume_path = format!("{}:/local-network", self.network_path.to_str().unwrap());
        let pkey_path = format!("/local-network/{}/{}", key_subdir, service_name);

        let args = vec![
            "run",
            "--rm",
            // "--user",
            // "1000:1000",
            "--env",
            "MINA_LIBP2P_PASS=naughty blue worm",
            "--entrypoint",
            "mina",
            "-v",
            &volume_path,
            self.docker_image.as_str(),
            "libp2p",
            "generate-keypair",
            "-privkey-path",
            &pkey_path,
        ];

        let output = run_command("docker", &args)?;

        // Extract the full keypair
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let keypair = stdout_str.replace("libp2p keypair:", "").trim().to_string();
        let keys = ServiceKeys {
            key_string: keypair,
            key_path_docker: pkey_path,
        };
        debug!("Generated keypair: {:?}", keys);
        Ok(keys)
    }

    // generate libp2p key pairs for multiple services
    pub fn generate_libp2p_key_pairs(
        &self,
        service_names: &[&str],
    ) -> std::io::Result<HashMap<String, ServiceKeys>> {
        let mut keypairs = HashMap::new();
        for &service_name in service_names {
            let keypair = self.generate_libp2p_key_pair(service_name)?;
            keypairs.insert(service_name.to_string(), keypair);
        }
        Ok(keypairs)
    }
}
