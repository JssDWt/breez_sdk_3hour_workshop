# Exercise 7: L402 client
The Breez SDK allows you to interact with the lightning network with ease. That allows you to focus on functionality, rather than spending a lot of time figuring out how to interact with the lightning network. In this exercise, you will focus on functionality around paid apis.

The [L402 protocol](https://github.com/lightning/blips/blob/d2a8c19ec6f49677d942d1c03f3ab0a3362e7b39/blip-0026.md) is a protocol around the HTTP status code 402 (payment required). A HTTP API can be wrapped with logic to require payments to complete API calls. 

If payment is required, the server will return HTTP status code 402 , together with a HTTP header `WWW-Authenticate: L402 macaroon="AGIAJEemVQUTEyNCR0exk7ek90Cg==", invoice="lnbc1..."`. In order to get access to the endpoint, you pay the invoice `P`. The payment result will give you a preimage. The preimage is sent along with a retry of the request in a HTTP header `Authorization: L402 AGIAJEemVQUTEyNCR0exk7ek90Cg==:1234abcd1234abcd1234abcd`. Where `1234abcd1234abcd1234abcd` is the preimage to the lightning payment. In lightning, the preimage is your proof of payment. This is how you can prove to the server that you paid the invoice.

In this exercise, you will call a HTTP api that wraps chatgpt. The api will accept lightning payments for access, rather than a monthly subscription.

## Implementation
Add a command `ChatGpt` to your application, with a `prompt` parameter. Send the prompt to the chatgpt api, and make sure you have access to the api by paying an invoice if required.

To get you setup with the api and the L402 protocol, do the following.
Add two models to your code:
```rust
#[derive(Serialize)]
pub struct GptRequest {
    pub model: String,
    pub messages: Vec<GptMessage>
}

#[derive(Serialize)]
pub struct GptMessage {
    pub role: String,
    pub content: String
}
```

Call the chatgpt wrapper like so:
```rust
let url = "http://localhost:8000/openai/v1/chat/completions";
info!("Calling http 402 API without a token.");

let client = reqwest::ClientBuilder::new().build().unwrap();
let req = &GptRequest{
    model: String::from("gpt-3.5-turbo"),
    messages: vec![
        GptMessage{
            role: String::from("user"),
            content: prompt.clone()
        }
    ]
};
let mut resp = client.post(url).json(&req).send()
    .await
    .unwrap();
info!("Response status is {}", resp.status());
```

If the response status is 402, pay the invoice from the WWW-Authenticate header and call the api again with the `Authenticate` header using the acquired preimage. Here's sample code to extract the invoice and macaroon from the http response:
```rust
let l402header = resp.headers()
    .get("WWW-Authenticate")
    .expect("server did not return WWW-Authenticate header in 402 response.")
    .to_str()
    .unwrap();

info!("Got WWW-Authenticate header: {}", l402header);
let re = regex::Regex::new(r#"^L402 (token|macaroon)=\"(?<token>.*)\", invoice=\"(?<invoice>.*)\""#).unwrap();
let caps = re.captures(l402header).expect("WWW-Authenticate header is not a valid L402");
let token = caps["token"].to_string();
let invoice = caps["invoice"].to_string();
info!("Got lightning invoice to get access to the API: {}", invoice);
```

## Done?
Go to the [next exercise](./08-nostr-wallet-connect.md).
