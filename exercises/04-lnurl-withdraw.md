# Exercise 4: Lnurl withdraw
In the previous exercise, you created a function that receives a payment by providing the sender with a bolt11 invoice. Another way to receive a payment is through the [lnurl withdraw protocol](https://github.com/lnurl/luds/blob/luds/03.md). This allows you to input a url to withdraw funds from and receive a payment over lightning that way. It's basically a lightning payment that is initiated by the recipient, rather than the sender.

## Implementation
- Add a command `LnurlWithdraw` with a `lnurl` parameter. The lnurl is the link to a faucet that you will call to receive a payment to your node. 
- Use the input parser (`parse` function exposed in the Breez SDK) to check the validity of the lnrul withdraw link.
- Invoke the `lnurl_withdraw` function according to the [documentation](https://sdk-doc.breez.technology/guide/lnurl_withdraw.html). Make sure the amount to withdraw is within `min_withdrawable` and `max_withdrawable`.
- Make sure the program doesn't exit before the invoice is paid, just like in the `receive_payment` flow in the previous exercise.

## Done?
Go to the [next exercise](./05-send-payment.md).
