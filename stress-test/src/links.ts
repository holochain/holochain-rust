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

// This is a customizable blue print for telephone game (aka chinese whispers)
// network/time topology setups.
// It cycles through all N agents in a way that at any given point only two agents
// are online. Before the next agent is spawned, the older one gets killed,
// such that every agent needs to first receive all previous data so it can gossip
// it to the next one.
//
// Callbacks:
// * init: gets called with the very first agent. Its return value will be saved through the whole loop
//         and passed to each call of the other two callbacks
// * preSpawn: before the next agent gets spawned, this callback is called with the older one to have it create new
//             entries/links while it is alone
// * postSpawn: after the next agent was spawned, this callback is called with the older one to have it create new
//              entries/links while the new agent is there
// * stepCheck:
const telephoneGame = async (s, t, N, players, functions) => {
    let {init, preSpawn, postSpawn, stepCheck} = functions
    console.log("##################################")
    console.log("### Starting 'telephone game'")
    console.log("##################################")
    console.log("Initializing first node")
    await players[0].spawn()
    const instance1 = await players[0]._instances[0]
    const baseHash = await init(instance1)

    for(let i=1;i<N-1;i++) {
        console.log("----------------------------------")
        console.log("##################################")
        console.log(`###Iteration ${i} (${i-1} -> ${i})`)
        console.log("##################################")
        console.log("----------------------------------")
        const instance1 = await players[i-1]._instances[0]

        console.log("##################################")
        console.log("### PRE SPAWN")
        console.log("##################################")
        await preSpawn(instance1, baseHash, i)
        await s.consistency()
        console.log("##################################")
        console.log(`### SPAWNING NODE ${i}`)
        console.log("##################################")
        await players[i].spawn()
        await s.consistency()
        const instance2 = await players[i]._instances[0]

        console.log("##################################")
        console.log("### POST SPAWN")
        console.log("##################################")
        await postSpawn(instance1, baseHash, i)
        await s.consistency()

        console.log("##################################")
        console.log("### STEP CHECK")
        console.log("##################################")
        await stepCheck(instance2, baseHash, i)

        console.log("Killing old node")
        players[i-1].kill()
    }
}

module.exports = (scenario, N, M) => {
    scenario('telephone game: const entry -> entry', async (s, t) => {
        const players = R.values(await s.players(configBatchSimple(N, M), false))
        const init = (instance) => {
            return instance.call('main', 'commit_entry', { content: 'base' }).then(r => r.Ok)
        }

        const preSpawn = () => {}

        const postSpawn = async (instance, baseHash, i) => {
            console.log("Committing entry")
            const entryHash = await instance.call('main', 'commit_entry', { content: 'player'+(i-1) }).then(r => r.Ok)
            console.log("Committing link")
            const link_result = await instance.call('main', 'link_entries', { base: baseHash, target: entryHash })
            console.log(`link result: ${link_result}`)
            t.ok(link_result)
        }

        const stepCheck = async (instance, baseHash, i) => {
            console.log(`Trying to get base from node ${i}`)
            const base = await instance.call('main', 'get_entry', {address: baseHash})
            t.ok(base)
            t.deepEqual(base.Ok, { App: [ 'generic_entry', 'base' ] })
            console.log("Trying to get all previous links on new node")
            const links = await instance.call('main', 'get_links', { base: baseHash })
            t.ok(links)
            t.equal(links.Ok.links.length, i)
        }

        await telephoneGame(s, t, N, players, {init, preSpawn, postSpawn, stepCheck})
    })

    scenario('telephone game: const entry -> agent_id', async (s, t) => {
        const players = R.values(await s.players(configBatchSimple(N, M), false))

        const init = (instance) => {
            return instance.call('main', 'commit_entry', { content: 'base' }).then(r => r.Ok)
        }

        const preSpawn = () => {}

        const postSpawn = async (instance, baseHash, i) => {
            console.log("Committing link")
            const link_result = await instance.call('main', 'link_entries_typed', {
                base: baseHash,
                target: instance.agentAddress,
                link_type: 'entry_2_agent'
            })
            console.log(`link result: ${link_result}`)
            t.ok(link_result)
        }

        const stepCheck = async (instance, baseHash, i) => {
            console.log(`Trying to get base from node ${i}`)
            const base = await instance.call('main', 'get_entry', {address: baseHash})
            t.ok(base)

            console.log("Trying to get all previous links on new node")
            const links = await instance.call('main', 'get_links', { base: baseHash })
            t.ok(links)
            t.equal(links.Ok.links.length, i)
        }

        await telephoneGame(s, t, N, players, {init, preSpawn, postSpawn, stepCheck})
    })

    scenario('telephone game: get all previously seen agent entries', async (s, t) => {
        const players = R.values(await s.players(configBatchSimple(N, M), false))

        const init = () => {
            return []
        }

        const preSpawn = (instance, all_agents) => { all_agents.push(instance.agentAddress) }

        const postSpawn = () => {}

        const stepCheck = async (instance, all_agents, i) => {
            for(let agent_id of all_agents) {
                const agent_entry = await instance.call('main', 'get_entry', {address: agent_id})
                console.log("AGENT ENTRY:", agent_entry)
                t.ok(agent_entry)
                t.ok(agent_entry.Ok)
            }
        }

        await telephoneGame(s, t, N, players, {init, preSpawn, postSpawn, stepCheck})
    })

    scenario('telephone game:  agent_id -> const entry', async (s, t) => {
        const players = R.values(await s.players(configBatchSimple(N, M), false))

        const init = (instance) => {
            return instance.agentAddress
        }

        const preSpawn = () => {}

        const postSpawn = async (instance, baseHash, i) => {
            console.log("Committing entry")
            const entryHash = await instance.call('main', 'commit_entry', { content: 'player'+(i-1) }).then(r => r.Ok)
            console.log("Committing link")
            const link_result = await instance.call('main', 'link_entries_typed', {
                base: baseHash,
                target: entryHash,
                link_type: 'agent_2_entry'
            })
            console.log(`link result: ${link_result}`)
            t.ok(link_result)
        }

        const stepCheck = async (instance, baseHash, i) => {
            console.log(`Trying to get base from node ${i}`)
            const base = await instance.call('main', 'get_entry', {address: baseHash})
            t.ok(base)

            console.log("Trying to get all previous links on new node")
            const links = await instance.call('main', 'get_links', { base: baseHash })
            t.ok(links)
            t.equal(links.Ok.links.length, i)
        }

        await telephoneGame(s, t, N, players, {init, preSpawn, postSpawn, stepCheck})
    })

    scenario('telephone game:  complex initial data', async (s, t) => {
        const players = R.values(await s.players(configBatchSimple(N, M), false))

        const init = async (instance) => {
            console.log("Committing entry")
            const aHash = await instance.call('main', 'commit_entry', { content: 'a' }).then(r => r.Ok)
            const bHash = await instance.call('main', 'commit_entry', { content: 'b' }).then(r => r.Ok)
            const link1 = await instance.call('main', 'link_entries_typed', {
                base: instance.agentAddress,
                target: aHash,
                link_type: 'agent_2_entry'
            })
            t.ok(link1)
            const link2 = await instance.call('main', 'link_entries_typed', {
                base: aHash,
                target: bHash,
                link_type: ''
            })
            t.ok(link2)
            const link3 = await instance.call('main', 'link_entries_typed', {
                base: bHash,
                target: instance.agentAddress,
                link_type: 'entry_2_agent'
            })
            t.ok(link3)
            return {agent: instance.agentAddress, aHash, bHash}
        }

        const preSpawn = () => {}

        const postSpawn = async () => {}

        const stepCheck = async (instance, initialData, i) => {
            let {agent, aHash, bHash} = initialData
            console.log(`Trying to get base from node ${i}`)
            const agent_entry = await instance.call('main', 'get_entry', {address: agent})
            t.ok(agent_entry)
            const a = await instance.call('main', 'get_entry', {address: aHash})
            t.ok(a)
            t.ok(a.Ok)
            const b = await instance.call('main', 'get_entry', {address: bHash})
            t.ok(b)
            t.ok(b.Ok)

            const agent_links = await instance.call('main', 'get_links', { base: agent })
            t.ok(agent_links)
            t.equal(agent_links.Ok.links.length, 1)
            t.equal(agent_links.Ok.links[0].address, aHash)

            const a_links = await instance.call('main', 'get_links', { base: aHash })
            t.ok(a_links)
            t.equal(a_links.Ok.links.length, 1)
            t.equal(a_links.Ok.links[0].address, bHash)

            const b_links = await instance.call('main', 'get_links', { base: bHash })
            t.ok(b_links)
            t.equal(b_links.Ok.links.length, 1)
            t.equal(b_links.Ok.links[0].address, agent)
        }

        await telephoneGame(s, t, N, players, {init, preSpawn, postSpawn, stepCheck})
    })
}