use anyhow::Result;
use bip39::{Language, Mnemonic};
use breez_sdk_core::{
    input_parser::parse, BreezEvent, BreezServices, EnvironmentType, EventListener, Network,
};
use dotenv::dotenv;
use once_cell::sync::Lazy;
use std::{env, io};
use tokio::sync::Notify;

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

fn generate_mnemonic() -> Result<()> {
    let mnemonic = Mnemonic::generate_in(Language::English, 12)?;
    println!("Generated mnemonic: {mnemonic}");
    println!("Set the environment variable 'MNEMONIC', and run again.");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let breez_sdk_api_key =
        get_env_var("BREEZ_API_KEY").expect("set the 'BREEZ_API_KEY' environment variable");
    let greenlight_invite_code = get_env_var("GREENLIGHT_INVITE_CODE")
        .expect("set the 'GREENLIGHT_INVITE_CODE' environment variable");

    let phrase = match get_env_var("MNEMONIC") {
        Ok(m) => m,
        _ => return generate_mnemonic(),
    };

    let mnemonic = Mnemonic::parse(phrase)?;
    let seed = mnemonic.to_seed("");

    println!("Checking whether user has an existing node.");
    let (greenlight_credentials, is_new_node) =
        match BreezServices::recover_node(Network::Bitcoin, seed.to_vec()).await {
            Ok(c) => {
                println!("User has existing node.");
                (c, false)
            }
            Err(_) => {
                println!("No existing node yet, creating a new Greenlight node.");
                (
                    BreezServices::register_node(
                        Network::Bitcoin,
                        seed.to_vec(),
                        None,
                        Some(greenlight_invite_code),
                    )
                    .await?,
                    true,
                )
            }
        };

    println!("Starting the Breez SDK.");
    let mut config = BreezServices::default_config(EnvironmentType::Production);
    config.api_key = Some(breez_sdk_api_key);
    let sdk = BreezServices::init_services(
        config,
        seed.to_vec(),
        greenlight_credentials,
        Box::new(AppEventListener {}),
    )
    .await?;
    BreezServices::start(&sdk).await?;

    println!("Fetching node info.");
    let node_info = sdk
        .node_info()?
        .expect("Expected node_info to not be None.");
    println!(
        "Node id: {}, balance msat {}",
        node_info.id, node_info.max_payable_msat
    );

    if is_new_node || node_info.max_payable_msat == 0 {
        let invoice = sdk
            .receive_payment(10000, "My first Breez SDK invoice".to_string())
            .await?;
        println!("Created invoice: {}", invoice.bolt11);

        println!("Waiting for the invoice to be paid... (pay it with another lightning app)");
        INVOICE_PAID_NOTIFIER.notified().await;
        println!("Invoice got paid!");
    }

    loop {
        println!("Paste an invoice to pay in the console and press Enter.");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("error: unable to read user input");
        input = input
            .strip_suffix("\r\n")
            .or(input.strip_suffix("\n"))
            .unwrap_or(&input)
            .to_string();

        match parse(input.as_str()).await {
            Ok(parsed) => match parsed {
                breez_sdk_core::InputType::Bolt11 { invoice } => {
                    println!("Sending payment for invoice {}", invoice.bolt11);
                    sdk.send_payment(invoice.bolt11, None).await?;
                }
                _ => println!("Input is not a bolt11 invoice"),
            },
            Err(e) => println!("Input is not a bolt11 invoice: {}", e),
        }
    }
    // println!("Calling http 402 API without a token.");
    // let url = "https://testing";
    // let client = reqwest::ClientBuilder::new().build()?;
    // let mut resp = client.get(url).send().await?;
    // println!("Response status is {}", resp.status());
    // if resp.status() == 402 {
    //     let l402header = resp.headers()
    //         .get("WWW-Authenticate")
    //         .expect("server did not return WWW-Authenticate header in 402 response.")
    //         .to_str()?;

    //     println!("Got WWW-Authenticate header: {}", l402header);
    //     let re = regex::Regex::new(r#"/^L402 (token|macaroon)=\"(?<token>.*)\", invoice=\"(?<invoice>.*)\""#)?;
    //     let caps = re.captures(l402header).expect("WWW-Authenticate header is not a valid L402");
    //     let token = caps["token"].to_string();
    //     let invoice = caps["invoice"].to_string();

    //     println!("Paying lightning invoice to get access to the API: {}", invoice);
    //     let payresult = sdk.send_payment(invoice, None).await?;
    //     let lnpayresult = match payresult.details {
    //         breez_sdk_core::PaymentDetails::Ln { data } => data,
    //         _ => unreachable!(),
    //     };

    //     let header = format!("L402 {}:{}", token, lnpayresult.payment_preimage);
    //     println!("Calling http 402 api again, now with header Authorization {}", header);
    //     resp = client.get(url).header("Authorization", header).send().await?;
    // }

    // let status = resp.status();
    // println!("Got Response. Status {}", status);
    // let text = resp.text().await?;
    // println!("{}", text);
    // Ok(())
}

fn get_env_var(name: &str) -> Result<String, String> {
    let v = env::var(name).expect("variable not set");

    if v.is_empty() {
        return Err("variable is empty".to_string());
    }

    Ok(v)
}
