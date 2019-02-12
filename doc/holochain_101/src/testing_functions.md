# Testing Functions

??

Start the Conductor and the instances
```javascript
conductor.start()
```

```javascript
const conductor = new Conductor(config)
conductor.start()

// zome functions can be called using the following, assuming the vars are defined with valid values
const callResult = conductor.call(aliceName, 'zome', 'function', {params: 'go here'})

// the same could be accomplished using the following, makeCaller is for convenience
const alice = conductor.makeCaller(aliceName)
const altCallResult = alice.call('zome', 'function', {params: 'go here'})

// get the actual agent_id for an instance, by passing an instance id
const aliceAgentId = conductor.agent_id(aliceInstanceId)

// stop all running instances
conductor.stop()
```