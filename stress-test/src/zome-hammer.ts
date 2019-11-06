import { Config } from '@holochain/try-o-rama'
import * as R from 'ramda'
import { ScenarioApi } from '@holochain/try-o-rama/lib/api'
import { Player } from '@holochain/try-o-rama/lib/player'
import { ConductorConfig } from '@holochain/try-o-rama/lib/types'

const dna = Config.dna('passthrough-dna.dna.json', 'passthrough')
const trace = R.tap(x => console.log('{T}', x))

module.exports = (scenario, hammerCount) => {
    scenario('hammer on a zome call', async (s, t) => {
        const { alice } = await s.players({ alice: {instances: {app: dna}} }, true)

        let calls = []
        for (let i = 0; i < hammerCount; i++) {
            calls.push(alice.call('app','main', 'commit_entry', { content: trace(`call-${i}`) }))
        }

        const results = await Promise.all(calls)

        t.equal(results.length, hammerCount)

    })
}
