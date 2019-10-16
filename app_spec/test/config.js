const _ = require('lodash')
const path = require('path')
const { Config } = require('@holochain/try-o-rama')

const dnaPath = path.join(__dirname, "../dist/app_spec.dna.json")
const dna = Config.dna(dnaPath, 'app-spec')
const dna2 = Config.dna(dnaPath, 'app-spec', {uuid: 'altered-dna'})


module.exports = {
  one: {
    instances: {
      app: dna,
    },
  },
  two: ({
    instances: {
      app1: dna,
      app2: dna2,
    },
    bridges: [
      Config.bridge('test-bridge', 'app1', 'app2')
    ]
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
