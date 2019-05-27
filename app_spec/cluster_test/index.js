const path = require('path')
const { ConductorCluster } = require('../../hc_cluster_test')

process.on('unhandledRejection', error => {
  console.log('unhandledRejection', error.message);
})

const scenarioTest = async (numConductors = 2, debugging = false) => {
  const dnaPath = path.join(__dirname, '..', 'dist/app_spec.dna.json')
  console.log(dnaPath)
  const instanceId = 'test-1'

  const cluster = new ConductorCluster(numConductors, { debugging })
  await cluster.initialize()
  await cluster.batch(conductor => conductor.createDnaInstance(instanceId, dnaPath))

  let enteringShutdown = false
  let countHolding = 0
  cluster.batch((c) => c.onSignal(async signal => {
    if (signal.action_type === "Hold") {
      countHolding++
    }
    // 2 Holds for each nodes ... one for the App entry and one for a LinkAdd entry
    if (countHolding === numConductors * 2 && !enteringShutdown) {
      enteringShutdown = true
      console.log('All nodes are successfully HOLDing all entries they should be.')
      await cluster.shutdown()
      process.exit() // success status code
    }
  }))

  // calling this will trigger a flurry of actions/signals
  // including the Hold actions related to the Commits this function
  // invokes
  await cluster.conductors[0].callZome(instanceId, 'blog', 'create_post')({
    content: 'hi',
    in_reply_to: null,
  })

  setTimeout(async () => {
    console.log('after 5 seconds, all nodes should be holding all entries and all links')
    console.log(`There are only ${countHolding} after 5 seconds.`)
    await cluster.shutdown()
    process.exit(1) // failure status code
  }, 5000)
}

// first argument is the number of nodes to run
const optionalNumber = process.argv[2]

// third argument is
const optionalDebugging = process.argv[3]

scenarioTest(optionalNumber, optionalDebugging)