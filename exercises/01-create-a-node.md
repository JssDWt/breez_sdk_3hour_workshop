# Exercise 1: Create a node
You have already configured a breez api key, greenlight invite code and mnemonic in your `.env` file in the introduction. If you haven't, make sure you follow the steps in the [introduction](./00-introduction.md) first.

In this exercise, you will create a new personal lightning node on the Greenlight infrastructure. Your code will either/or
- create a new node in Greenlight
- connect to an existing node in Greenlight

## Implementation
- Create a `connect` function that is reusable for different commands we will create later. The connect function should connect to your Greenlight node and return the `BreezServices` object.
- Look at the [Getting Started](https://sdk-doc.breez.technology/guide/getting_started.html#connecting) documentation how to connect to your node.
- The working directory can be left empty, because the current directory is fine.
- Make sure you set `config.exemptfee_msat` to a reasonably large amount for this workshop, because we'll be paying very small amounts. A number that will surely always work is `config.exemptfee_msat = 50000;`. The exemptfee skips the maximum fee check when the fee is lower than the set amount. That helps in our low amount payments scenarios.

## Done?
Go to the [next exercise](./02-fetch-node-info.md).
