# Calling Zome Functions

### `conductor.call(instanceId, zomeName, functionName, callParams)` => `object`

?? desc
___
**Name** instanceId

**Type** `string`

**Description** 
___
**Name** zomeName

**Type** `string`

**Description**
___
**Name** functionName

**Type** `string`

**Description**
___
**Name** callParams

**Type** `object`

**Description**
___

#### Example

```javascript
const callResult = conductor.call('alice', 'people', 'create_person', {name: 'Franklin'})
```

## Simplifying Call

### `conductor.makeCaller(instanceId)` => `object`

`makeCaller` is for convenience. Instead of passing the instanceId every time, we can retrieve an object that could be considered equivalent to an actual instance. An instance with the given instanceId must exist, otherwise it will throw an error.

#### Example

```javascript
// ...
const alice = conductor.makeCaller('alice')
const callResult = alice.call('people', 'create_person', {name: 'Franklin'})
```

