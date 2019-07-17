const path = require('path')
const ConductorCluster = require('../../hc_cluster_test').default

process.on('unhandledRejection', error => {
  console.log('unhandledRejection', error.message);
})

const scenarioTest = async (numConductors = 2, debugging = false) => {
  const dnaPath = path.join(__dirname, '..', 'dist/app_spec.dna.json')
  const instanceId = 'test-1'

  // just creates the instance
  const cluster = new ConductorCluster(numConductors, { debugging })
  // spawns the conductors and connects
  // to their newly opened websocket connections
  await cluster.initialize()
  // install the DNA and create an instance
  // with the test agent already in each Conductor
  await cluster.batch(conductor => conductor.createDnaInstance(instanceId, dnaPath))

  let enteringShutdown = false

  // wait a maximum of 5 seconds for
  // all the expected signals to arrive
  // otherwise consider it a timed out failure
  setTimeout(async () => {
    if (!enteringShutdown) {
      enteringShutdown = true
      console.log('after 10 seconds, all nodes should be holding all entries and all links')
      console.log(`There are only ${countHolding} after 5 seconds.`)
      // done this way because cluster.shutdown() was causing
      // errors on CircleCI
      if (process.env.CI) {
        process.exit(1) // failure status code
      } else {
        cluster.shutdown().finally(() => {
          process.exit(1) // failure status code
        })
      }
    }
  }, 10000)

  let countHolding = 0
  cluster.batch(conductor => conductor.onSignal(async signal => {
    if (signal.action_type === "Hold") {
      countHolding++
      console.log("Nodes holding so far:" + countHolding)
    }
    // 2 Holds for each nodes ... one for the App entry and one for a LinkAdd entry
    if (countHolding === numConductors * 2 && !enteringShutdown) {
      enteringShutdown = true
      console.log('All nodes are successfully HOLDing all entries they should be.')

      // done this way because cluster.shutdown() was causing
      // errors on CircleCI
      if (process.env.CI) {
        process.exit() // success status code
      } else {
        cluster.shutdown().finally(() => {
          process.exit() // success status code
        })
      }
    }
  }))

  // calling this will trigger a flurry of actions/signals
  // including the Hold actions related to the Commits this function
  // invokes
  cluster.conductors[0].callZome(instanceId, 'blog', 'create_post')({
    content: 'hi',
    in_reply_to: null,
  })
}

// first argument is the number of nodes to run
const optionalNumber = process.argv[2]

// second argument is whether to show debugging logs or not
const optionalDebugging = process.argv[3]

scenarioTest(optionalNumber, optionalDebugging)
