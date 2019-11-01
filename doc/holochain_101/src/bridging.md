# Building Holochain Apps: Bridging

As you saw in [Building Apps](./building_apps.md) each **DNA** has a unique hash that spawns a brand new **DHT network** and creates isolated **source chains** for each agent. Even when you change the DNA, releasing a new version of the app, it will spawn a brand new DHT network and source chains.

So if every app lives in an entirely separated world how can they talk to each other? This is where **bridging** comes into play.

A **bridge** is a connector between two apps (or two versions of the same app, for that matter) that allows a synchronous bidirectional transfer of information between them.

To use a bridge, right now you need to configure a [production Holochain conductor](./production_conductor.md), at least two instances configured, along the lines of the following example setup (in a **conductor-config.toml** file):

```
[[instances]]
id = "caller-instance"
dna = "caller-dna"
agent = "caller-agent"
[instances.logger]
type = "simple"
[instances.storage]
type = "memory"

[[instances]]
id = "target-instance"
dna = target-dna"
agent = "target-agent"
[instances.logger]
type = "simple"
[instances.storage]
type = "memory"

[[bridges]]
caller_id = "caller-instance"
callee_id = "target-instance"
handle = "sample-bridge"
```

Then on the caller DNA you have to initiate the bridge call using `hdk::call` like this:

```rust
    let response = match hdk::call(
        "sample-bridge",
        "sample_zome",
        Address::from(PUBLIC_TOKEN.to_string()), // never mind this for now
        "sample_function",
        json!({
            "some_param": "some_val",
        }).into()
    ) {
        Ok(json) => serde_json::from_str(&json.to_string()).unwrap(), // converts the return to JSON
        Err(e) => return Err(e)
    };
```

And the corresponding target / callee DNA on the other end should have a zome called "sample_zome", with a function as follows:
```rust
pub fn handle_sample_function(some_param: String) -> ZomeApiResult<Address> {
    // do something here
}

define_zome! {
    entries: []

    init: || { Ok(()) }

    validate_agent: |validation_data : EntryValidationData::<AgentId>| {
        Ok(())
    }

    functions: [
        sample_function: {
            inputs: |some_param: String|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_sample_function
        }
    ]

    traits: {
        hc_public [
            sample_function
        ]
    }
```

Remember that the **call** will block the execution of the caller DNA until the callee (target) finishes executing the call, so it's best to mind performance issues when working with bridges. Try to make contextual or incremental calls rather than all-encompassing ones.
