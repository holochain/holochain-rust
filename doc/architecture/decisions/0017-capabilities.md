# Capabilities & Security ADR

Date: 2019-02-26

## Status
Draft

## Context

A unified security model for Holochain applications:
* Each zome must be able to represent and enforce its own security modeling because that is the appropriate place to do so. (Push the intelligence to the edges.)
* Developers must be able to build in granualar specificity and revokability of access to functions and entries.
* We must be able to distinguish between application security model, and architectural and code security model.  I.e. what is the security model application developers build into their apps, and the security model of Conductor/Core, etc.  E.g. we have to ensure that zome calls aren't subject to replay attacks in general, and also allow zome developers to declare and create security policies in specific.

## Decision

Holochain will use a variant of the [capabilities](https://en.wikipedia.org/wiki/Capability-based_security) security model. Holochain DNA instances will grant revokable, cryptographic capability tokens which are shared as access credentials. Appropriate access credentials must be used to access to functions and private data.

This enables us to use a single security pattern for:
 - connecting end-user UIs,
 - calls across zomes within a DNA,
 - bridging calls between different DNAs,
 - and providing selective users of a DNA the ability to query private entries on the local chain via send/receive.

Each capability grant gets recorded as a private entry on the grantor's chain, and are validated against for every zome function call.

## Elements:
### CapabilityGrant
The CapabilityGrant is by default a private entry recorded on the chain because this does not need to be published to the DHT (though it may be in some cases). The Address (i.e. hash) of these entries serves as the capability token credential that is part of an access request.  The instance can look up the grant by that address and confirm if the request conforms to the grant type.   DNA developers can do this manually using `grant` and `verify_grant` api calls for custom access use-cases, but this is also done automatically to verify both zome function calls as well as bridge and cross zome calls.

#### Grant attributes:
1. **Asignees:** a list of agents to whom the grant applies.  If the list is not specified then, then the grant is assumed to be transferable, i.e. possesion of the token is sufficient and serves as a password.  Note that "anonymous" and assigned grants are mutually exclusive.
2. **Pre-filled Parameters:** A zome-function call grant may also specify a template for parameter values of a the function being granted access.  This allows for "currying" type behavior on grants, where the grant itself forces a function parameter to specifc value.

*Comments:* In the past we have talked about the "public" capability. All access must be signed.  "public" access comes from publishing an unassigned token publicly.

#### Special Case -- Agent Grant:
There is one special case grant, the **Agent Grant**.  So far in Holochain, we have been saying that the second entry on the chain, after the DNA, is the AgentID entry, that identifies the agency by public-key that "owns" the chain.  We have also talked about (and implemented in proto along with the revoking and re-issue of the AgentID entries). In the capabilities security model we can unify this with capability grants where that second entry can be thought of as "super-user/admin/root" capability which should indeed be granted to the agent who "owns" the chain, but the token of that agency (the public key) may need to be revoked and replace, just like any other capability token.

### CapabilityRequest
A capability request is a structure for making a request referring to a particular capability grant.

Request attributes:
1. **Token:** the address of the capability grant entry committed to the chain
2. **Provenance:** the address of the requester and the signature of request contents.
3. **Contents:** the exact data of the contents of the request that is signed.  In the case of zome function call requests this is the function name, the function parameters, and a nonce for preventing replay attacks.

*Comments:* Core has to be able to do three things with a CapabilityRequst:
1. Load the grant from the address (or detect that this is a special case grant)
2. Confirm that the Contents matches the the Provenance.
3. Extract the data from the contents that's needed for the purpose, i.e. actually get the function name and parameters of the zome-call.

## Processes
### Zome Function Calls

Capabilities allow developers to specify control access to zome function calling, either from exterior calls or via bridging or even cross-zome calling.  This latter may seem odd, but it's important for enabling more secure development patterns for zome mix-ins.  See Consequences for details.  A broad overview of how zome function call from an outside source would flow under the capabilities model:

1. Agent bundles function name, parameters and a timestamp into a call request block, and signs it with the agent's private key, and sends it to Conductor along with any other parameters necessary for routing to the correct instance over what ever interface is being used.
2. Conductor creates a CapabilityRequest structure and passes it into holochain Core.
3. Core loads CapabilityGrant from chain by it's address, and checks validity according to the grant's parameters, returning a CapabilityCheckFailed error, or calling the zome function if successfull.  It may also check the timestamp of the call to make-sure it's within a reasonble window to prevent some re-play attacks.  Additionally, if complete security from replay attacks is necessary we may implement an additional handshake where the agent makes a "pre-call" indicating the desire to make a zome call.  In that case the Conductor would have to pass this request into core where a nonce could be generated that the client has to include in the call request block. Note that this also requires implementing an ephemeral store in core, something that's on our development path.

*Comments:*

1. The "agent" above could be a web-UI that holds the private key (in some Holo cases) or could be an extended Conductor that is an electron app.  In the former case, the necessary token grants have to be passed to the UI.  If the agency in the UI is the "owner" of the chain, then it can effectively use the special case agent grant to make any zome call it wants.  Otherwise, it will have to have received the public token, or an assigned or transferable token.  See Consequences below.

#### Calling Elements/Structs
We will use a simple JSON structure for what gets signed by which ever component of the system has the private key:

TBD. Draft structures: https://hackmd.io/cvXMlcffThSpN-C5WrfGzg?view#

### Genesis

- We use the convention of using a reserved-trait name ("hc_public") to identify functions for which such a public grant can be created at genesis time, and be made available to the Conductor to send to UIs (or proxy on their behalf in the various use-cases i.e. as a web-proxy) for creating provenance for public access.

*Comments:* this has been implemented in `capabilities-3`

### Bridging

TBD

## Consequences

### Exposing Tokens

#### Public Token
We need to add a method/convention for agents to be able to access the public token generated at genesis time.  In `capabilities-3` the public token is returned as part of the results of initialization during genesis.  That initialization data strcture from genesis should be made available in the HDK through a new PUBLIC_TOKEN global, and to the conductor for additions to the conductor_api.

#### Conventions & examples for generated tokens
We need to provide some examples of zome functions/patterns of how to request tokens, generate them and return them for use by calling agents, both at the level of UI zome function calling, and at the application level for use in node-to-node send & receive communications.

### Zome mix-in security
The capability model is not only useful for extra-membrane security, but also intra-membrane security for Cross-zome calls.  Because our composibility model includes drop-in zomes, for which the developer may not be able to see the source code (i.e. they only get the WASM), it is important to create the ability to make calling functions in other zomes subject to a capability request on a specific grant.

For this to work, we may need to expand the expressivity of the `sign` API call.  i.e. we may need to limit under what agency that call can be made for certain zomes.
