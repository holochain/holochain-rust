const path = require('path')
const ConductorCluster = require('../../hc_cluster_test').default

process.on('unhandledRejection', error => {
  console.log('unhandledRejection', error.message);
})

const bugTest = async () => {
  const dnaPath = path.join(__dirname, 'holochain-basic-chat.dna.json')
  const instanceId = 'test-1'

  // start with two
  const cluster = new ConductorCluster(2, { debugging: true, adminPortStart: 3000, instancePortStart: 4000 })

  const createDnaInstance = (conductor) => conductor.createDnaInstance(instanceId, dnaPath)

  // spawns the conductors and connects
  // to their newly opened websocket connections
  await cluster.initialize()
  // install the DNA and create an instance
  // with the test agent already in each Conductor
  await cluster.batch(createDnaInstance)

  let enteringShutdown = false

  let thirdConductor

  // wait a maximum of 5 seconds for
  // all the expected signals to arrive
  // otherwise consider it a timed out failure
  setTimeout(async () => {
    const result = await thirdConductor.callZome(instanceId, 'chat', 'get_all_public_streams')({})
    console.log('get_all_public_streams_result2', result)

    if (!enteringShutdown) {
      enteringShutdown = true
      console.log('after 30 seconds, third node should be holding all entries and all links from first node')
      cluster.shutdown().finally(() => {
        process.exit(1) // failure status code
      })
    }
  }, 120000)

  const proceedWithThirdNode = async () => {
    // shut down the FIRST conductor
    await cluster.conductors[0].shutdown()
    // add a new conductor
    thirdConductor = await cluster.addConductor()
    // set up that conductor with the same DNA
    await createDnaInstance(thirdConductor)

    thirdConductor.onSignal(async signal => {
      if (signal.action_type === "Hold") {
        console.log('new HOLD!', signal.data)
      }
    })

    const result = await thirdConductor.callZome(instanceId, 'chat', 'get_all_public_streams')({})
    console.log('get_all_public_streams_result', result)
  }

  let countHolding = 0
  cluster.conductors[1].onSignal(async signal => {
    if (signal.action_type === "Hold") {
      countHolding++
      console.log('old HOLD [',countHolding,']',signal.data)
      if (countHolding === 5) {
        proceedWithThirdNode()
      }
    }
  })

  // calling this will trigger a flurry of actions/signals
  // including the Hold actions related to the Commits this function
  // invokes
  const result = await cluster.conductors[0].callZome(instanceId, 'chat', 'create_stream')({
    name: 'streamname',
    description: '',
    initial_members: []
  })
  console.log('result123', result)
}

bugTest()
