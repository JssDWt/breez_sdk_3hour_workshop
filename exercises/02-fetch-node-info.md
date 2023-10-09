# Exercise 2: Fetch node info
In this exercise we will fetch the node information from your personal lightning node. The information will contain for example your node's identity, and also information about the state of your node, like inbound and outbound liquidity.

## Implementation
Add a `NodeInfo` command to the cli that will:
- connect to the node
- fetch the node information with `sdk.node_info()`
- prints the node id and the spendable amount

## Done?
Go to the [next exercise](./03-receive-payment.md).
