use super::super::error::AppError;
use super::trace::*;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AppConfigItem {
    pub log_path: Option<String>,
    pub ip: String,
    pub port: u16,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Config {
    Config(AppConfigItem),
}

pub fn decode_config(config_file: &PathBuf) -> anyhow::Result<AppConfigItem> {
    let config_data = decode_config_data(config_file)?;
    if let Some(config_info) = config_data {
        let log_path = if let Some(log_path) = config_info.log_path.clone() {
            if log_path.eq("default") {
                get_default_log_path()?
            } else {
                let log_path = PathBuf::from(log_path);
                if !log_path.exists() {
                    match std::fs::create_dir(&log_path) {
                        Ok(_) => {}
                        Err(_e) => {
                            panic!("create logs path with err: {} {}", &log_path.display(), _e);
                        }
                    }
                }
                log_path
            }
        } else {
            get_default_log_path()?
        };

        let mut new_config_info = config_info.clone();
        new_config_info.log_path = Some(log_path.display().to_string());
        anyhow::Ok(new_config_info)
    } else {
        anyhow::bail!(AppError::ConfigFileLost);
    }
}

#[allow(unused_variables)]
fn decode_config_data(config_file: &PathBuf) -> anyhow::Result<Option<AppConfigItem>> {
    let abs_file = std::fs::canonicalize(config_file).expect("can't canonicalize file");
    let file = std::fs::File::open(abs_file).expect("unable to open file");
    let data: Vec<Config> = serde_yaml::from_reader(file)?;

    if let Some(config) = data.first() {
        match config {
            Config::Config(config) => return anyhow::Ok(Some(config.clone())),
        }
    }

    anyhow::Ok(None)
}
