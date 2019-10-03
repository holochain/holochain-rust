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


  callSyncMiddleware: (run, f) => run(s => {
    const s_ = Object.assign({}, s, {
      players: async (...a) => {
        const players = await s.players(...a)
        const players_ = _.mapValues(
          players,
          api => Object.assign(api, {
            callSync: async (...b) => {
              const result = await api.call(...b)
              await s.consistency()
              return result
            }
          })
        )
        return players_
      }
    })
    return f(s_)
  })

}
