const path = require('path')
const tape = require('tape')

const { Orchestrator, tapeExecutor, backwardCompatibilityMiddleware } = require('@holochain/try-o-rama')
const spawnConductor = require('./spawn_conductors')

process.on('unhandledRejection', error => {
  // Will print "unhandledRejection err is not defined"
  console.error('got unhandledRejection:', error);
});

const dnaPath = path.join(__dirname, "../dist/app_spec.dna.json")
const dna = Orchestrator.dna(dnaPath, 'app-spec')
const dna2 = Orchestrator.dna(dnaPath, 'app-spec', {uuid: 'altered-dna'})

const commonConductorConfig = {
  instances: {
    app: dna,
  },
}

const orchestratorSimple = new Orchestrator({
  conductors: {
    alice: commonConductorConfig,
    bob: commonConductorConfig,
    carol: commonConductorConfig,
  },
  debugLog: false,
  executor: tapeExecutor(require('tape')),
  middleware: backwardCompatibilityMiddleware,
})

const orchestratorMultiDna = new Orchestrator({
  conductors: {
    conductor: {
      instances: {
        app1: dna,
        app2: dna2,
      },
      bridges: [
        Orchestrator.bridge('test-bridge', 'app1', 'app2')
      ],
    }
  },
  debugLog: false,
  executor: tapeExecutor(require('tape')),
  middleware: backwardCompatibilityMiddleware,
  callbacksPort: 8888,
})

require('./regressions')(orchestratorSimple.registerScenario)
require('./test')(orchestratorSimple.registerScenario)
require('./multi-dna')(orchestratorMultiDna.registerScenario)
require('./validate-agent-test')(dnaPath)

const run = async () => {
  const alice = await spawnConductor('alice', 3000)
  await orchestratorSimple.registerConductor({name: 'alice', url: 'http://0.0.0.0:3000'})
  const bob = await spawnConductor('bob', 4000)
  await orchestratorSimple.registerConductor({name: 'bob', url: 'http://0.0.0.0:4000'})
  const carol = await spawnConductor('carol', 5000)
  await orchestratorSimple.registerConductor({name: 'carol', url: 'http://0.0.0.0:5000'})

  const delay = ms => new Promise(resolve => setTimeout(resolve, ms))
  console.log("Waiting for conductors to settle...")
  await delay(5000)
  console.log("Ok, starting tests!")

  await orchestratorSimple.run()
  alice.kill()
  bob.kill()
  carol.kill()

  // Multi instance tests where n3h is the network connecting them currently fails with the 2nd instance
  // waiting for and not receiving the agent entry of the first one.
  // I believe this is due to n3h not sending a peer connected message for a local instance
  // and core has not implented the authoring list yet...
  //const conductor = await spawnConductor('conductor', 6000)
  //await orchestratorMultiDna.registerConductor({name: 'conductor', url: 'http://0.0.0.0:6000'})
  //await orchestratorMultiDna.run()
  //conductor.kill()

  process.exit()
}

run()