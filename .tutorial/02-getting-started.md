## Environment
Make sure there's a `.env` file as shown in `.env.example`, or replit secrets
set to the the prerequisites defined in [prerequisites](./01-prerequisites.md). 

## Breez SDK reference
Add a reference to Breez SDK.

In `Cargo.toml`, under `[dependencies]`, add the following line

```
breez-sdk-core = { git = "https://github.com/breez/breez-sdk", tag = "0.1.1" }
```

## Starter code

```rust
use bip39::{Language, Mnemonic};
use dotenv::dotenv;
use std::{io, env};
use once_cell::sync::Lazy;

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
    println!("All environment variables are set, we're good to go.");
    Ok(())
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

[Next page](./03-create-a-node.md)