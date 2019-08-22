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

const commonConductorConfig = {
  instances: {
    app: dna,
  },
}

const orchestratorSimple = new Orchestrator({
  conductors: {
    alice: commonConductorConfig,
    bob: commonConductorConfig,
    // carol: commonConductorConfig,
  },
  debugLog: false,
  executor: tapeExecutor(require('tape')),
  middleware: backwardCompatibilityMiddleware,
})


const runSimpleTests = async () => {
  const alice = await spawnConductor('alice', 3000)
  await orchestratorSimple.registerConductor({name: 'alice', url: 'http://0.0.0.0:3000'})
  const bob = await spawnConductor('bob', 4000)
  await orchestratorSimple.registerConductor({name: 'bob', url: 'http://0.0.0.0:4000'})
  // const carol = await spawnConductor('carol', 5000)
  // await orchestratorSimple.registerConductor({name: 'carol', url: 'http://0.0.0.0:5000'})

  const delay = ms => new Promise(resolve => setTimeout(resolve, ms))
  console.log("Waiting for conductors to settle...")
  await delay(5000)
  console.log("Ok, starting tests!")

  await orchestratorSimple.run()
  alice.kill()
  bob.kill()
  // carol.kill()
}

const run = async () => {
  orchestratorSimple.registerScenario("call the commit entry function", async (s, t, {alice}) => {
    let result = await alice.app.call("simple", "commit_entry", {"content": "some content"})
    console.log(result)
    t.equal(result.Ok.length, 46)
  })
  await runSimpleTests()
  process.exit()
}

run()