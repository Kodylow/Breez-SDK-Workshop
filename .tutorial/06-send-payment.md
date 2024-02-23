## Send payment
If everything in the previous sections went well, you now have a node, with some
balance. Let's send a payment using our lightning balance.

Say we have a String variable called `invoice_to_pay`, that contains a lightning
invoice we want to pay.

```
sdk.send_payment(invoice_to_pay, None).await?;
```

## Subscribing to send payment events
Just like the events for receiving a payment, there's also events for sending a
payment you can subscribe to t get some updates on the payment status.

```rust
impl EventListener for AppEventListener {
    fn on_event(&self, e: BreezEvent) {
        match e {
            BreezEvent::InvoicePaid { details: _ } => INVOICE_PAID_NOTIFIER.notify_waiters(),
            BreezEvent::PaymentSucceed { details: _ } => println!("Payment succeeded."),
            BreezEvent::PaymentFailed { details } => println!("Payment failed: {}", details.error),
            _ => return,
        }
    }
}
```
That's it!

[Next page](./07-SPOILER.md)