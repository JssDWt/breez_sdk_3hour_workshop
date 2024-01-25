use std::{env, str::FromStr, sync::Arc};
mod nwc;

use bip39::{Language, Mnemonic};
use breez_sdk_core::{
    parse, BreezEvent, BreezServices, EnvironmentType, EventListener, GreenlightNodeConfig,
    ListPaymentsRequest, LnUrlWithdrawRequest, LnUrlWithdrawResult, NodeConfig,
    ReceivePaymentRequest, SendPaymentRequest,
};
use clap::{Parser, Subcommand};
use dotenv::dotenv;
use log::{error, info, warn};
use nostr_sdk::{
    prelude::{FromSkStr, NostrWalletConnectURI, ToBech32},
    Keys, Url,
};
use serde::Serialize;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let cli = Cli::parse();
    stderrlog::new()
        .show_level(false)
        .modules(vec![
            "breez_sdk_3hour_workshop",
            "breez_sdk_core",
            "gl_client",
            "lightning_signer",
        ])
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
        Commands::LnurlWithdraw { lnurl } => {
            let sdk = connect().await;
            let input = match parse(lnurl).await {
                Ok(input) => match input {
                    breez_sdk_core::InputType::LnUrlWithdraw { data } => data,
                    _ => {
                        error!("Invalid input");
                        return;
                    }
                },
                Err(e) => {
                    error!("Invalid input: {}", e);
                    return;
                }
            };

            let amount_msat = input.max_withdrawable;
            let result = sdk
                .lnurl_withdraw(LnUrlWithdrawRequest {
                    data: input,
                    amount_msat,
                    description: Some(String::from("collecting some funds to play with Breez SDK")),
                })
                .await
                .unwrap();
            match result {
                LnUrlWithdrawResult::Ok { data: _ } => {
                    info!("Success!");
                }
                LnUrlWithdrawResult::ErrorStatus { data } => {
                    error!("Error: {}", data.reason)
                }
            }
        }
        Commands::SendPayment { invoice } => {
            let sdk = connect().await;
            let input = match parse(invoice).await {
                Ok(input) => match input {
                    breez_sdk_core::InputType::Bolt11 { invoice } => invoice,
                    _ => {
                        error!("Invalid input");
                        return;
                    }
                },
                Err(e) => {
                    error!("Invalid input: {}", e);
                    return;
                }
            };

            let resp = sdk
                .send_payment(SendPaymentRequest {
                    bolt11: input.bolt11,
                    amount_msat: None,
                })
                .await
                .unwrap();
            info!("Success. Fee paid (msat): {}", resp.payment.fee_msat);
        }
        Commands::ListPayments => {
            let sdk = connect().await;
            let payments = sdk
                .list_payments(ListPaymentsRequest::default())
                .await
                .unwrap();
            for payment in payments.iter() {
                info!(
                    "- type: {:?}\n  description: {:?}\n  amount_msat: {}\n  fee_msat: {}",
                    payment.payment_type,
                    payment.description,
                    payment.amount_msat,
                    payment.fee_msat
                );
            }
        }
        Commands::ChatGpt { prompt } => {
            let sdk = connect().await;
            let url = "http://localhost:8000/openai/v1/chat/completions";
            info!("Calling http 402 API without a token.");

            let client = reqwest::ClientBuilder::new().build().unwrap();
            let req = &GptRequest {
                model: String::from("gpt-3.5-turbo"),
                messages: vec![GptMessage {
                    role: String::from("user"),
                    content: prompt.clone(),
                }],
            };
            let mut resp = client.post(url).json(&req).send().await.unwrap();
            info!("Response status is {}", resp.status());
            if resp.status() == 402 {
                let l402header = resp
                    .headers()
                    .get("WWW-Authenticate")
                    .expect("server did not return WWW-Authenticate header in 402 response.")
                    .to_str()
                    .unwrap();

                info!("Got WWW-Authenticate header: {}", l402header);
                let re = regex::Regex::new(
                    r#"^L402 (token|macaroon)=\"(?<token>.*)\", invoice=\"(?<invoice>.*)\""#,
                )
                .unwrap();
                let caps = re
                    .captures(l402header)
                    .expect("WWW-Authenticate header is not a valid L402");
                let token = caps["token"].to_string();
                let invoice = caps["invoice"].to_string();

                info!(
                    "Paying lightning invoice to get access to the API: {}",
                    invoice
                );
                let payresp = sdk
                    .send_payment(SendPaymentRequest {
                        bolt11: invoice,
                        amount_msat: None,
                    })
                    .await
                    .unwrap();
                let lnpayresult = match payresp.payment.details {
                    breez_sdk_core::PaymentDetails::Ln { data } => data,
                    _ => unreachable!(),
                };

                let header = format!("L402 {}:{}", token, lnpayresult.payment_preimage);
                info!(
                    "Calling http 402 api again, now with header Authorization {}",
                    header
                );
                resp = client
                    .post(url)
                    .header("Authorization", header)
                    .json(&req)
                    .send()
                    .await
                    .unwrap();
            }

            let status = resp.status();
            info!("Got Response. Status {}", status);
            let text = resp.text().await.unwrap();
            info!("{}", text);
        }
        Commands::NostrWalletConnect => {
            let (nwc_key, nostr_client_key) = get_nostr_keys();
            let sdk = connect().await;
            let nwc = nwc::NostrWalletConnectEndpoint::init_with(sdk, nostr_client_key)
                .await
                .unwrap();
            nwc.publish_text_note("Hello world").await.unwrap();
            nwc.listen_nwc_requests(&nwc_key).await.unwrap();
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
    #[clap(alias = "withdraw")]
    LnurlWithdraw {
        #[clap(long, short)]
        lnurl: String,
    },
    #[clap(alias = "send")]
    SendPayment {
        #[clap(long, short)]
        invoice: String,
    },
    #[clap(alias = "list")]
    ListPayments,
    #[clap(alias = "gpt")]
    ChatGpt {
        #[clap(long, short)]
        prompt: String,
    },
    #[clap(alias = "nwc")]
    NostrWalletConnect,
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

#[derive(Serialize)]
pub struct GptRequest {
    pub model: String,
    pub messages: Vec<GptMessage>,
}

#[derive(Serialize)]
pub struct GptMessage {
    pub role: String,
    pub content: String,
}

fn get_nostr_keys() -> (Keys, Keys) {
    let nwc_key = match get_env_var("NWC_KEY") {
        Ok(key) => Keys::from_sk_str(&key).unwrap(),
        Err(_) => {
            warn!("Could not find NWC_KEY environment variable.");
            generate_nostr_keys();
            panic!();
        }
    };

    let nostr_client_key = match get_env_var("NOSTR_CLIENT_KEY") {
        Ok(key) => Keys::from_sk_str(&key).unwrap(),
        Err(_) => {
            warn!("Could not find NOSTR_CLIENT_KEY environment variable.");
            generate_nostr_keys();
            panic!();
        }
    };

    (nwc_key, nostr_client_key)
}

fn generate_nostr_keys() {
    let nwc_key = Keys::generate();
    info!(
        "Set the 'NWC_KEY' environment variable to {}",
        nwc_key.secret_key().unwrap().to_bech32().unwrap()
    );

    let nostr_client_key = Keys::generate();
    info!(
        "Set the 'NOSTR_CLIENT_KEY' environment variable to {}",
        nostr_client_key.secret_key().unwrap().to_bech32().unwrap()
    );

    let nwc_connect_uri = NostrWalletConnectURI {
        public_key: nostr_client_key.public_key(),
        secret: nwc_key.secret_key().unwrap(),
        relay_url: Url::from_str(nwc::RELAY).unwrap(),
        lud16: None,
    };
    info!("Set the NWC connection string in your nostr profile to: '{nwc_connect_uri}'");
}
