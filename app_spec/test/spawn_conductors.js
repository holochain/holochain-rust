const child_process = require('child_process')
const fs = require('fs')
const os = require('os')
const path = require('path')

const genConfig = (adminPort, debugging, tmpPath, n3hPath) => {
    const config = `
persistence_dir = "${tmpPath}"

agents = []
dnas = []
instances = []

[signals]
consistency = true
trace = true

[[interfaces]]
admin = true
id = "admin interface"
instances = []
    [interfaces.driver]
    type = "websocket"
    port = ${adminPort}

[logger]
type = "debug"
${debugging ? '' : '[[logger.rules.rules]]'}
${debugging ? '' : 'exclude = true'}
${debugging ? '': 'pattern = "^debug"'}
state_dump = true

[network]
type="n3h"
n3h_log_level = "${debugging ? 'i' : 'e'}"
bootstrap_nodes = []
n3h_mode = "REAL"
n3h_persistence_path = "${n3hPath}"
    `

    return config
}

module.exports = (name, port) => {
    let holochainBin = ""
    if(process.env.EMULATION_HOLOCHAIN_BIN_PATH) {
        holochainBin = process.env.EMULATION_HOLOCHAIN_BIN_PATH
    } else {
        holochainBin = "holochain"
    }

    const tmpPath = fs.mkdtempSync(path.join(os.tmpdir(), 'n3h-test-conductors-'))
    const n3hPath = path.join(tmpPath, 'n3h-storage')
    fs.mkdirSync(n3hPath)
    const configPath = path.join(tmpPath, `empty-conductor-${name}.toml`)

    const config = genConfig(port, true, tmpPath, n3hPath)

    fs.writeFileSync(configPath, config)

    console.info(`Spawning conductor ${name} process...`)
    console.info(`holochain binary = ${holochainBin}`)
    console.info(`config path      = ${configPath}`)
    const handle = child_process.spawn(holochainBin, ['-c', configPath])

    handle.stdout.on('data', data => console.log(`[C '${name}']`, data.toString('utf8')))
    handle.stderr.on('data', data => console.error(`!C '${name}'!`, data.toString('utf8')))
    handle.on('close', code => console.log(`conductor '${name}' exited with code`, code))

    return new Promise((resolve) => {
        handle.stdout.on('data', data => {
            // wait for the logs to convey that the interfaces have started
            // because the consumer of this function needs those interfaces
            // to be started so that it can initiate, and form,
            // the websocket connections
            if (data.toString('utf8').indexOf('Starting interfaces...') >= 0) {
                console.info(`Conductor '${name}' process spawning successful`)
                resolve(handle)
            }
        })
    })
}
