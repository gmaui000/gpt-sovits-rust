use clap::Parser;
use std::{path::PathBuf, process::exit};
use tracing::{self, error, info};
use tts_server::base::*;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, value_name = "CONFIG_FILE_PATH")]
    config: PathBuf,
}

pub fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if !args.config.is_file() {
        error!("config file is not existed: {}", args.config.display());
        exit(1);
    }

    let config_data = configuration::decode_config(&args.config)?;
    let _guard = trace::init(&config_data);

    info!("tts_server start with config: {:#?}", config_data);

    if let Some(err) = tts_server::tts::server::start(&config_data).err() {
        error!("error: {}", err);
    }

    anyhow::Ok(())
}
