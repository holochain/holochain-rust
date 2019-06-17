<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->
**Contents**

- [API DNA Variables](#api-dna-variables)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

# API DNA Variables

Note: Full reference is available in language-specific API Reference documentation.

For the Rust `hdk`, [see here](https://developer.holochain.org/api/0.0.7-alpha/hdk/api/index.html#structs)

| Name        | Purpose           |
| ------------- |:-------------|
| DNA_NAME | Name of the Holochain DNA taken from the DNA. |
| DNA_ADDRESS | The address of the DNA |
| AGENT_ID_STR | The identity string used to initialize this Holochain |
| AGENT_ADDRESS | The address (constructed from the public key) of this agent. |
| AGENT_INITIAL_HASH | The hash of the first identity entry on the local chain. |
| AGENT_LATEST_HASH | The hash of the most recent identity entry that has been committed to the local chain. |
| CAPABILITY_REQ | The capability request that was used to run the zome call |
