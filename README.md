# Breez SDK 3 hour workshop
This workshop walks through the main functionality of the Breez SDK. 

- [x] Create/connect to a node
- [x] Receive a payment
- [x] lnurl withdraw
- [x] send a payment

Next to the basic functionality, there's two examples of how to use the SDK in a functional context.

- [x] Paying for api calls with the [L402 protocol](https://github.com/lightning/blips/blob/d2a8c19ec6f49677d942d1c03f3ab0a3362e7b39/blip-0026.md)
- [x] Paying for zaps with Nostr Wallet Connect ([NIP-47: NWC](https://github.com/nostr-protocol/nips/blob/master/47.md))

## Prerequisites
### Rust
Make sure you have Rust installed on your system. Find instructions to install Rust [here](https://www.rust-lang.org/tools/install). For unix-like systems the easiest to install rust is:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Getting started
To get started, go to the first [exercise](./exercises/00-introduction.md).