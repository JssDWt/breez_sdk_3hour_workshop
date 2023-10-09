# Introduction
This workshop will create a simple commandline application in Rust, using the Breez SDK. In the next exercises, you will walk through the core functionality of the SDK, and if there's time, implement some cool features.

## Setup
Clone this repository onto your local machine and move into the repo directory.
```bash
git clone https://github.com/JssDWt/breez_sdk_3hour_workshop
cd breez_sdk_3hour_workshop
```

Copy `.env.example` to `.env`
```bash
cp .env.example .env
```

In the `.env` file, fill in your Greenlight invite code and Breez API key into the `GREENLIGHT_INVITE_CODE` and `BREEZ_API_KEY` variables respectively.

Compile the main branch. This should give no errors. There will be a warning, but that's fine.
```bash
cargo build
```

## Mnemonic
With Breez SDK, every user has their own personal lightning node in the cloud, hosted by the Blockstream Greenlight infrastructure. The user holds their own keys, they never leave the user's device. The code to generate a mnemonic for the user is already available in the starter code. 

Run the mnemonic command to generate a mnemonic of your own. Put the resulting mnemonic in the `MNEMONIC` variable in the `.env` file.

Note: The `--` is there to separate the parameters for 
```bash
cargo run -- mnemonic
```

If someone else has this mnemonic, they can access your funds on your lightning node. Normally it's a good idea to persist the mnemonic in secure storage. Here, for this example, the mnemonic is simply put in an environment variable.

## Documentation / solutions
The Breez SDK methods are documented [here](https://sdk-doc.breez.technology/). 

You can find 'a' solution for every exercise in the `solution` branch. Every exercise has a commit in the `solution` branch, so you can look at the diff to see how to solve the exercise you're working on.

Or checkout the `solution` branch to see full solutions for all exercises at once.
```bash
git checkout solution
```

## Done?
Go to the [next exercise](./01-create-a-node.md).
