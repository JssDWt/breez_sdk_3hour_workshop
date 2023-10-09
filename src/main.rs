use std::env;
use bip39::{Mnemonic, Language};
use clap::{Parser, Subcommand};
use dotenv::dotenv;
use log::info;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let cli = Cli::parse();
    stderrlog::new()
        .show_level(false)
        .modules(vec![
            "breez_sdk_3hour_workshop", 
            "breez_sdk_core",
        ])
        .verbosity(match cli.verbose {
            true => stderrlog::LogLevelNum::Debug,
            false => stderrlog::LogLevelNum::Info
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
