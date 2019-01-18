# Genesis

## The Initialization Process

Recall that every peer will be running instances of DNA on their device. This is how peers join the network for a given DNA. There are some actions that a developer may wish to initiate, upon a new peer joining a network. For this reason, a hook into this moment of the lifecycle is implemented by Holochain. It also provides an opportunity to reject the user from joining, if for some reason there is an issue with the way they are attempting to.

This lifecycle stage is called `genesis`. It is a callback that Holochain expects every single Zome to implement, because it will call it during initialization. If it does not exist within the WASM code for a Zome it will cause an error and peers will not be able to launch an instance of the DNA.

This function also has the opportunity to reject the success of the launch of the instance. If it succeeds, the expected return value is just an integer (in WASM) representing that, but if it fails, a string is expected to be passed, explaining why.

When Holochain is attempting to launch an instance of a Zome, it will iterate through all the Zomes one by one, calling `genesis` within each. If each succeeds, success. If any one fails, the launch will fail, and the error string will be returned to the peer.

Holochain will wait up to 30 seconds for a `genesis` response from the Zome, before it will throw a timeout error.

Of course, this also indicates that `genesis` is a reserved function name and should not be used as the name of any other function that is publicly callable in the Zome.


## Building in Rust: genesis

How is `genesis` used within the Rust HDK?

[Previously](./define_zome.md), the general structure of `define_zome!` has been covered. It includes a Rust function closure called `genesis`, which is passed zero arguments. This is the hook that Holochain is expecting. It expects a Rust `Result` as a return value, which is either `Ok(())` or an `Err`, with the string explaining the error.

In the following two examples, nothing interesting will happen in the `genesis` functions, they are simply to illustrate how to return success, and how to return failure.

More complex capabilities will be possible during `genesis` in the future, yet for now, using the first simple example that succeeds is recommended.

If `genesis` should succeed:
```rust
define_zome! {
    entries: []

    genesis: || {
        Ok(())
    }

    functions: {}
}
```

If `genesis` should fail:
```rust
define_zome! {
    entries: []

    genesis: || {
        Err("the error string".to_string())
    }

    functions: {}
}
```
