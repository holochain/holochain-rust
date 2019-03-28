# Crypto Functions

Holochain DNA instances are designed to function in the context of a [Distributed Public Key Insfrastructure (DPKI)](./dpki.md) which:

1. manages key creation and revocation
2. manages an agency context that allows grouping and verifying that sets of DNA instances are controlled by the same agent.
3. creates a context for identity verification

Holochain assumes that there may be different DPKI implementations but provides a reference implementation we call DeepKey.   We assume that the DPKI implementation is itself a Holochain application, and we provide access to a set of generic cryptographic functions.  These functions also allow DNA authors to build ad-hoc cryptogrpahic protocols.

For each Holochain DNA instance, the conductor maintains a Keystore, which holds "secrets" (seeds and keys) needed for cryptographic signing and encrypting. Each of the secrets in the Keystore is associated with a string which is a handle needed when using that secret for some cryptographic operation.  Our cryptographic implementation is based on libsodium, and the seeds use their notions of context and index for key derivation paths.  This implementation allows DNA developers to securely call cryptographic functions from wasm which will be executed in the conductor's secure memory space when actually doing the cryptographic processing.

Here are the available functions:

- `keystore_list()` -> returns a list of all the secret identifiers in the keystore
- `keystore_new_random(dst_id)` -> creates a new random root seed identified by `dst_id`
- `keystore_derive_seed(src_id,dst_id,context,index)` -> derives a higherarchical deterministic key seed to be identifided by `dst_id` from the `src_id`.  Uses `context` and `index` as part of the derivation path.
- `keystore_derive_key(src_id,dst_id,key_type)`-> derives a key (signing or encrypting) to be identified by `dst_id` from a previously created seed identified by `src_id`.  This function returns the public key of the keypair.
- `keystore_sign(src_id,payload)` -> returns a signature of the payload as signed by the key identified by `src_id`
- `sign(payload)` -> signs the payload using the DNA's instance agent ID public key.  This is a convenience function which is equivalent to calling `keystore_sign("primary_keybundle:sign_key",payload)`
- `sign-one-time(payload)` -> signs the payload with a randomly generated key-pair, returning the signature and the public key of the key-pair after shredding the private-key.
- `verify_signature(provenance, payload)` -> verifies that the `payload` matches the `provenance` which is a public key/signature pair.

Not Yet Implemented:

- `encrypt(payload)` -> encrypts
- `keystore_encrypt(src_id,payload)`
