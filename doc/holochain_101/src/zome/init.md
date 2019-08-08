# Init and Validate Agent

## The Initialization Process

Recall that every peer will be running instances of DNA on their device. This is how peers join the network for a given DNA. There are some actions that a developer may wish to initiate, by an agent, upon them joining a network. For this reason, a hook into this moment of the lifecycle is implemented by Holochain. 

This lifecycle stage is called `init`. It is a callback that Holochain expects every single Zome to implement, because it will call it during initialization. If it does not exist within the WASM code for a Zome it will cause an error and peers will not be able to launch an instance of the DNA.

Another important principle in Holochain is that not all networks are public. Some DNAs should only be joinable by agent meeting certain requirements. Some examples of this might be possessing a valid invite key, being on a whitelist or even meeting some requirements in another DNA. We refer to this access control as the *membrane* of the network.

To enforce the membrane requirements Holochain exposes a callback called `validate_agent`. This is analogous to a validation function for an entry but the entry is the agent themselves. Similar to an entry validation callback it is run by DHT nodes before they will acknowledge that the new agent has joined.

When Holochain is attempting to launch an instance of a Zome, it will iterate through all the Zomes one by one, calling `init` and `validate_agent` callbacks within each. If each succeeds, success. If any one fails, the launch will fail, and the error string will be returned to the peer.

Holochain will wait up to 30 seconds for a `init` response from the Zome, before it will throw a timeout error.

## Building in Rust: init

How is `init` used within the Rust HDK?

[Previously](./define_zome.md), the general structure of a zome module has been covered. It includes an annotated function called `init`, which is passed zero arguments. This is the hook that Holochain is expecting. It expects a Rust `Result` as a return value, which is either `Ok(())` or an `Err`, with the string explaining the error.

In the following two examples, nothing interesting will happen in the `init` functions, they are simply to illustrate how to return success, and how to return failure.

More complex capabilities will be possible during `init` in the future, yet for now, using the first simple example that succeeds is recommended.

If `init` should succeed:
```rust
#[zome]
mod my_zome {
    
    #[init]
    fn init() -> ZomeApiResult<()> {
        Ok(())
    }

}
```

If `init` should fail:
```rust
#[zome]
mod my_zome {
    
    #[init]
    fn init() -> ZomeApiResult<()> {
        Err("Somem error string".to_string())
    }

}
```

## Building in Rust: validate_agent

Validate agent is required by every zome. Unlike init the callback takes a single argument, the entry validation data for the agent. This is identical to the `EntryValidationData` passed to entry validation callbacks and is an enum which contains the agent/authors public key, agent data and signed header.

If the zome has no membrane requirements: 

```rust
#[zome]
mod my_zome {

    #[validate_agent]
    fn validate_agent(_validation_data : EntryValidationData::<AgentId>) -> ZomeApiResult {
        Ok(())
    }

}
```

An example of ensuring the agents private key is in a hard-coded whitelist. One use-case for this could be private chat channels:

```rust

static WHITELIST_AGENT_KEYS: &'static [&str] = &["<keys-go-here>"];

#[zome]
mod my_zome {

    #[validate_agent]
    fn validate_agent(validation_data : EntryValidationData::<AgentId>) -> ZomeApiResult {
        if let EntryValidationData::Create{entry, ..} = validation_data {
            let agent = entry as AgentId;
            if WHITELIST_AGENT_KEYS.contains(&agent.pub_sign_key.as_str()) { {
                Ok(()) // the agent is allowed
            } else {
                Err("This agent is not on the whitelist".to_string())
            }
        } else {
            Err("Cannot update or delete an agent at this time".into())
        }
    }

}
```
