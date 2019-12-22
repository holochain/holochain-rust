import { Config } from '@holochain/tryorama'
import * as R from 'ramda'
import { Batch } from '@holochain/tryorama-stress-utils'


const trace = R.tap(x => console.log('{T}', x))
const delay = ms => new Promise(r => setTimeout(r, ms))

module.exports = (scenario, configBatch, N, C, I) => {
    const totalInstances = N*C*I
    const totalConductors = N*C

    scenario('all agents exists', async (s, t) => {
        const players = R.sortBy(p => parseInt(p.name, 10), R.values(await s.players(configBatch(totalConductors, I), false)))

        // range of random number of milliseconds to wait before startup
        const startupSpacing = 10000
        // number of milliseconds to wait between gets
        const getWait = 100

        await Promise.all(players.map(async player => {
//            await delay(Math.random()*startupSpacing)
            return player.spawn()
        }))

     //   console.log("all nodes have started, now waiting 10 seconds for settling")
   //     await delay(10000)

        const batch = new Batch(players).iteration('parallel')

        const agentIds = await batch.mapInstances(async instance => instance.agentAddress)
        let results = []
        //        await bath.mapInstances(async instance => {

        // get the first instance
        let instance = players[0].instances()[0]
        console.log(`\n-------------------------------------------\ngetting ${totalInstances} entries for ${instance.agentAddress}\n---------------------------\n`)
        for (const id of agentIds) {
            if (instance.agentAddress != id) {
                console.log(`\n==== getting ${id}`)
                //                    await delay(getWait)
                const result = await instance.call('main', 'get_entry', {address: instance.agentAddress})
                results.push( Boolean(result.Ok) )
            }
        }
        //        })
        console.log("RESULTS:", results)
        // All results contain the full set of other nodes
        t.deepEqual(results , R.repeat(true,totalInstances-1))
        //t.deepEqual(results , R.repeat(true,totalInstances*(totalInstances-1)))

    })
}
