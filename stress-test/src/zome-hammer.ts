import { Config } from '@holochain/tryorama'
import * as R from 'ramda'

const dna = Config.dna('passthrough-dna.dna.json', 'passthrough')
const trace = R.tap(x => console.log('{T}', x))

const config = Config.gen({app: dna})

module.exports = (scenario, hammerCount) => {
    scenario('hammer on a zome call', async (s, t) => {
        const { alice } = await s.players({ alice: config }, true)

        let calls = []
        for (let i = 0; i < hammerCount; i++) {
            calls.push(alice.call('app','main', 'commit_entry', { content: trace(`call-${i}`) }))
        }

        const results = await Promise.all(calls)

        t.equal(results.length, hammerCount)

    })
}
