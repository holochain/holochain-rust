# Distributed Public Key Infrastructure (DPKI)

## Context

Like most other distributed systems, Holochain fundamentally relies on public key cryptography. Among other uses, Holochain nodes are identified by their public keys, and thus provenance of messages from nodes can be checked simply by checking a message signature against the node's identifier.  This fundamental use requires nodes to keep the private key secret, lest the node's very agency be compromised.  Keeping such secrets in the digital age is a non-trivial problem.  Additionally, distributed systems create a context of a profusion of nodes with multiple public keys, many of which may wish to be identified as being under the control a of a single actor.  Solutions to this need create yet another layer of keys which sign other keys, and the management of all this can become quite complex.

Addressing these needs is the function of Public Key Infrastructure, and in our case, because we use the power of Holochain itself to do this, a Distributed Public Key Infrastructure: DPKI.

## Requirements

DPKI needs to fulfill at least the following design requirements:

1. Provide a way create new keys for nodes.
2. Provide a way to revoke compromised keys and re-issue keys for a node.
3. Provide a way to verify the proveneance of keys by grouping them as originating from a single actor.
4. Securely manage the private keys.
5. Reliably distribute and make available information about public keys.

In designing a solution for DPKI we recognize that the this is a complex and difficult enough problem that any solution will need to evolve, and in fact there will be multiple solutions necessary for different context.  Thus, we have built into the Holochain conductor a simple interface for the fundamental needed functions, like creating new keys when installing a DNA for the first time, that can then be implemented by special DPKI applications.  Furthermore we've implemented a reference implementation of a Holochain based DPKI application, which we call DeepKey.

## DeepKey

TODO: merge the various docs we developed to explain DeepKey here.
- https://medium.com/holochain/part-2-holochain-holo-accounts-cryptographic-key-management-and-deepkey-bf32ee91af65
- https://hackmd.io/UbfvwQdJRKaAHI9Xa7F3VA?view
- https://hackmd.io/8c8rZCyaTTqH_7TIBVtEUQ
- https://hackmd.io/oobu0sKMSMadLXza4rHY_g

## Technical Details

For each Holochain DNA instance, the Conductor maintains a Keystore, which holds "secrets" (seeds and keys) needed for cryptographic signing and encrypting. Each of the secrets in the Keystore is associated with a string which is a handle needed when using that secret for some cryptographic operation.  Our cryptographic implementation is based on libsodium, and the seeds use their notions of context and index for key derivation paths.  This implementation allows DNA developers to securely call cryptographic functions from wasm which will be executed in the conductor's secure memory space when actually doing the cryptographic processing.

TODO: describe the conductor bootstrap flow here
