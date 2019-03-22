# Agents

`agents` is an array of configurations for "agents". This means that you can define, and later reference, multiple distinct agents in this single config file. An "agent" has a name, ID, public address and is defined by a private key that resides in a file on their device.

**Required**: `agents` is a required property in the config file. It is the ONLY required property.

### Properties

#### `id`: `string`
Give an ID of your choice to the agent

#### `name`: `string`
Give a name of your choice to the agent

#### `public_address`: `string`
A public address for the agent. Run ```hc keygen``` and copy the public address to this value

#### `keystore_file`: `string`
Path to the keystore file for this agent. Copy the path from when you ran ```hc keygen``` into this value.


### Example
```toml
[[agents]]
id = "test_agent2"
name = "HoloTester2"
public_address = "HcSCJts3fQ6Y4c4xr795Zj6inhTjecrfrsSFOrU9Jmnhnj5bdoXkoPSJivrm3wi"
keystore_file = "/org.holochain.holochain/keys/HcSCJts3fQ6Y4c4xr795Zj6inhTjecrfrsSFOrU9Jmnhnj5bdoXkoPSJivrm3wi"
```
