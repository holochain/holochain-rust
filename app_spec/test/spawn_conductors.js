const {spawn} = require('child_process')
const fs = require('fs')
const os = require('os')
const path = require('path')

const adminInterfaceId = "admin-interface"
const holochainBin = process.env.EMULATION_HOLOCHAIN_BIN_PATH || 'holochain'


const genConfig = async (debug, index) => {

  const tmpPath = fs.mkdtempSync(path.join(os.tmpdir(), 'n3h-test-conductors-'))
  const n3hPath = path.join(tmpPath, 'n3h-storage')
  fs.mkdirSync(n3hPath)
  const configPath = path.join(tmpPath, `conductor-config.toml`)

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

  const adminUrl = `http://0.0.0.0:${adminPort}`
  return { configPath, adminUrl }
}

const spawnConductor = (name, configPath) => {

  console.info(`Spawning process for conductor ${name}...`)
  const handle = spawn(holochainBin, ['-c', configPath])

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

module.exports = {
    genConfig,
    spawnConductor,
}