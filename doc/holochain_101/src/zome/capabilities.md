# Capabilities

## Overview
Holochain uses a modified version of the [capabilities](https://en.wikipedia.org/wiki/Capability-based_security) security model.  Holochain DNA instances will grant revokable, cryptographic capability tokens which are shared as access credentials. Appropriate access credentials must be used to access functions and private data.

This enables us to use a single security pattern for:

- connecting end-user UIs,
- calls across zomes within a DNA,
- bridging calls between different DNAs,
- and providing selective users of a DNA the ability to query private entries on the local chain via send/receive.

Each capability grant gets recorded as a private entry on the grantor’s chain.  The hash (i.e. address) of that entry is then serves as the capability token usable by the grantee when making zome function call, because the grantor simply verifies the existence of that grant in it's chain.  Thus, all zome functions calls include a capability request object which contains: public key of the grantee and signature of the parameters being used to call the function, along with the capability token being used as the access credential.

## Using Capabilities

### Public Capabilities
You can declare some functions as "public"  using the special `hc_public` marker trait in your `define_zome!` call.  Functions in that trait will be added to the public capability grant which gets auto-committed during init.  Like this:

```
define_zome! {

...

   traits: {
       hc_public [read_post, write_post]
   }

...

}
```

### Grant Capabilities

You can use the `commit_capability_grant` HDK function to create a custom capability grant.  For example, imaging a blogging use-case where you want to grant friends the ability to call the `create_post` function in a `blog` zome.  Assuming the function `is_my_friend(addr)` correctly examines the provenance in CAPABILITY_REQ global which always holds the capability request of the current zome call, then the following code is an example of how you might call `hdk::commit_capability_grant`:

``` rust
pub fn handle_request_post_grant() -> ZomeApiResult<Option<Address>> {
    let addr = CAPABILITY_REQ.provenance.source();
    if is_my_friend(addr.clone()) {
        let mut functions = BTreeMap::new();
        functions.insert("blog".to_string(), vec!["create_post".to_string()]);
        Ok(Some(hdk::commit_capability_grant(
            "can_post",
            CapabilityType::Assigned,
            Some(vec![addr]),
            functions,
        )?))
    } else {
        Ok(None)
    }
}
```

### Capabilities in Bridging

TBD.
