const { spawnConductors } = require('./spawn_conductors')
const { ConductorHandle } = require('./conductor_handle')

class ConductorCluster {
  constructor(numConductors, options = { debugging: false }) {
    this.numConductors = numConductors
    this.options = options
  }

  async initialize() {
    const conductorsArray = await spawnConductors(this.numConductors, this.options.debugging)
    console.log('spawnConductors completed')
    this.conductors = conductorsArray.map(conductorInfo => new ConductorHandle(conductorInfo))
    return Promise.all(this.conductors.map(conductor => conductor.initialize()))
  }

  batch(fn) {
    return Promise.all(this.conductors.map(fn))
  }

  shutdown() {
    return this.batch(c => c.shutdown())
  }
}
module.exports.ConductorCluster = ConductorCluster