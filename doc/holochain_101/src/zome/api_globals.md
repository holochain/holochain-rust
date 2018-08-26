# Zome API Constants and Application Variables

There are two kinds of API values that are available in zome code:
 1. system-wide global constants
 2. app-specific global variables

## Reference

Note: Full reference is available in language-specific API Reference documentation.
(TODO add links)

### System-wide global constants

| Name        | Purpose           | 
| ------------- |:-------------| 
| VERSION      | Version of the Holochain software running the zome | 
| HashNotFound      | Value returned when a hash provided could not be found. | 
| Status | Enum holding all possible state of an entry. | 
| GetEntryMask | Mask values used for calling the `get_entry` Zome API Function. |
| LinkAction | Constants used for calling the `link_entries` Zome API Function. |
| PkgRequest | TODO |
| ChainOption | TODO |
| BridgeSide | TODO |
| SysEntryType | Enum holding all possible types of system entries |
| bundle_cancel.Reason | Enum used as argument for `bundle_canceled` callback |
| bundle_cancel.Response | Enum used as return value for `bundle_canceled` callback |
 

### app-specific global variables

| Name        | Purpose           | 
| ------------- |:-------------| 
| APP_NAME | Name of the Holochain app taken from the DNA. |
| APP_DNA_HASH | The hash of this Holochain's DNA |
| APP_AGENT_ID_STR | The identity string used to initialize this Holochain with `hcadmin init`. |
| APP_AGENT_KEY_HASH | The hash of local agent's public key. |
| APP_AGENT_INITIAL_HASH | The hash of the first identity entry on the local chain. |
| APP_AGENT_LATEST_HASH | The hash of the most recent identity entry that has been committed to the local chain. |
