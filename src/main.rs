use bip39::{Language, Mnemonic};
use breez_sdk_core::{
    BreezEvent, BreezServices, EnvironmentType, EventListener, GreenlightNodeConfig, NodeConfig,
    ReceivePaymentRequest,
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
        Commands::NodeInfo => {
            let sdk = connect().await;
            let info = sdk.node_info().unwrap();
            info!("{:?}", info);
        }
        Commands::ReceivePayment {
            amount_sats,
            description,
        } => {
            let sdk = connect().await;
            let invoice = sdk
                .receive_payment(ReceivePaymentRequest {
                    amount_msat: *amount_sats * 1000,
                    description: description.clone(),
                    cltv: None,
                    expiry: None,
                    opening_fee_params: None,
                    preimage: None,
                    use_description_hash: None,
                })
                .await
                .unwrap();
            info!(
                "Invoice: {}, expected opening fee (msat): {:?}",
                invoice.ln_invoice.bolt11, invoice.opening_fee_msat
            );
            info!("Waiting for invoice to be paid. Press <enter> to exit.");
            let mut s = Default::default();
            std::io::stdin().read_line(&mut s).unwrap();
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
    #[clap(alias = "info")]
    NodeInfo,
    #[clap(alias = "receive")]
    ReceivePayment {
        #[clap(long, short)]
        amount_sats: u64,
        #[clap(long, short)]
        description: String,
    },
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
            BreezEvent::InvoicePaid { details } => {
                info!("invoice got paid: {}", details.bolt11)
            }
            _ => (),
        }
    }
}
