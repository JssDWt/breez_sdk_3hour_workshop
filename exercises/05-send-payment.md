# Exercise 5: Send a payment
In this exercise, you will pay an invoice of another lightning node. You can either pay one of the other workshop participants, or pay an invoice of another lightning node you own.

## Implementation
- Add a command `SendPayment` with an `invoice` parameter.
- Look at the [documentation](https://sdk-doc.breez.technology/guide/payments.html#sending-lightning-payments) how to send the payment.
- `None` can be passed for the `amount` parameter. The amount parameter is only used for amountless invoices. In this case we will pay an invoice with an amount.

## Done?
Go to the [next exercise](./06-list-payments.md).
