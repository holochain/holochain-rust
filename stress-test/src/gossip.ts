import { Config } from '@holochain/tryorama'
import * as R from 'ramda'
import { configCommon } from './config'

const dna = Config.dna('passthrough-dna.dna.json', 'passthrough')
const trace = R.tap(x => console.log('{T}', x))

const path = require('path')
const id = 'app'

const instanceConfig = ({playerName, uuid, configDir}) => [
    {
        id: id,
        dna: dna,
        agent: {
            name: `${playerName}::${id}::${uuid}`,
            id: id,
            keystore_file: '[UNUSED]',
            public_address: '[SHOULD BE REWRITTEN]',
            test_agent: true,
        },
        storage: {
            type: 'lmdb',
            path: path.join(configDir, id)
        }
    }
]

const config = Config.gen(instanceConfig, configCommon)
const delay = ms => new Promise(r => setTimeout(r, ms))

module.exports = (scenario) => {
    scenario('gossip transfers entries', async (s, t) => {
        const { alice, bob, carol } = await s.players({ alice: config, bob: config, carol: config }, false)

        // alice creates an entry
        await alice.spawn()
        let result = await alice.call(id, 'main', 'commit_entry', { content: "alice entry" })
        console.log("RESULTS:", result)
        t.equal(Boolean(result.Ok), true)
        const entryHash = result.Ok
        console.log("entryHash:", entryHash)

        // bob comes on line and starts holding that entry
        await bob.spawn()
        await s.consistency()
        result = await bob.call(id, 'main', 'get_entry', {address: entryHash})
        console.log("RESULTS2:", result)
        t.equal(Boolean(result.Ok), true)

        // alice and bob both go off line which means that the sim2h server should
        // clear all it's knowledge of the space
        await alice.kill()
        await bob.kill()
        await delay(2000)

        // bob comes back and carol comes on for the first time
        await bob.spawn()
        await carol.spawn()
        await delay(5000)

        // carol can get alices entry
        result = await carol.call(id, 'main', 'get_entry', {address: entryHash})
        console.log("RESULTS:", result)
        t.equal(Boolean(result.Ok), true)

    })
}
