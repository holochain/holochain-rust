# Calling Zome Functions

### `conductor.call(instanceId, zomeName, functionName, callParams)` => `object`

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
const callResult = conductor.call('alice', 'zome', 'function', {params: 'go here'})
```

## Simplifying Call

### `conductor.makeCaller(instanceId)` => `object`

`makeCaller` is for convenience. Instead of passing the instanceId every time, we can retrieve an object that could be considered equivalent to an actual instance.

#### Example

```javascript
const aliceName = "alice"
// ...
const alice = conductor.makeCaller(aliceName)
const altCallResult = alice.call('zome', 'function', {params: 'go here'})
```

