# DNA Spec v (Draft 6/28)

## Some notes:

- The DNA spec does not include zome references, those would happen as macros in the dev environment.
- This tree is JSON only as convenience, it could be s-expressions or any other tree structure.
- Two levels: the tree has a level which gets interpreted by the Holochain library, and a level that gets interpreted by the Ribosome.  It's not allways clear what goes in which level.  For example, schema used to live at the holochain library level under the `schema` property, because spinning the ribosome to do frequent schema matching didn't seem to make sense.  In the new way of doing things, that validation my be a pre-build Wasm chunk that runs very fast as part all the available validation functions.
- How to treat Wasm code chunks, i.e. do they get assembled into one Wasm module per zome, or are they each seperately sandboxed modules?
- We need function declarations (i.e. names and signatures) Outside the Wasm, kind of like C header files, becasue they get used to validate function calls according to capability membranes at the Holochain library level, not the Wasm level.

## Thinking about link entries

We want to remove the notion of links as an entry format, and instead make declarations about linking in the entry type definition and use that data to dispatch validation instead of dispatching off of the entry-type's type name.  For example:

``` javascript
// OLD WAY:
// Note that you write validation by checking the type in the call to
// `validateLink`
commit("post",{content:"my post"})
commit("post_link",[
  {base:post,tag:"handle",target:myHandle},
  {base:myHandle,tag:"posts",target:post}
])
commit("tag_link",[
  {base:myHandle,tag:"_tag"+theTag, target:post}
  {base:post,tag:"taggedBy",target:myHandle},
])

// New Way:
// Note: that validation code is in the entry type `links-to` block
// which gets found by lookup via the type of the base and the tag.
let bundle = startBundle()
let post = commit("post",{content:"my post"})
link(post, "author", myHandle)
link(myHandle, "posts", post)
link(myHandle, "_tag"+theTag, post)
link(post,"taggedBy", myHandle)
closeBundle(bundle)
```

## Key / Reserved Capabilities

Some functions are required for core process/features of Holochain to work.
They are available in keyword specific Capabilities and function names

### LifeCycle Capability

`"name": "hc_lifecycle"`

#### Genesis
`genesis()`

#### BridgeGenesis
`bridgeGenesis(side, dna, appData)`

#### Migrate

### Communication Capability

`"name": "hc_web_gateway"`

### Receive
`receive(from, message)`
### Send


## Example DNA Tree

``` javascript
{
  "name": "Example app",
  "description": "this app does very cool stuff",
  "version": "0.0.1",
  "uuid": "123e4567-e89b-12d3-a456-426655440000",
  "dna_spec_version": "2.0",
  "dht_config": {
  },
  "properties": {
    "weight": "2kg"
  },
  "zomes": [
    {
      "name": "clutter",
      "description": "zome that implements micro-blogging",
      "config": {
        "error_handling": "throw-errors"
      },
      //"RibosomeType":"Wasm", #do we just commit to Wasm only at
      //  at this stage?
      "entry_types": [
        {
          "name": "post",
          "description": "this entry stores the post and acts as a base for links back to the author",
          "schema": {}, // maybe NOT because happens inside Wasm validations and it's fast enough.

          "validation": ".." // Wasm code

          //"semantics":{}, #??? UI hints do this later
          "sharing": "public", ///private/encrypted
          "links_to": [
            {
              "target_type": "handle",
              "tag": "author",
              "validation": ".." //Wasm
            }
          ]
        },
        {
          "name": "handle",
          "links_to": [
            {
              "target_type": "post",
              "tag": "posts",
              "validation": ".." //Wasm
            },
            {
              "target_type": "post",
              "tag": "_tag*",
              "validation": ".." //Wasm
            }

            ...
          ]
        },
        {
          "name": "handle_link",
          "schema": {
            "role": "string"
          }
        }
      ], // end of entry-types
      "capabilities": [
        {
          "name": "web_gateway",
          "capability": {
            "membrane": "public"
          },
          "fn_declarations": [
            {
              "name": "newPost",
              "signature" :
              {
                "input" : [ "string": "post" ],
                "output" : [ "hash": "hash" ]
              },
            },
            // ...
          ],
          "code": ".." //s-expression encoded Wasm or Base64 encoded Wasm bytecode
        },
        {
          "name": "UI/Browser/Container",
          "capability": {
            "membrane": "agent",
          },
          "fn_declarations": [
            // see above
          ],
          "code": ".." //s-expression encoded Wasm or Base64 encoded Wasm bytecode
        },
        {
          "name": "indexing",
          "capability": {
            "membrane": "api-key",
          },
          "fn_declarations": [
            // see above
          ],
          "code": ".." //s-expression encoded Wasm or Base64 encoded Wasm bytecode
        },
        {
          "name": "library",
          "capability": {
            "membrane": "zome"
          },
          ,
          "fn_declarations": [
            // see above
          ],
          "code": ".." //s-expression encoded Wasm or Base64 encoded Wasm bytecode
        },
      ] // end of capabilities
    }
  ] // end of zomes
}
```
