## Initialize SDK
Now we will initialize the SDK. That will connect to the user's lightning node
and start the signer and background services.

### Config
Construct a config object, containing the variables you want. One of them is the
Breez API key, to use the LSP services.

```rust
let mut config = BreezServices::default_config(EnvironmentType::Production);
config.api_key = Some(breez_sdk_api_key);
```

### Initialize and start
```rust
let sdk = BreezServices::init_services(
    config, 
    seed.to_vec(), 
    greenlight_credentials, 
    Box::new(AppEventListener {})
).await?;
BreezServices::start(&sdk).await?;
```

## Eventlistener
Note that an eventlistener is passed, which you can use to monitor events, like
when the node is synced to chain, when a payment was received or sent. A basic
eventlistener that logs event to the console looks like this.

```rust
struct AppEventListener {}
impl EventListener for AppEventListener {
    fn on_event(&self, e: BreezEvent) {
        println!("Received Breez event: {:?}", e);
    }
}
```

Now the user has a node and everything is in place to receive a payment!

[Next page](./05-receive-payment.md)