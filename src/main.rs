use bip39::{Language, Mnemonic};
use breez_sdk_core::{
    BreezEvent, BreezServices, EnvironmentType, EventListener, GreenlightNodeConfig, NodeConfig,
};
use clap::{Parser, Subcommand};
use dotenv::dotenv;
use log::info;
use std::{env, sync::Arc};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let cli = Cli::parse();
    stderrlog::new()
        .show_level(false)
        .modules(vec!["breez_sdk_3hour_workshop", "breez_sdk_core"])
        .verbosity(match cli.verbose {
            true => stderrlog::LogLevelNum::Debug,
            false => stderrlog::LogLevelNum::Info,
        })
        .init()
        .unwrap();
    match &cli.command {
        Commands::GenerateMnemonic => {
            let mnemonic = Mnemonic::generate_in(Language::English, 12).unwrap();
            info!("Generated mnemonic: {mnemonic}");
            info!("Set the environment variable 'MNEMONIC', and run another command.");
        }
    };
}

#[derive(Parser)]
#[command(name = "breez-sdk-demo")]
#[command(author = "Jesse de Wit <witdejesse@hotmail.com>")]
#[command(version = "0.1")]
#[command(about = "Example commandline application for the Breez SDK")]
#[command(long_about = None)]
struct Cli {
    #[arg(short, long, action)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(alias = "mnemonic")]
    GenerateMnemonic,
}

fn get_env_var(name: &str) -> Result<String, String> {
    let v = match env::var(name) {
        Ok(v) => v,
        Err(_) => return Err("variable not set".to_string()),
    };

    if v.is_empty() {
        return Err("variable is empty".to_string());
    }

    Ok(v)
}

async fn connect() -> Arc<BreezServices> {
    let breez_sdk_api_key =
        get_env_var("BREEZ_API_KEY").expect("set the 'BREEZ_API_KEY' environment variable");
    let greenlight_invite_code = get_env_var("GREENLIGHT_INVITE_CODE")
        .expect("set the 'GREENLIGHT_INVITE_CODE' environment variable");
    let phrase = get_env_var("MNEMONIC").expect("set the 'MNEMONIC' environment variable");

    let mnemonic = Mnemonic::parse(phrase).unwrap();
    let seed = mnemonic.to_seed("");

    let mut config = BreezServices::default_config(
        EnvironmentType::Production,
        breez_sdk_api_key,
        NodeConfig::Greenlight {
            config: GreenlightNodeConfig {
                invite_code: Some(greenlight_invite_code),
                partner_credentials: None,
            },
        },
    );
    config.exemptfee_msat = 50000;

    let sdk = BreezServices::connect(config, seed.to_vec(), Box::new(AppEventListener {}))
        .await
        .unwrap();
    sdk
}

struct AppEventListener {}
impl EventListener for AppEventListener {
    fn on_event(&self, e: BreezEvent) {
        match e {
            _ => return,
        }
    }
}
