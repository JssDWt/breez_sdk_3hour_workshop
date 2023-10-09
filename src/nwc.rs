use std::sync::Arc;

use breez_sdk_core::{BreezServices, PaymentDetails};
use log::{error, info, warn};
use nostr_sdk::nips::nip47;
use nostr_sdk::prelude::*;

pub(crate) const RELAY: &str = "wss://nos.lol";
pub(crate) struct NostrWalletConnectEndpoint {
    keys: Keys,
    nostr_client: Arc<Client>,
    breez_sdk: Arc<BreezServices>,
}

impl NostrWalletConnectEndpoint {
    pub(crate) async fn init_with(breez_sdk: Arc<BreezServices>, keys: Keys) -> Result<Self> {
        let nostr_client = Client::new(&keys);
        info!(
            "Nostr client created with pubkey {}",
            keys.public_key().to_bech32()?
        );
        nostr_client.add_relay(RELAY).await?;
        nostr_client.connect().await;

        Ok(NostrWalletConnectEndpoint {
            nostr_client: Arc::new(nostr_client),
            breez_sdk,
            keys,
        })
    }

    pub(crate) async fn publish_text_note(&self, note: &str) -> Result<()> {
        let res = self.nostr_client.publish_text_note(note, []).await?;
        info!("Published note with ID {}", res.to_bech32()?);
        Ok(())
    }

    pub(crate) async fn listen_nwc_requests(&self, nwc_keys: &Keys) -> Result<()> {
        // Announce that this NWC endpoint supports `pay_invoice` requests
        let nwc_info_ev =
            EventBuilder::new(Kind::WalletConnectInfo, "pay_invoice", []).to_event(&self.keys)?;
        self.nostr_client.send_event(nwc_info_ev).await?;

        // Subscribe to NWC requests from Nostr apps that have the NWC connection string
        self.nostr_client
            .subscribe(vec![Filter::new().kind(Kind::WalletConnectRequest)])
            .await;

        loop {
            let event = match self.nostr_client.notifications().recv().await? {
                RelayPoolNotification::Event { event, .. } => event,
                n => {
                    warn!("Unknown notification type {:?}", n);
                    continue;
                }
            };

            if event.kind != Kind::WalletConnectRequest {
                warn!("Unknown kind {:?}", event.kind);
                continue;
            }

            let req_encrypted = event.content.clone();
            let req_decrypted = nostr_sdk::nips::nip04::decrypt(
                &self.keys.secret_key()?,
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

            let payment_response = self
                .breez_sdk
                .send_payment(breez_sdk_core::SendPaymentRequest {
                    bolt11: params.invoice,
                    amount_msat: None,
                })
                .await?;
            let details = match payment_response.payment.details {
                PaymentDetails::Ln { data } => data,
                _ => {
                    error!("got unknown payment details");
                    continue;
                }
            };

            info!("Payment successful, sending NWC response to confirm");

            let resp_encrypted = nostr_sdk::nips::nip04::encrypt(
                &self.keys.secret_key()?,
                &nwc_keys.public_key(),
                nip47::Response {
                    result_type: Method::PayInvoice,
                    result: Some(ResponseResult::PayInvoice(PayInvoiceResponseResult {
                        preimage: details.payment_preimage,
                    })),
                    error: None,
                }
                .as_json(),
            )?;

            let resp_event = EventBuilder::new(
                Kind::WalletConnectResponse,
                resp_encrypted,
                [
                    Tag::Event {
                        event_id: event.id,
                        relay_url: None,
                        marker: None,
                    },
                    Tag::PublicKey {
                        public_key: nwc_keys.public_key(),
                        relay_url: None,
                        alias: None,
                        uppercase: false,
                    },
                ],
            )
            .to_event(&self.keys)?;
            self.nostr_client.send_event(resp_event).await?;
            info!("Sent NWC response");
        }
    }
}
