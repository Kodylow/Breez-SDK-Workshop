## Spoiler (solution)
We've initialized a lightning node, connected to an existing lightning node, 
received a payment and sent a payment. We can put this together in a console
application.

You can run this application if you have all the environment variables set. The
idea is you can run it multiple times, and the app checks whether you have an
existing node or a new node.

Note as a little extra the use of the input_parser to parse user input to check
whether this is actually an invoice we're getting as input.

```rust
use anyhow::Result;
use bip39::{Language, Mnemonic};
use breez_sdk_core::{BreezEvent, BreezServices, EnvironmentType, EventListener, Network, input_parser::parse};
use dotenv::dotenv;
use once_cell::sync::Lazy;
use tokio::sync::Notify;
use std::{io, env};

static INVOICE_PAID_NOTIFIER: Lazy<Notify> = Lazy::new(|| Notify::new());
struct AppEventListener {}
impl EventListener for AppEventListener {
    fn on_event(&self, e: BreezEvent) {
        match e {
            BreezEvent::InvoicePaid { details: _ } => INVOICE_PAID_NOTIFIER.notify_waiters(),
            // BreezEvent::NewBlock { block } => todo!(),
            // BreezEvent::Synced => todo!(),
            BreezEvent::PaymentSucceed { details: _ } => println!("Payment succeeded."),
            BreezEvent::PaymentFailed { details } => println!("Payment failed: {}", details.error),
            // BreezEvent::BackupStarted => todo!(),
            // BreezEvent::BackupSucceeded => todo!(),
            // BreezEvent::BackupFailed { details } => todo!(),
            _ => return,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let breez_sdk_api_key = get_env_var("BREEZ_API_KEY");
    let greenlight_invite_code = get_env_var("GREENLIGHT_INVITE_CODE");

    let mnemonic = match env::var("MNEMONIC") {
        Ok(phrase) => {
            Mnemonic::parse(phrase)?
        },
        Err(_) => {
            let mnemonic = Mnemonic::generate_in(Language::English, 12)?;
            println!("Generated mnemonic: {mnemonic}");
            println!("Set the environment variable 'MNEMONIC', and run again.");
            return Ok(());
        }
    };
    let seed = mnemonic.to_seed("");

    println!("Checking whether user has an existing node.");
    let (greenlight_credentials, is_new_node) = match BreezServices::recover_node(Network::Bitcoin, seed.to_vec()).await {
        Ok(c) => {
            println!("User has existing node.");
            (c, false)
        },
        Err(_) => {
            println!("No existing node yet, creating a new Greenlight node.");
            (BreezServices::register_node(
                Network::Bitcoin, 
                seed.to_vec(), 
                None, 
                Some(greenlight_invite_code)
            ).await?, true)
        },
    };

    println!("Starting the Breez SDK.");
    let mut config = BreezServices::default_config(EnvironmentType::Production);
    config.api_key = Some(breez_sdk_api_key);
    let sdk = BreezServices::init_services(config, seed.to_vec(), greenlight_credentials, Box::new(AppEventListener {})).await?;
    BreezServices::start(&sdk).await?;

    println!("Fetching node info.");
    let node_info = sdk.node_info()?.expect("Expected node_info to not be None.");
    println!("Node id: {}, balance msat {}", node_info.id, node_info.max_payable_msat);

    if is_new_node || node_info.max_payable_msat == 0 {
        let invoice = sdk.receive_payment(10000, "My first Breez SDK invoice".to_string()).await?;
        println!("Created invoice: {}", invoice.bolt11);

        println!("Waiting for the invoice to be paid... (pay it with another lightning app)");
        INVOICE_PAID_NOTIFIER.notified().await;
        println!("Invoice got paid!");
    }
    
    loop {
        println!("Paste an invoice to pay in the console and press Enter.");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("error: unable to read user input");
        input = input.strip_suffix("\r\n")
            .or(input.strip_suffix("\n"))
            .unwrap_or(&input)
            .to_string();

        match parse(input.as_str()).await {
            Ok(parsed) => match parsed {
                breez_sdk_core::InputType::Bolt11 { invoice } => {
                    println!("Sending payment for invoice {}", invoice.bolt11);
                    sdk.send_payment(invoice.bolt11, None).await?;
                },
                _ => println!("Input is not a bolt11 invoice")
            },
            Err(e) => println!("Input is not a bolt11 invoice: {}", e)
        }
    }
}

fn get_env_var(name: &str) -> String {
    match env::var(name) {
        Ok(c) => match c.as_str() {
         "" => Err(format!("'{}' is empty", name)),
         _ => Ok(c),
        },
        _ => Err(format!("'{}' is not set", name))
    }.expect(format!("set the '{}' environment variable", name).as_str())
}
```

[Next page](./08-wrapping-up.md)