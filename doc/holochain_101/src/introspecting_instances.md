# Introspecting Instances

### `conductor.agent_id(instanceId)` => `string`

Get the agent_id for an instance, by passing an instance id.

___
**Name** instanceId

**Type** `string`

**Description**
___

#### Example

```javascript
const aliceAgentId = conductor.agent_id('alice')
```

### `conductor.dna_address(instanceId)` => `string`

Get the address of the DNA for an instance, by passing an instance id.

___
**Name** instanceId

**Type** `string`

**Description**
___

#### Example

```javascript
const dnaAddress = conductor.dna_address('alice')
```