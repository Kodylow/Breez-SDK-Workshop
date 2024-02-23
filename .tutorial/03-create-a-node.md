## Create a node
The first thing an app does that implements Breez SDK, is make sure you're 
connected to the user's lightning node. That can mean 2 things. The user already
has a node, or you create a new lightning node for them.

### Creating a new node
```rust
let greenlight_credentials = BreezServices::register_node(
    Network::Bitcoin, 
    seed.to_vec(), 
    None, 
    Some(greenlight_invite_code)
).await?;
```

### Recovering an existing node
In a normal flow, you would persist the `greenlight_credentials` returned by the 
`register_node` function call. Then, if you have persisted credentials, you
connect to the node using those credentials and skip this section. For this
replit, however, we will use the `recover_node` function to recover our 
greenlight credentials.

```rust
let greenlight_credentials = BreezServices::recover_node(
    Network::Bitcoin, 
    seed.to_vec()
).await?;
```

[Next page](./04-initialize-sdk.md)