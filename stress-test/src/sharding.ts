import { Config } from '@holochain/tryorama'
import * as R from 'ramda'
import { Batch } from '@holochain/tryorama-stress-utils'


const trace = R.tap(x => console.log('{T}', x))
const delay = ms => new Promise(r => setTimeout(r, ms))

module.exports = (scenario, configBatch, N, C, I, sampleSize) => {
    const totalInstances = N*C*I
    const totalConductors = N*C

    scenario('agents can get other agents on sharded network', async (s, t) => {
        const players = R.sortBy(p => parseInt(p.name, 10), R.values(await s.players(configBatch(totalConductors, I), false)))

        // range of random number of milliseconds to wait before startup
        const startupSpacing = 10000
        // number of milliseconds to wait between gets
        const getWait = 50

        await Promise.all(players.map(async player => {
            await delay(Math.random()*startupSpacing)
            return player.spawn()
        }))

        console.log("============================================\nall nodes have started\n============================================")
        console.log(`beginning test with sample size: ${sampleSize}`)

       //await delay(10000)

        const batch = new Batch(players).iteration('parallel')

        await consistency(batch)

        const agentIds = await batch.mapInstances(async instance => instance.agentAddress)
        let results = []
        let i = 0
        let checkedCount = 0;
        let mod = Math.floor(totalInstances/sampleSize)
        await batch.mapInstances(async instance => {
            if  ( i % mod == 0) {
                checkedCount += 1
                console.log(`\n-------------------------------------------\ngetting ${totalInstances} entries for ${i} (${instance.agentAddress})\n---------------------------\n`)
                for (const id of agentIds) {
                    if (instance.agentAddress != id) {
                        console.log(`\n==== getting ${id}`)
                        await delay(getWait)
                        const result = await instance.call('main', 'get_entry', {address: id})
                        results.push( Boolean(result.Ok) )
                    }
                }
            }
            i+=1
        })
        // All results contain the full set of other nodes
        t.deepEqual(results , R.repeat(true,checkedCount*(totalInstances-1)))

    })
}

async function consistency(batch) {
    let retries = 5
    let retryDelay = 10000
    let tries = 0
    while (tries < retries) {
        tries += 1
        console.log(`Checking holding: try ${tries}`)
        const dht_state = await getDHTstate(batch)
        const {missing, held_by} = checkHolding(dht_state)
        if (missing == 0) {
           return true
        }
        console.log(`all not held missing: ${missing}, retrying after delay`)
        await delay(retryDelay)
    }
    console.log(`consistency not reached after ${retries} attempts`)
    return false
}

const getDHTstate = async (batch: Batch) => {
  let entries_map = {}
  const holding_map = await batch.mapInstances(async instance => {
    const id = `${instance.id}:${instance.agentAddress}`
    console.log(`calling state dump for instance ${id})`)
    const dump = await instance.stateDump()
    const held_addresses = R.keys(dump.held_aspects)
    const sourceChain = R.values(dump.source_chain)
    const entryMap = {}
    for (const entry of sourceChain) {
        if (entry.entry_type == "AgentId" ) {
        entries_map[entry.entry_address] = true
      }
    }
    return {
      instance_id: instance.id,
      held_addresses,
      agent_address: instance.agentAddress
    }
  })
  return {
    entries: R.keys(entries_map),
    holding: holding_map
  }
}

function checkHolding(dht_state) {
    let missing = 0
    let held_by = {}
    console.log("total number of entries returned by state dumps:", dht_state["entries"].length)
    for (const entry_address of dht_state["entries"]) {
        let holders = []
        for (const holding of dht_state["holding"]) {
            if (holding.held_addresses.includes(entry_address)) {
                holders.push(holding.agent_address)
            }
        }
        held_by[entry_address] = holders
        console.log(`${entry_address} is held by ${holders.length} agents`)
        if (holders.length === 0) {
            missing += 1
        }
    }
    return {missing, held_by}
}
