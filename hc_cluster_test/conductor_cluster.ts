import spawnConductors from './spawn_conductors'
import ConductorHandle from './conductor_handle'

interface ClusterOptions {
  debugging: boolean,
  adminPortStart: number,
  instancePortStart: number
}

const defaultOptions: ClusterOptions = {
  debugging: false,
  adminPortStart: 3000,
  instancePortStart: 4000
}

export default class ConductorCluster {
  numConductors: number
  options: ClusterOptions
  conductors: Array<ConductorHandle>

  constructor(numConductors: number, options = defaultOptions) {
    this.numConductors = numConductors
    this.options = options
  }

  async initialize() {
    const { debugging, adminPortStart, instancePortStart } = this.options
    const conductorsArray = await spawnConductors(this.numConductors, debugging, adminPortStart, instancePortStart)
    console.log('spawnConductors completed')
    this.conductors = conductorsArray.map(conductorInfo => new ConductorHandle(conductorInfo))
    return this.batch(conductor => conductor.initialize())
  }

  async addConductor() {
    const { debugging, adminPortStart, instancePortStart } = this.options
    const indexStart = this.conductors.length
    const conductorArray = await spawnConductors(1, debugging, adminPortStart, instancePortStart, indexStart)
    const conductorInfo = conductorArray[0]
    const conductor = new ConductorHandle(conductorInfo)
    this.conductors.push(conductor)
    await conductor.initialize()
    return conductor
  }

  batch(fn: (conductor: ConductorHandle) => any) {
    return Promise.all(this.conductors.map(fn))
  }

  shutdown() {
    return this.batch(conductor => conductor.shutdown())
  }
}
