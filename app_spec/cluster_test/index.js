const path = require('path')
const ConductorCluster = require('../../hc_cluster_test').default

process.on('unhandledRejection', error => {
  console.log('unhandledRejection', error.message);
})

const waitForHoldSignals = (count, cb) => {
  let countHolding = 0
  return (signal) => {
    console.log('signal', signal.action_type, signal.data.header && signal.data.header.entry_type)
    if (signal.action_type === "Hold" && signal.data.header.entry_type !== 'AgentId') {
      countHolding++
      if (countHolding === count) cb()
    }
  }
}

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

  // wait a maximum of 5 seconds for
  // all the expected signals to arrive
  // otherwise consider it a timed out failure
  setTimeout(async () => {
    if (!enteringShutdown) {
      enteringShutdown = true
      cluster.shutdown().finally(() => {
        process.exit(1) // failure status code
      })
    }
  }, 120000)

  const proceedWithFourthNode = async () => {
    // shut down the SECOND conductor
    await cluster.conductors[1].shutdown()
    // add a new conductor
    const fourthConductor = await cluster.addConductor()
    // set up that conductor with the same DNA
    await createDnaInstance(fourthConductor)

    fourthConductor.onSignal(waitForHoldSignals(10, () => {
      console.log('got dare')
    }))
  }

  const proceedWithThirdNode = async () => {
    // shut down the FIRST conductor
    await cluster.conductors[0].shutdown()
    // add a new conductor
    const thirdConductor = await cluster.addConductor()
    // set up that conductor with the same DNA
    await createDnaInstance(thirdConductor)

    thirdConductor.onSignal(waitForHoldSignals(10, proceedWithFourthNode))
    thirdConductor.callZome(instanceId, 'chat', 'create_stream')({
      name: 'streamtwo',
      description: '',
      initial_members: []
    })
  }

  // in node 2, wait for holding of all relevant entries
  cluster.conductors[1].onSignal(waitForHoldSignals(5, proceedWithThirdNode))

  // calling this will trigger a flurry of actions/signals
  // including the Hold actions related to the Commits this function
  // invokes
  await cluster.conductors[0].callZome(instanceId, 'chat', 'create_stream')({
    name: 'streamname',
    description: '',
    initial_members: []
  })
}

bugTest()
