import { Config } from '@holochain/tryorama'
import * as R from 'ramda'
import { configCommon } from './config'

const dna = Config.dna('passthrough-dna.dna.json', 'passthrough')
const trace = R.tap(x => console.log('{T}', x))

const config = Config.gen({app: dna}, configCommon)
const delay = ms => new Promise(r => setTimeout(r, ms))

module.exports = (scenario) => {
    scenario('gossip transfers entries', async (s, t) => {
        const { alice, bob, carol } = await s.players({ alice: config, bob: config, carol: config }, false)

        await alice.spawn()
//        await bob.spawn()

        let result = await alice.call('app','main', 'commit_entry', { content: "alice entry" })
        console.log("RESULTS:", result)
        t.equal(Boolean(result.Ok), true)
        const entryHash = result.Ok
        console.log("entryHash:", entryHash)


        await delay(2000)

//        await alice.kill()
        await bob.spawn()
        await delay(2000)
        result = await bob.call('app', 'main', 'get_entry', {address: entryHash})
        console.log("RESULTS2:", result)
        t.equal(Boolean(result.Ok), true)

        await alice.kill()
        await bob.kill()
        await delay(2000)
        await bob.spawn()
        await carol.spawn()
        await delay(10000)

        result = await carol.call('app', 'main', 'get_entry', {address: entryHash})
        console.log("RESULTS:", result)
        t.equal(Boolean(result.Ok), true)

    })
}
