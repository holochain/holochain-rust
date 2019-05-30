import spawnConductors from './spawn_conductors'
import ConductorHandle from './conductor_handle'

interface ClusterOptions {
  debugging: boolean
}

export default class ConductorCluster {
  numConductors: number
  options: ClusterOptions
  conductors: Array<ConductorHandle>

  constructor(numConductors: number, options: ClusterOptions = { debugging: false }) {
    this.numConductors = numConductors
    this.options = options
  }

  async initialize() {
    const conductorsArray = await spawnConductors(this.numConductors, this.options.debugging)
    console.log('spawnConductors completed')
    this.conductors = conductorsArray.map(conductorInfo => new ConductorHandle(conductorInfo))
    return this.batch(conductor => conductor.initialize())
  }

  batch(fn: (conductor: ConductorHandle) => any) {
    return Promise.all(this.conductors.map(fn))
  }

  shutdown() {
    return this.batch(conductor => conductor.shutdown())
  }
}
