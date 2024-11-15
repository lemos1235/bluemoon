use super::{Draft, IClashTemp, IMoon, IProfiles, IRuntime, IVerge};
use crate::{
    config::PrfItem,
    enhance,
    utils::{dirs, help},
};
use anyhow::{anyhow, Result};
use once_cell::sync::OnceCell;
use std::{env::temp_dir, path::PathBuf};

pub const RUNTIME_CONFIG: &str = "clash-verge.yaml";
pub const CHECK_CONFIG: &str = "clash-verge-check.yaml";

pub struct Config {
    clash_config: Draft<IClashTemp>,
    verge_config: Draft<IVerge>,
    moon_config: Draft<IMoon>,
    profiles_config: Draft<IProfiles>,
    runtime_config: Draft<IRuntime>,
}

impl Config {
    pub fn global() -> &'static Config {
        static CONFIG: OnceCell<Config> = OnceCell::new();

        CONFIG.get_or_init(|| Config {
            clash_config: Draft::from(IClashTemp::new()),
            verge_config: Draft::from(IVerge::new()),
            moon_config: Draft::from(IMoon::new()),
            profiles_config: Draft::from(IProfiles::new()),
            runtime_config: Draft::from(IRuntime::new()),
        })
    }

    pub fn clash() -> Draft<IClashTemp> {
        Self::global().clash_config.clone()
    }

    pub fn verge() -> Draft<IVerge> {
        Self::global().verge_config.clone()
    }

    pub fn moon() -> Draft<IMoon> {
        Self::global().moon_config.clone()
    }

    pub fn profiles() -> Draft<IProfiles> {
        Self::global().profiles_config.clone()
    }

    pub fn runtime() -> Draft<IRuntime> {
        Self::global().runtime_config.clone()
    }

    /// 初始化订阅
    pub async fn init_config() -> Result<()> {
        if Self::profiles()
            .data()
            .get_item(&"Merge".to_string())
            .is_err()
        {
            let merge_item = PrfItem::from_merge(Some("Merge".to_string()))?;
            Self::profiles().data().append_item(merge_item.clone())?;
        }
        if Self::profiles()
            .data()
            .get_item(&"Script".to_string())
            .is_err()
        {
            let script_item = PrfItem::from_script(Some("Script".to_string()))?;
            Self::profiles().data().append_item(script_item.clone())?;
        }
        crate::log_err!(Self::generate().await);
        if let Err(err) = Self::generate_file(ConfigType::Run) {
            log::error!(target: "app", "{err}");

            let runtime_path = dirs::app_home_dir()?.join(RUNTIME_CONFIG);
            // 如果不存在就将默认的clash文件拿过来
            if !runtime_path.exists() {
                help::save_yaml(
                    &runtime_path,
                    &Config::clash().latest().0,
                    Some("# Clash Verge Runtime"),
                )?;
            }
        }
        Ok(())
    }

    /// 将订阅丢到对应的文件中
    pub fn generate_file(typ: ConfigType) -> Result<PathBuf> {
        let path = match typ {
            ConfigType::Run => dirs::app_home_dir()?.join(RUNTIME_CONFIG),
            ConfigType::Check => temp_dir().join(CHECK_CONFIG),
        };

        let runtime = Config::runtime();
        let runtime = runtime.latest();
        let config = runtime
            .config
            .as_ref()
            .ok_or(anyhow!("failed to get runtime config"))?;

        help::save_yaml(&path, &config, Some("# Generated by Clash Verge"))?;
        Ok(path)
    }

    /// 生成订阅存好
    pub async fn generate() -> Result<()> {
        let (config, exists_keys, logs) = enhance::enhance().await;

        *Config::runtime().draft() = IRuntime {
            config: Some(config),
            exists_keys,
            chain_logs: logs,
        };

        Ok(())
    }
}

#[derive(Debug)]
pub enum ConfigType {
    Run,
    Check,
}
