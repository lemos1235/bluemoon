use crate::config::Config;
use crate::utils::dirs;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::{env::current_exe, process::Command as StdCommand};
use tokio::time::Duration;

// Windows only

const SERVICE_URL: &str = "http://127.0.0.1:33211";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResponseBody {
    pub core_type: Option<String>,
    pub bin_path: String,
    pub config_dir: String,
    pub log_file: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JsonResponse {
    pub code: u64,
    pub msg: String,
    pub data: Option<ResponseBody>,
}

#[cfg(target_os = "windows")]
pub async fn reinstall_service() -> Result<()> {
    log::info!(target:"app", "reinstall service");

    use deelevate::{PrivilegeLevel, Token};
    use runas::Command as RunasCommand;
    use std::os::windows::process::CommandExt;

    let binary_path = dirs::service_path()?;
    let install_path = binary_path.with_file_name("prepare-service.exe");

    if !install_path.exists() {
        bail!(format!("installer not found: {install_path:?}"));
    }

    let token = Token::with_current_process()?;
    let level = token.privilege_level()?;
    let _ = match level {
        PrivilegeLevel::NotPrivileged => RunasCommand::new(install_path.clone()).arg("uninstall").show(false).status()?,
        _ => StdCommand::new(install_path.clone())
            .arg("uninstall")
            .creation_flags(0x08000000)
            .status()?,
    };

    let status = match level {
        PrivilegeLevel::NotPrivileged => RunasCommand::new(install_path.clone()).arg("install").show(false).status()?,
        _ => StdCommand::new(install_path.clone())
            .arg("install")
            .creation_flags(0x08000000)
            .status()?,
    };

    if !status.success() {
        bail!(
            "failed to install service with status {}",
            status.code().unwrap()
        );
    }

    Ok(())
}

#[cfg(target_os = "linux")]
pub async fn reinstall_service() -> Result<()> {
    log::info!(target:"app", "reinstall service");
    use users::get_effective_uid;

    let install_path = tauri::utils::platform::current_exe()?.with_file_name("prepare-service");

    if !install_path.exists() {
        bail!(format!("installer not found: {install_path:?}"));
    }

    let install_shell: String = install_path.to_string_lossy().replace(" ", "\\ ");

    let elevator = crate::utils::help::linux_elevator();
    let status = match get_effective_uid() {
        0 => StdCommand::new(&install_shell).arg("uninstall").status()?,
        _ => StdCommand::new(elevator.clone())
            .arg("sh")
            .arg("-c")
            .arg(&install_shell)
            .arg("uninstall")
            .status()?,
    };
    log::info!(target:"app", "status code:{}", status.code().unwrap());

    let status = match get_effective_uid() {
        0 => StdCommand::new(&install_shell).arg("install").status()?,
        _ => StdCommand::new(elevator.clone())
            .arg("sh")
            .arg("-c")
            .arg(&install_shell)
            .arg("install")
            .status()?,
    };

    if !status.success() {
        bail!(
            "failed to install service with status {}",
            status.code().unwrap()
        );
    }

    Ok(())
}

#[cfg(target_os = "macos")]
pub async fn reinstall_service() -> Result<()> {
    log::info!(target:"app", "reinstall service");

    let binary_path = dirs::service_path()?;
    let install_path = binary_path.with_file_name("prepare-service");

    if !install_path.exists() {
        bail!(format!("installer not found: {install_path:?}"));
    }

    let install_shell: String = install_path.to_string_lossy().into_owned();
    let command = format!(
        r#"do shell script "sudo '{install_shell}' uninstall && sudo '{install_shell}' install" with administrator privileges"#
    );

    log::debug!(target: "app", "command: {}", command);

    let status = StdCommand::new("osascript")
        .args(vec!["-e", &command])
        .status()?;

    if !status.success() {
        bail!(
            "failed to install service with status {}",
            status.code().unwrap()
        );
    }
    Ok(())
}

/// check the windows service status
pub async fn check_service() -> Result<JsonResponse> {
    let url = format!("{SERVICE_URL}/get_clash");
    let response = reqwest::ClientBuilder::new()
        .no_proxy()
        .timeout(Duration::from_secs(3))
        .build()?
        .get(url)
        .send()
        .await
        .context("failed to connect to the Clash Verge Service")?
        .json::<JsonResponse>()
        .await
        .context("failed to parse the Clash Verge Service response")?;

    Ok(response)
}

/// start the clash by service
pub(super) async fn run_core_by_service(config_file: &PathBuf) -> Result<()> {
    let clash_core = { Config::verge().latest().clash_core.clone() };
    let clash_core = clash_core.unwrap_or("verge-mihomo".into());

    let bin_ext = if cfg!(windows) { ".exe" } else { "" };
    let clash_bin = format!("{clash_core}{bin_ext}");
    let bin_path = current_exe()?.with_file_name(clash_bin);
    let bin_path = dirs::path_to_str(&bin_path)?;

    let config_dir = dirs::app_home_dir()?;
    let config_dir = dirs::path_to_str(&config_dir)?;

    let log_path = dirs::service_log_file()?;
    let log_path = dirs::path_to_str(&log_path)?;

    let config_file = dirs::path_to_str(config_file)?;

    let mut map = HashMap::new();
    map.insert("core_type", clash_core.as_str());
    map.insert("bin_path", bin_path);
    map.insert("config_dir", config_dir);
    map.insert("config_file", config_file);
    map.insert("log_file", log_path);

    log::info!(target:"app", "start service: {:?}", map.clone());

    let url = format!("{SERVICE_URL}/start_clash");
    let _ = reqwest::ClientBuilder::new()
        .no_proxy()
        .build()?
        .post(url)
        .json(&map)
        .send()
        .await
        .context("failed to connect to the Clash Verge Service")?;

    Ok(())
}

/// stop the clash by service
pub(super) async fn stop_core_by_service() -> Result<()> {
    let url = format!("{SERVICE_URL}/stop_clash");
    let _ = reqwest::ClientBuilder::new()
        .no_proxy()
        .build()?
        .post(url)
        .send()
        .await
        .context("failed to connect to the Clash Verge Service")?;

    Ok(())
}
