import { Config } from '@holochain/try-o-rama'
import * as R from 'ramda'

const dna = Config.dna('passthrough-dna.dna.json', 'passthrough')

/** Generates a bunch of identical conductor configs with multiple identical instances */
const configBatchSimple = (numConductors, numInstances) => {
    const conductor = R.pipe(
        R.map(n => [`${n}`, dna]),
        R.fromPairs,
        x => ({ instances: x }),
    )(R.range(0, numInstances))
    return R.repeat(conductor, numConductors)
}

module.exports = (scenario, N, M) => {
    scenario('telephone game: const entry -> entry', async (s, t) => {
        const players = R.values(await s.players(configBatchSimple(N, M), false))

        console.log("### Sequenced 'telephone game': links from constant base entry to one entry per agent")
        console.log("Initializing first node")
        await players[0].spawn()
        const instance1 = await players[0]._instances[0]
        const baseHash = await instance1.call('main', 'commit_entry', { content: 'base' }).then(r => r.Ok)

        for(let i=1;i<N-1;i++) {
            console.log(`Iteration ${i} (${i-1} -> ${i})`)
            console.log("Spawning new node")
            await players[i].spawn()

            const instance1 = await players[i-1]._instances[0]
            const instance2 = await players[i]._instances[0]

            console.log("Committing entry")
            const entryHash = await instance1.call('main', 'commit_entry', { content: 'player'+(i-1) }).then(r => r.Ok)
            console.log("Committing link")
            const link_result = await instance1.call('main', 'link_entries', { base: baseHash, target: entryHash })
            console.log(`link result: ${link_result}`)
            t.ok(link_result)


            console.log("Awaiting consistency")
            await s.consistency()

            console.log(`Trying to get base from node ${i}`)
            const base = await instance2.call('main', 'get_entry', {address: baseHash})
            t.ok(base)
            t.deepEqual(base.Ok, { App: [ 'generic_entry', 'base' ] })
            console.log("Trying to get all previous links on new node")
            const links = await instance2.call('main', 'get_links', { base: baseHash })
            t.ok(links)
            t.equal(links.Ok.links.length, i)

            console.log("Killing old node")
            players[i-1].kill()
        }
    })

    scenario('telephone game: const entry -> agent_id', async (s, t) => {
        const players = R.values(await s.players(configBatchSimple(N, M), false))

        console.log("### Sequenced 'telephone game': links from constant base entry each agent's agent entry")
        console.log("Initializing first node")
        await players[0].spawn()
        const instance1 = await players[0]._instances[0]
        const baseHash = await instance1.call('main', 'commit_entry', { content: 'base' }).then(r => r.Ok)

        for(let i=1;i<N-1;i++) {
            console.log(`Iteration ${i} (${i-1} -> ${i})`)
            console.log("Spawning new node")
            await players[i].spawn()

            const instance1 = await players[i-1]._instances[0]
            const instance2 = await players[i]._instances[0]

            console.log("Committing link")
            const link_result = await instance1.call('main', 'link_entries', { base: baseHash, target: instance1.agentAddress })
            console.log(`link result: ${link_result}`)
            t.ok(link_result)


            console.log("Awaiting consistency")
            await s.consistency()

            console.log(`Trying to get base from node ${i}`)
            const base = await instance2.call('main', 'get_entry', {address: baseHash})
            t.ok(base)

            console.log("Trying to get all previous links on new node")
            const links = await instance2.call('main', 'get_links', { base: baseHash })
            t.ok(links)
            t.equal(links.Ok.links.length, i)

            console.log("Killing old node")
            players[i-1].kill()
        }
    })
}