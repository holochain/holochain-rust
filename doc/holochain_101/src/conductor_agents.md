# Agents

`agents` is an array of configurations for "agents". This means that you can define, and later reference, multiple distinct agents in this single config file. An "agent" has a name, ID, public address and is defined by a private key that resides in a file on their device.

**Required**: `agents` is a required property in the config file. It is the ONLY required property.

### Properties

#### `id`: `string`
Give an ID of your choice to the agent

#### `name`: `string`
Give a name of your choice to the agent

#### `public_address`: `string`
A public address for the agent

#### `key_file`: `string`
Path to the private key file for this agent. This property is not yet in use, so put any value here for the time being.


### Example
```toml
[[agents]]
id = "test agent 1"
name = "Holo Tester 1"
public_address = "HoloTester1-----------------------------------------------------------------------AAACZp4xHB"
key_file = "holo_tester.key"
```
