# hc_cluster_test

hc_cluster_test provides a VERY streamlined method for spinning up any (reasonable) number of Conductors, which can then have DNA Instances installed and real networking capabilities can be tested. 

## Background and Important Context

This one little library interacts with a very significant portion of the core functionalities across Holochain. It has to:
- spin up and shut down (empty) Conductors on demand
- listen for specific logs through the `stdout` of the Conductors
- Utilize a number of the `admin` level websocket functions for administrating a Conductor, in order to create test agents, install DNAs and start Instances
- Utilize `hc-web-client` for making the Websocket connections
- Call Zome functions in the Conductors
- Subscribe to "Signals" from the Conductors

If any of those things changes in a breaking way, the tests that run using `hc_cluster_test` are liable to fail. This is actually a very good way for us to have an overview of how the system is working as a whole.

There are a few gotcha areas in particular, that if those change, this library should change as well, they are:
- the Conductor TOML configuration format/properties -> edit in `spawn_conductors.ts`
- The log line "Starting interfaces..." which is used as a trigger log in `spawn_conductors.ts`
- The following websocket exposed admin functions, referenced in `conductor_handle.ts`
    - test/agent/add
    - admin/dna/install_from_file
    - admin/instance/add
    - admin/instance/start
    - admin/interface/add_instance
- A specific format for the Trace level Signals, found in the `onSignal` callback setter in `conductor_handle.ts`

## Environment Variables

The following environment variable needs to be set before using this,
to specify which Conductor binary to run.
```
EMULATION_HOLOCHAIN_BIN_PATH=/Path/to/holochain/binary
```

## API and Example Usage

The most useful class to know about is `ConductorCluster`. Create a group of conductors, stored in an array on the `conductors` property of a `ConductorCluster` instance. Here is some sample code:

```javascript
  const dnaPath = path.join(__dirname, '..', 'dist/app_spec.dna.json')
  const instanceId = 'test-1'

  // just creates the instance
  const cluster = new ConductorCluster(numConductors, { debugging: true })
  // spawns the conductors and connects
  // to their newly opened websocket connections
  await cluster.initialize()
  // install the DNA and create an instance
  // with the test agent already in each Conductor
  await cluster.batch(conductor => conductor.createDnaInstance(instanceId, dnaPath))

  // call a Zome function in a particular Conductor/DNA Instance
  const result = await cluster.conductors[0].callZome(instanceId, 'blog', 'create_post')({
    content: 'hi',
    in_reply_to: null,
  })

  await cluster.shutdown()
```

To compile it from Typescript, just run:
`npm install`
`./node_modules/.bin/tsc`

Then use a require statement like:
```javascript
const ConductorCluster = require('./hc_cluster_test').default
```