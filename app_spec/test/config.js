const _ = require('lodash')
const path = require('path')
const { Config } = require('@holochain/tryorama')

const dnaPath = path.join(__dirname, '../dist/app_spec.dna.json')
const dna = Config.dna(dnaPath, 'app-spec')
const dna2 = Config.dna(dnaPath, 'app-spec', { uuid: 'altered-dna' })

const commonConfig = {
  logger: {
    type: 'debug',
    rules: {
      rules: [
        {
          exclude: true,
          pattern: '.*parity.*'
        },
        {
          exclude: true,
          pattern: '.*mio.*'
        },
        {
          exclude: true,
          pattern: '.*tokio.*'
        },
        {
          exclude: true,
          pattern: '.*hyper.*'
        },
        {
          exclude: true,
          pattern: '.*rusoto_core.*'
        },
        {
          exclude: true,
          pattern: '.*want.*'
        },
        {
          exclude: true,
          pattern: '.*rpc.*'
        }
      ]
    },
    state_dump: true
  },
  network: {
    type: 'sim2h',
    sim2h_url: 'ws://localhost:9000'
  }
}

module.exports = {
  one: Config.gen({
      app: dna
    },
    commonConfig
  ),
  two: Config.gen({
      app1: dna,
      app2: dna2
    },
    {
      bridges: [
        Config.bridge('test-bridge', 'app1', 'app2')
      ],
      ...commonConfig
    }),
}


// Replace the real hachiko waiter with a simple delay.
// i.e. makes `await s.consistency()` delay a certain number of milliseconds
// rather than actually waiting for consistency
const dumbWaiterMiddleware = interval => (run, f) => run(s =>
  f(Object.assign({}, s, {
    consistency: () => new Promise(resolve => {
      console.log(`dumbWaiter is waiting ${interval}ms...`)
      setTimeout(resolve, interval)
    })
  }))
)
