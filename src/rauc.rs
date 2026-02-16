use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;

#[derive(Debug, Serialize, Deserialize)]
pub struct RaucStatus {
    pub compatible: String,
    pub variant: String,
    pub booted: String,
    pub boot_primary: String,
    pub slots: Vec<serde_json::Value>,
    #[serde(rename = "artifact-repositories")]
    pub artifact_repositories: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RaucBundleInfo {
    pub compatible: String,
    pub version: String,
    pub description: String,
    pub build: String,
    pub format: String,
    pub hooks: Vec<serde_json::Value>,
    pub hash: String,
    pub images: Vec<serde_json::Value>,
}

#[derive(Clone)]
pub enum RaucMode {
    Development {
        ssh_host: String,
        ssh_password: String,
    },
    Production,
}

#[derive(Clone)]
pub struct RaucClient {
    mode: RaucMode,
}

impl RaucClient {
    pub fn new(mode: RaucMode) -> Self {
        Self { mode }
    }

    /// Helper method to execute SSH commands with sshpass in development mode
    fn execute_ssh_command(&self, args: &[&str]) -> Result<std::process::Output, String> {
        match &self.mode {
            RaucMode::Development {
                ssh_host,
                ssh_password,
            } => {
                let mut ssh_args = vec![
                    "-o",
                    "StrictHostKeyChecking=no",
                    "-o",
                    "UserKnownHostsFile=/dev/null",
                    ssh_host.as_str(),
                ];
                ssh_args.extend_from_slice(args);

                Command::new("sshpass")
                    .arg("-p")
                    .arg(ssh_password)
                    .arg("ssh")
                    .args(&ssh_args)
                    .output()
                    .map_err(|e| format!("Failed to execute sshpass ssh command: {}", e))
            }
            RaucMode::Production => Err("SSH command called in production mode".to_string()),
        }
    }

    /// Helper method to execute SCP commands with sshpass in development mode
    fn execute_scp_command(
        &self,
        local_path: &str,
        remote_path: &str,
    ) -> Result<std::process::Output, String> {
        match &self.mode {
            RaucMode::Development {
                ssh_host,
                ssh_password,
            } => Command::new("sshpass")
                .arg("-p")
                .arg(ssh_password)
                .arg("scp")
                .arg("-p")
                .arg("-o")
                .arg("StrictHostKeyChecking=no")
                .arg("-o")
                .arg("UserKnownHostsFile=/dev/null")
                .arg(local_path)
                .arg(format!("{}:{}", ssh_host, remote_path))
                .output()
                .map_err(|e| format!("Failed to execute scp command: {}", e)),
            RaucMode::Production => Err("SCP command called in production mode".to_string()),
        }
    }

    pub async fn copy_file_to_target(
        &self,
        local_path: &str,
        target_path: &str,
    ) -> Result<String, String> {
        match &self.mode {
            RaucMode::Development { ssh_host, .. } => {
                // Get local file size before transfer
                let local_size = std::fs::metadata(local_path)
                    .map_err(|e| format!("Failed to read local file metadata: {}", e))?
                    .len();

                // Ensure parent directory exists on target
                let parent_dir = std::path::Path::new(target_path)
                    .parent()
                    .ok_or_else(|| "Invalid target path".to_string())?;

                let mkdir_output =
                    self.execute_ssh_command(&[&format!("mkdir -p {}", parent_dir.display())])?;

                if !mkdir_output.status.success() {
                    let stderr = String::from_utf8_lossy(&mkdir_output.stderr);
                    return Err(format!("Failed to create remote directory: {}", stderr));
                }

                // Copy file to target
                let scp_output = self.execute_scp_command(local_path, target_path)?;

                if !scp_output.status.success() {
                    let stderr = String::from_utf8_lossy(&scp_output.stderr);
                    return Err(format!("scp command failed: {}", stderr));
                }

                // Verify remote file size
                let stat_output =
                    self.execute_ssh_command(&[&format!("stat -c %s {}", target_path)])?;

                if stat_output.status.success() {
                    let remote_size_str = String::from_utf8_lossy(&stat_output.stdout);
                    if let Ok(remote_size) = remote_size_str.trim().parse::<u64>() {
                        if remote_size != local_size {
                            return Err(format!(
                                "File size mismatch after transfer: local {} bytes, remote {} bytes",
                                local_size, remote_size
                            ));
                        }
                        return Ok(format!(
                            "File copied to {} successfully ({} bytes verified)",
                            ssh_host, remote_size
                        ));
                    }
                }

                Ok(format!(
                    "File copied to {} successfully ({} bytes)",
                    ssh_host, local_size
                ))
            }
            RaucMode::Production => {
                // In production, file is already local, no copy needed
                Ok("File already on target system (production mode)".to_string())
            }
        }
    }

    async fn execute_command(&self, args: &[&str]) -> Result<String, String> {
        let output = match &self.mode {
            RaucMode::Development { .. } => {
                let mut rauc_args = vec!["rauc"];
                rauc_args.extend_from_slice(args);
                self.execute_ssh_command(&rauc_args)?
            }
            RaucMode::Production => Command::new("rauc")
                .args(args)
                .output()
                .map_err(|e| format!("Failed to execute rauc command: {}", e))?,
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("rauc command failed: {}", stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn get_status(&self) -> Result<RaucStatus, String> {
        let stdout = self
            .execute_command(&["status", "--output-format=json"])
            .await?;

        let status: RaucStatus = serde_json::from_str(&stdout)
            .map_err(|e| format!("Failed to parse rauc status JSON: {}", e))?;

        Ok(status)
    }

    pub async fn get_bundle_info(&self, bundle_path: &str) -> Result<RaucBundleInfo, String> {
        let stdout = self
            .execute_command(&["info", "--output-format=json", bundle_path])
            .await?;

        let info: RaucBundleInfo = serde_json::from_str(&stdout)
            .map_err(|e| format!("Failed to parse rauc bundle info JSON: {}", e))?;

        Ok(info)
    }

    pub async fn install_bundle(
        &self,
        bundle_path: &str,
    ) -> Result<impl tokio_stream::Stream<Item = Result<String, std::io::Error>>, String> {
        let (mut command, description) = match &self.mode {
            RaucMode::Development {
                ssh_host,
                ssh_password,
            } => {
                let ssh_args = vec![
                    "-o".to_string(),
                    "StrictHostKeyChecking=no".to_string(),
                    "-o".to_string(),
                    "UserKnownHostsFile=/dev/null".to_string(),
                    ssh_host.clone(),
                    "rauc".to_string(),
                    "install".to_string(),
                    bundle_path.to_string(),
                ];

                let mut cmd = TokioCommand::new("sshpass");
                cmd.arg("-p").arg(ssh_password).arg("ssh").args(&ssh_args);

                (cmd, "ssh install")
            }
            RaucMode::Production => {
                let mut cmd = TokioCommand::new("rauc");
                cmd.arg("install").arg(bundle_path);
                (cmd, "local install")
            }
        };

        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn {} command: {}", description, e))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to capture stdout".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "Failed to capture stderr".to_string())?;

        let stream = async_stream::stream! {
            let mut stdout_reader = BufReader::new(stdout).lines();
            let mut stderr_reader = BufReader::new(stderr).lines();

            loop {
                tokio::select! {
                    result = stdout_reader.next_line() => {
                        match result {
                            Ok(Some(line)) => yield Ok(format!("[OUT] {}\n", line)),
                            Ok(None) => break,
                            Err(e) => yield Err(e),
                        }
                    }
                    result = stderr_reader.next_line() => {
                        match result {
                            Ok(Some(line)) => yield Ok(format!("[ERR] {}\n", line)),
                            Ok(None) => {},
                            Err(e) => yield Err(e),
                        }
                    }
                }
            }

            // Wait for process to complete
            match child.wait().await {
                Ok(status) => {
                    if status.success() {
                        yield Ok("\n[DONE] Installation completed successfully\n".to_string());
                    } else {
                        yield Ok(format!("\n[ERROR] Installation failed with status: {}\n", status));
                    }
                }
                Err(e) => {
                    yield Ok(format!("\n[ERROR] Failed to wait for process: {}\n", e));
                }
            }
        };

        Ok(stream)
    }

    pub async fn reboot(&self) -> Result<String, String> {
        match &self.mode {
            RaucMode::Development { .. } => {
                let output = self.execute_ssh_command(&["reboot"])?;

                // Note: reboot might not return successfully as the connection will be dropped
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    // Don't treat this as a fatal error, as the connection may drop
                    eprintln!("Reboot stderr (may be expected): {}", stderr);
                }

                Ok("Reboot command sent".to_string())
            }
            RaucMode::Production => {
                let _ = Command::new("reboot")
                    .output()
                    .map_err(|e| format!("Failed to execute reboot command: {}", e))?;

                Ok("Reboot initiated".to_string())
            }
        }
    }
}
