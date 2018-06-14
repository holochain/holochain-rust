# DNA Spec v (Draft 6/13)

## Some notes:

- The DNA spec does not include zome references, those would happen as macros in the dev environment.
- This tree is JSON only as convenience, it could be s-expressions or any other tree structure.
- Two levels: the tree has a level which gets interpreted by the Holochain library, and a level that gets interpreted by the Ribosome.  It's not allways clear what goes in which level.  For example, schema used to live at the holochain library level under the `schema` property, because spinning the ribosome to do frequent schema matching didn't seem to make sense.  In the new way of doing things, that validation my be a pre-build wasm chunk that runs very fast as part all the available validation functions.
- How to treat wasm code chunks, i.e. do they get assembled into one WASM module per zome, or are they each seperately sandboxed modules?
- We need function declarations (i.e. names and signatures) Outside the WASM, kind of like C header files, becasue they get used to validate function calls according to capability membranes at the Holochain library level, not the WASM level.

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

## Example DNA Tree:

``` javascript
{
    "name": "Example app",
    "description: "this app does very cool stuff",
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
            //"RibosomeType":"WASM", #do we just commit to WASM only at
            //  at this stage?
            "entry-types": [

                {
                    "name": "post",
                    "description": "this entry stores the post and acts as a base for links back to the author",
                    "schema": {}, // maybe NOT because happens inside WASM validations and it's fast enought.

                    "validation": ".." // WASM code

                    //"semantics":{}, #??? UI hints do this later
                    "sharing": "public", ///private/encrypted
                    "links_to": [
                        {
                            "target_type": "handle",
                            "tag": "author",
                            "validation": ".." //WASM
                        }
                    ]
                },
                {
                    "name": "handle",
                    "links_to": [
                        {
                            "target_type": "post",
                            "tag": "posts",
                            "validation": ".." //WASM
                        },
                        {
                            "target_type": "post",
                            "tag": "_tag*",
                            "validation": ".." //WASM
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
                            "name": "xxx",
                            "signature" :
                            {
                                "input" : [
                                ],
                                "output" : [
                                ]
                            },
                        },
                        ...
                    ],
                    "code": ".." //s-expresion encoded wasm or Base64 encoded WASM bytecode
                },
                {
                    "name": "UI/Browser/Container",
                    "capability": {
                        "membrane": "agent",
                    },
                    "fn_declarations": [
                        // see above
                    ],
                     "code": ".." //s-expresion encoded wasm or Base64 encoded WASM bytecode
                },
                {
                    "name": "indexing",
                    "capability": {
                        "membrane": "api-key",
                    },
                    "fn_declarations": [
                        // see above
                    ],
                     "code": ".." //s-expresion encoded wasm or Base64 encoded WASM bytecode
                },
                {
                    "name": "library",
                    "capability": {
                        "membrane": "zome"
                    },
                    ,
                    "fn_functions": [
                    ],
                     "code": ".." //s-expresion encoded wasm or Base64 encoded WASM bytecode
                },
            ] // end of capabilities
        }
    ] // end of zomes
}
```
