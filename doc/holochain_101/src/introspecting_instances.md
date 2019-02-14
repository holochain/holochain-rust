# Introspecting Instances

### `conductor.agent_id(instanceId)` => `string`

Get the agent_id for an instance, by passing an instance id.

___
**Name** instanceId

**Type** `string`

**Description** Specifies an instance by its instanceId. This instanceId should be the equivalent thing as an `instanceConfig.name` which was passed to [Config.instance](./testing_configuration.md#instances). This in turn would be equivalent to the original name given to [Config.agent](./testing_configuration.md#agents), unless you overrode it when calling [Config.instance](./testing_configuration.md#instances). See more [here](./testing_configuration.md#example-2).
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

**Description** Specifies an instance by its instanceId. This instanceId should be the equivalent thing as an `instanceConfig.name` which was passed to [Config.instance](./testing_configuration.md#instances). This in turn would be equivalent to the original name given to [Config.agent](./testing_configuration.md#agents), unless you overrode it when calling [Config.instance](./testing_configuration.md#instances). See more [here](./testing_configuration.md#example-2).
___

#### Example

```javascript
const dnaAddress = conductor.dna_address('alice')
```