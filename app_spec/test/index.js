const path = require('path')
const tape = require('tape')

const { Orchestrator, tapeExecutor, backwardCompatibilityMiddleware } = require('@holochain/try-o-rama')
const spawnConductor = require('./spawn_conductors')

// This constant serves as a check that we haven't accidentally disabled scenario tests.
// Try to keep this number as close as possible to the actual number of scenario tests.
// (But never over)
const MIN_EXPECTED_SCENARIOS = 45

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

const orchestratorValidateAgent = new Orchestrator({
  conductors: {
    valid_agent: { instances: { app: dna } },
    reject_agent: { instances: { app: dna } },
  },
  debugLog: false,
  executor: tapeExecutor(require('tape')),
  middleware: backwardCompatibilityMiddleware,
})

const registerAllScenarios = () => {
  // NB: all scenarios must be registered before any orchestrator is run. Tape will fail to register its
  // test cases if there is any Promise awaiting in between test declarations.
  let numRegistered = 0

  const registerer = orchestrator => {
    const f = (...info) => {
      numRegistered += 1
      return orchestrator.registerScenario(...info)
    }
    f.only = (...info) => {
      numRegistered += 1
      return orchestrator.registerScenario.only(...info)
    }
    return f
  }

  require('./regressions')(registerer(orchestratorSimple))
  require('./files/test')(registerer(orchestratorSimple))
  require('./files/entry')(registerer(orchestratorSimple))
  require('./files/links')(registerer(orchestratorSimple))
  require('./files/memo')(registerer(orchestratorSimple))
  require('./files/crypto')(registerer(orchestratorSimple))
  // require('./multi-dna')(registerer(orchestratorMultiDna))
  require('./validate-agent-test')(registerer(orchestratorValidateAgent))

  return numRegistered
}


const runSimpleTests = async () => {
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

}

const runMultiDnaTests = async () => {
  // Multi instance tests where n3h is the network connecting them currently fails with the 2nd instance
  // waiting for and not receiving the agent entry of the first one.
  // I believe this is due to n3h not sending a peer connected message for a local instance
  // and core has not implented the authoring list yet...
  const conductor = await spawnConductor('conductor', 6000)
  await orchestratorMultiDna.registerConductor({name: 'conductor', url: 'http://0.0.0.0:6000'})
  await orchestratorMultiDna.run()
  conductor.kill()
}

const runValidationTests = async () => {
  const valid_agent = await spawnConductor('valid_agent', 3000)
  await orchestratorValidateAgent.registerConductor({name: 'valid_agent', url: 'http://0.0.0.0:3000'})
  const reject_agent = await spawnConductor('reject_agent', 4000)
  await orchestratorValidateAgent.registerConductor({name: 'reject_agent', url: 'http://0.0.0.0:4000'})

  const delay = ms => new Promise(resolve => setTimeout(resolve, ms))
  console.log("Waiting for conductors to settle...")
  await delay(5000)
  console.log("Ok, starting tests!")

  await orchestratorValidateAgent.run()
  valid_agent.kill()
  reject_agent.kill()
}

const run = async () => {
  const num = registerAllScenarios()

  // Check to see that we haven't accidentally disabled a bunch of scenarios
  if (num < MIN_EXPECTED_SCENARIOS) {
    console.error(`Expected at least ${MIN_EXPECTED_SCENARIOS}, but only ${num} were registered!`)
    process.exit(1)
  } else {
    console.log(`Registered ${num} scenarios (at least ${MIN_EXPECTED_SCENARIOS} were expected)`)
  }

  await runSimpleTests()
  // await runMultiDnaTests()
  // await runValidationTests()
  process.exit()
}

run()