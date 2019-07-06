import * as child_process from 'child_process'
import * as fs from 'fs'
import * as os from 'os'
import * as path from 'path'
import { adminInterfaceId, dnaInterfaceId } from './config'

const holochainBin = process.env.EMULATION_HOLOCHAIN_BIN_PATH

interface ConductorConfig {
  configPath: string
  adminPort: number
}

interface ConductorDetails {
  adminPort: number
  instancePort: number
  handle: any
}

export const genConfig = async (debug: boolean, index: number): Promise<ConductorConfig> => {

  const tmpPath = fs.mkdtempSync(path.join(os.tmpdir(), 'n3h-test-conductors-'))
  const n3hPath = path.join(tmpPath, 'n3h-storage')
  fs.mkdirSync(n3hPath)
  const configPath = path.join(tmpPath, `empty-conductor-${index}.toml`)

  const adminPort = 3000 + index

  const config = `
persistence_dir = "${tmpPath}"

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

[logger]
type = "debug"
${debug ? '' : '[[logger.rules.rules]]'}
${debug ? '' : 'exclude = true'}
${debug ? '': 'pattern = "^debug"'}

[network]
type = "n3h"
n3h_log_level = "${debug ? 'i' : 'e'}"
bootstrap_nodes = []
n3h_mode = "REAL"
n3h_persistence_path = "${n3hPath}"
    `

  fs.writeFileSync(configPath, config)
  return { configPath, adminPort }
}

export const spawnConductor = (name: string, configPath: string): Promise<ConductorDetails> => {


  console.info(`Spawning process for conductor ${name}...`)
  const handle = child_process.spawn(holochainBin, ['-c', configPath])

  handle.stdout.on('data', data => console.log(`[C ${name}]`, data.toString('utf8')))
  handle.stderr.on('data', data => console.error(`!C ${name}!`, data.toString('utf8')))
  handle.on('close', code => console.log(`conductor '${name}' exited with code`, code))

  console.info(`Conductor '${name}' process spawning successful`)

  return new Promise((resolve) => {
    handle.stdout.on('data', data => {
      // wait for the logs to convey that the interfaces have started
      // because the consumer of this function needs those interfaces
      // to be started so that it can initiate, and form,
      // the websocket connections
      if (data.toString('utf8').indexOf('Starting interfaces...') >= 0) {
        resolve(handle)
      }
    })
  })
}
