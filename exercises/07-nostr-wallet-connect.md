# Exercise 7: Nostr Wallet Connect
The Breez SDK allows you to interact with the lightning network with ease. That allows you to focus on functionality, rather than spending a lot of time figuring out how to interact with the lightning network. In this exercise, you will focus on functionality around nostr.

## Introduction
Nost Wallet Connect is a protocol that allows you to 'zap' nostr users or notes, by connecting software on your lightning node to your nostr profile. In a nutshell, it works like this:
- You will have a user identity on nostr.
- Your software will have its own nostr identity and a nostr wallet connect identity.
- Your software is connected to a nostr relay, listening for events.
- Your user identity configures the nostr wallet connect secret in its profile.
- When you click a 'zap' button in a nostr client, your software will receive an encrypted event.
- Your software will decrypt the event, which contains a lightning invoice to pay.
- Your software will pay the lightning invoice
- Your software will publish an event that the invoice is paid.
- Your nostr client will see that the 'zap' succeeded.

## Implementation
Add a command `NostrWalletConnect`. It will require the environment variables `NOSTR_CLIENT_KEY` and `NWC_KEY` to be set. Make it listen for NWC requests, and when a valid one comes in, pay the associated invoice and publish a WalletConnectResponse event. In the bottom of this file you can find sample code to use nostr-rs. Make sure the program runs in a infinite loop to receive events, so it doesn't exit when you click a 'zap' button in your nostr client.

See [Sample code](#sample-code) for inspiration.

## Connecting the dots
### Configure Nostr account
- Make sure your application is running your `NostrWalletConnect` command.
- Open https://snort.social (web-based Nostr client) and create a new profile.
- Connect it to your NWC endpoint (Settings > Wallet > Nostr Wallet Connect)
- Check and adjust Profile > Settings > Preferences > Default Zap amount (default is 50 sats)
  - This is the amount that will be sent from your Breez SDK wallet when you trigger a NWC payment from snort.social.
- You can check that your snort.social profile is paired (Profile > Settings > Wallets > View Wallets > There should be a NWC entry)

### Zap a note
- Go to a profile with a LN Address setup (for example Gigi: npub1dergggklka99wwrs92yz8wdjs952h2ux2ha2ed598ngwu9w7a6fsh9xzpc)
- Find a note you like
- Click the Lightning icon underneath it (once payment completes, it should turn blue)
- Check your terminal and logs (you should see a payment attempt and its result: success or failure)
- Call list_payments to see the zap you just sent

### Notes (pun intended)
This is a plain LN payment, not a so-called zap. "Zap" is often used with extra messaging to make the payment public, but for simplicity (and privacy) we skipped that.

You finished this when you can confirm a successful payment in your logs.

Congrats! You've sent sats on Nostr from your own self-custodial Breez SDK wallet!

## Sample code
Function to get the nostr keys from the environment variables, or generate them if they don't exist:
```rust
fn get_nostr_keys() -> (Keys, Keys) {
    let nwc_key = match get_env_var("NWC_KEY") {
        Ok(key) => Keys::from_sk_str(&key).unwrap(),
        Err(_) => {
            warn!("Could not find NWC_KEY environment variable.");
            generate_nostr_keys();
            panic!();
        },
    };

    let nostr_client_key = match get_env_var("NOSTR_CLIENT_KEY") {
        Ok(key) => Keys::from_sk_str(&key).unwrap(),
        Err(_) => {
            warn!("Could not find NOSTR_CLIENT_KEY environment variable.");
            generate_nostr_keys();
            panic!();
        },
    };

    (nwc_key, nostr_client_key)
}

fn generate_nostr_keys() {
    let nwc_key = Keys::generate();
    info!("Set the 'NWC_KEY' environment variable to {}", nwc_key.secret_key().unwrap().to_bech32().unwrap());

    let nostr_client_key = Keys::generate();
    info!("Set the 'NOSTR_CLIENT_KEY' environment variable to {}", nostr_client_key.secret_key().unwrap().to_bech32().unwrap());

    let nwc_connect_uri = NostrWalletConnectURI {
        public_key: nostr_client_key.public_key(),
        secret: nwc_key.secret_key().unwrap(),
        relay_url: Url::from_str(nwc::RELAY).unwrap(),
        lud16: None,
    };
    info!("Set the NWC connection string in your nostr profile to: '{nwc_connect_uri}'");
}
```

File with a nostr wallet connect endpoint, that listens for events and pays the zap invoices (`nwc.rs`).

```rust
use std::sync::Arc;

use breez_sdk_core::{BreezServices, PaymentDetails};
use log::{error, info, warn};
use nostr_sdk::nips::nip47;
use nostr_sdk::prelude::*;

pub(crate) const RELAY: &str = "wss://nos.lol";
pub(crate) struct NostrWalletConnectEndpoint {
    nostr_client: Arc<Client>,
    breez_sdk: Arc<BreezServices>,
}

impl NostrWalletConnectEndpoint {
    pub(crate) async fn init_with(breez_sdk: Arc<BreezServices>, keys: &Keys) -> Result<Self> {
        let nostr_client = Client::new(keys);
        info!(
            "Nostr client created with pubkey {}",
            nostr_client.keys().public_key().to_bech32()?
        );
        nostr_client.add_relay(RELAY, None).await?;
        nostr_client.connect().await;

        Ok(NostrWalletConnectEndpoint {
            nostr_client: Arc::new(nostr_client),
            breez_sdk,
        })
    }

    pub(crate) async fn publish_text_note(&self, note: &str) -> Result<()> {
        let res = self.nostr_client.publish_text_note(note, &[]).await?;
        info!("Published note with ID {}", res.to_bech32()?);
        Ok(())
    }

    pub(crate) async fn listen_nwc_requests(&self, nwc_keys: &Keys) -> Result<()> {
        // Announce that this NWC endpoint supports `pay_invoice` requests
        let nwc_info_ev = EventBuilder::new(Kind::WalletConnectInfo, "pay_invoice", &[])
            .to_event(&self.nostr_client.keys())?;
        self.nostr_client.send_event(nwc_info_ev).await?;

        // Subscribe to NWC requests from Nostr apps that have the NWC connection string
        self.nostr_client
            .subscribe(vec![Filter::new().kind(Kind::WalletConnectRequest)])
            .await;

        loop {
            let event = match self.nostr_client.notifications().recv().await? {
                RelayPoolNotification::Event(_, event) => event,
                n => {
                    warn!("Unknown notification type {:?}", n);
                    continue;
                }
            };
        
            if event.kind != Kind::WalletConnectRequest {
                warn!("Unknown kind {:?}", event.kind);
                continue;
            }

            let req_encrypted = event.content;
            let req_decrypted = decrypt(
                &self.nostr_client.keys().secret_key()?,
                &nwc_keys.public_key(),
                req_encrypted,
            )?;

            let req: nip47::Request = from_str(&req_decrypted)?;
            if req.method != Method::PayInvoice {
                warn!("Unknown method {:?}", req.method);
                continue;
            }

            let params = match req.params {
                RequestParams::PayInvoice(params) => params,
                _ => {
                    warn!("got unknown req.params: {:?}", req.params);
                    continue;
                }
            };
            info!("Got NWC request to pay invoice {}", params.invoice);

            let payment = self.breez_sdk.send_payment(params.invoice, None).await?;
            let details = match payment.details {
                PaymentDetails::Ln { data } => data,
                _ => {
                    error!("got unknown payment details");
                    continue;
                }
            };

            info!("Payment successful, sending NWC response to confirm");

            let resp_encrypted = encrypt(
                &self.nostr_client.keys().secret_key()?,
                &nwc_keys.public_key(),
                nip47::Response {
                    result_type: Method::PayInvoice,
                    result: Some(ResponseResult::PayInvoice(
                        PayInvoiceResponseResult {
                            preimage: details.payment_preimage,
                        },
                    )),
                    error: None,
                }.as_json(),
            )?;

            let resp_event = EventBuilder::new(
                Kind::WalletConnectResponse,
                resp_encrypted,
                &[
                    Tag::Event(event.id, None, None),
                    Tag::PubKey(nwc_keys.public_key(), None),
                ],
            ).to_event(&self.nostr_client.keys())?;
            self.nostr_client.send_event(resp_event).await?;
            info!("Sent NWC response");
        }
    }
}
```

## Done?
That's it! Congratulations, you've completed all workshop exercises. Legend.
