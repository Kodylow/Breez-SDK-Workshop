## Calling a HTTP 402 api

```rust
println!("Calling http 402 API without a token.");
let url = "https://testing";
let client = reqwest::ClientBuilder::new().build()?;
let mut resp = client.get(url).send().await?;
println!("Response status is {}", resp.status());
if resp.status() == 402 {
    let l402header = resp.headers()
        .get("WWW-Authenticate")
        .expect("server did not return WWW-Authenticate header in 402 response.")
        .to_str()?;
    let re = regex::Regex::new(r#"/^L402 (token|macaroon)=\"(?<token>.*)\", invoice=\"(?<invoice>.*)\""#)?;
    let caps = re.captures(l402header).expect("WWW-Authenticate header is not a valid L402");
    let token = caps["token"].to_string();
    let invoice = caps["invoice"].to_string();

    println!("Paying lightning invoice to get access to the API: {}", invoice);
    let payresult = sdk.send_payment(invoice, None).await?;
    let lnpayresult = match payresult.details {
        breez_sdk_core::PaymentDetails::Ln { data } => data,
        _ => unreachable!(),
    };

    let header = format!("L402 {}:{}", token, lnpayresult.payment_preimage);
    println!("Calling http 402 api again, now with header Authorization {}", header);
    resp = client.get(url).header("Authorization", header).send().await?;
} 

let status = resp.status();
println!("Got Response. Status {}", status);
let text = resp.text().await?;
println!("{}", text);
```
