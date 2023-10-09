# Exercise 3: Receive a payment
Normally, if you would run your own lightning node, you would need to manage your lightning channels and inbound and outbound liquidity. With the Breez SDK, liquidity comes out of the box through a Lightning Service Provider (LSP). 

In this exercise, we will receive a payment. On the first payment you receive, the LSP will open a just-in-time channel to your personal lightning node in order to provide you with inbound liquidity to be able to receive the payment. The LSP connects you to the lightning network, similar to how an ISP connects you to the internet. The channel opening will cost a fee. Subsequent receives will be free.

## Implementation
- Add a command `ReceivePayment` to receive a payment with a specific amount in satoshis and a description.
- Look at the [documentation](https://sdk-doc.breez.technology/guide/payments.html#receiving-lightning-payments) how to receive a payment.
- Note that the `receive_payment` function will create a lightning invoice, but does not receive the payment yet. In order to receive a payment on the lightning network through the Breez SDK, your _signer_ needs to be running. So make sure the program doesn't exit until the invoice is paid. An easy way to do that is to wait for the user to press the `<enter>` key.
- Make sure to print the LSP opening fee if a fee is necessary to open a channel.
- Make sure the `EventListener` prints to the console when the invoice was paid. Subscribe to the `InvoicePaid` event.
- Pay the invoice with another lightning node.

## Done?
Go to the [next exercise](./04-lnurl-withdraw.md).
