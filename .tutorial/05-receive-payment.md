## Receive a payment
Receiving a payment is a single function call. This function call creates a
lightning invoice, the user can share to get paid. As long as the Breez SDK is
started, the user will be able to receive this payment. If the Breez SDK is not
started, for example because the app has exited, the lightning node will not
be able to sign messages in order to reeceive the payment.

```rust
let invoice = sdk.receive_payment(
    10000, 
    "My first Breez SDK invoice".to_string(),
).await?;
```

## Waiting for the invoice to be paid
You might want to wait for the invoice to be paid, or show a notification when
the invoice gets paid. Let's modify the event listener to give an update when
the invoice was paid. Here's a hacky version for a single invoice. The
`INVOICE_PAID_NOTIFIER` is an object that allows a caller to block until a piece
of code calls `notify_waiters`. 

```rust
static INVOICE_PAID_NOTIFIER: Lazy<Notify> = Lazy::new(|| Notify::new());
struct AppEventListener {}
impl EventListener for AppEventListener {
    fn on_event(&self, e: BreezEvent) {
        match e {
            BreezEvent::InvoicePaid { details: _ } => INVOICE_PAID_NOTIFIER.notify_waiters(),
            _ => return,
        }
    }
}
```

Then we can wait for the invoice to be paid like so.

```rust
INVOICE_PAID_NOTIFIER.notified().await;
```

## Explanation
What happens when the user creates an invoice? The first time you create an 
invoice, the user's node will not be connected to the lightning network yet, 
through lightning channels. Normally, the user would need to acquire some
inbound liquidity through a lightning channel in order to be able to receive the
payment.

With Breez SDK, liquidity comes out of the box. When the user creates the
invoice, it will contain a hint that the payment should be routed over the LSP.
So the LSP will be an intermediary hop in the payment route over the lightning
network. The LSP opens a lightning channel to the user's node on-the-fly when 
forwarding the payment to the user. That way the user will get inbound liquidity
the moment it gets paid. 

This channel open requires an onchain transaction, and the LSP will charge a fee
that covers both the opening of the channel and the future closing of the
channel. This fee is deducted from the amount the user receives. So let's say
the user creates an invoice for 12,000 sats and the LSP fee is 5,000 sats, that
means the user will receive 7,000 sats in their balance.

This lightning channel is a zero-conf channel, so the funding transaction hasn't
been confirmed  onchain yet. When the channel confirms, there will be no trust
between the client and the LSP. The user can send and receive payments and
doesn't need to touch the chain again. SO for subsequent payments, no LSP fee
is charged other than regular lightning routing fees.

[Next page](./06-send-payment.md)