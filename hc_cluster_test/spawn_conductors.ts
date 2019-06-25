import * as child_process from 'child_process'
import * as fs from 'fs'
import * as os from 'os'
import * as path from 'path'
import { adminInterfaceId, dnaInterfaceId } from './config'

const holochainBin = process.env.EMULATION_HOLOCHAIN_BIN_PATH

interface ConductorConfigOptions {
  index: number,
  debugging: boolean,
  adminPortStart: number,
  instancePortStart: number,
  tmpPath: string,
  n3hPath: string
}

interface ConductorConfig {
  config: string
  adminPort: number
  instancePort: number
}

interface ConductorDetails {
  adminPort: number
  instancePort: number
  handle: any
}

const genConfig = (options: ConductorConfigOptions): ConductorConfig => {

  const adminPort = options.adminPortStart + options.index
  const instancePort = options.instancePortStart + options.index

  const config = `
persistence_dir = "${options.tmpPath}"

agents = []
dnas = []
instances = []

[signals]
consistency = false
trace = true

[[interfaces]]
admin = true
id = "${adminInterfaceId}"
instances = []
    [interfaces.driver]
    type = "websocket"
    port = ${adminPort}

[[interfaces]]
admin = true
id = "${dnaInterfaceId}"
instances = []
    [interfaces.driver]
    type = "websocket"
    port = ${instancePort}

[logger]
type = "debug"
${options.debugging ? '' : '[[logger.rules.rules]]'}
${options.debugging ? '' : 'exclude = true'}
${options.debugging ? '': 'pattern = "^debug"'}

[network]
n3h_log_level = "${options.debugging ? 'i' : 'e'}"
bootstrap_nodes = []
n3h_mode = "REAL"
n3h_persistence_path = "${options.n3hPath}"
    `

  return { config, adminPort, instancePort }
}

const spawnConductor = (i: number, debugging: boolean, adminPortStart: number, instancePortStart: number): Promise<ConductorDetails> => {
  const tmpPath = fs.mkdtempSync(path.join(os.tmpdir(), 'n3h-test-conductors-'))
  const n3hPath = path.join(tmpPath, 'n3h-storage')
  fs.mkdirSync(n3hPath)
  const configPath = path.join(tmpPath, `empty-conductor-${i}.toml`)

  const { config, adminPort, instancePort } = genConfig({
    index: i,
    debugging,
    adminPortStart,
    instancePortStart,
    tmpPath,
    n3hPath
  })

  fs.writeFileSync(configPath, config)

  console.info(`Spawning conductor${i} process...`)
  const handle = child_process.spawn(holochainBin, ['-c', configPath])

  handle.stdout.on('data', data => console.log(`[C${i}]`, data.toString('utf8')))
  handle.stderr.on('data', data => console.error(`!C${i}!`, data.toString('utf8')))
  handle.on('close', code => console.log(`conductor ${i} exited with code`, code))

  console.info(`Conductor${i} process spawning successful`)

  return new Promise((resolve) => {
    handle.stdout.on('data', data => {
      // wait for the logs to convey that the interfaces have started
      // because the consumer of this function needs those interfaces
      // to be started so that it can initiate, and form,
      // the websocket connections
      if (data.toString('utf8').indexOf('Starting interfaces...') >= 0) {
        resolve({
          adminPort,
          instancePort,
          handle
        })
      }
    })
  })
}

const spawnConductors = async (numberOfConductors: number, debugging: boolean, adminPortStart: number, instancePortStart: number, indexStart: number = 0): Promise<Array<ConductorDetails>> => {
  const promises = []

  // start the first conductor and
  // wait for it, because it sets up n3h
  const firstConductor = await spawnConductor(indexStart + 0, debugging, adminPortStart, instancePortStart)
  promises.push(firstConductor)

  for (let i = 1; i < numberOfConductors; i++) {
    promises.push(spawnConductor(indexStart + i, debugging, adminPortStart, instancePortStart))
  }
  return Promise.all(promises)
}
export default spawnConductors
