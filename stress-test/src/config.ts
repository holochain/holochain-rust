import * as R from 'ramda'
import { Config } from '@holochain/tryorama'
import { configBatchSimple } from '@holochain/tryorama-stress-utils'


export const networkType = process.env.APP_SPEC_NETWORK_TYPE || 'sim2h'

const logger = {
  type: 'debug',
  rules: {
    rules: [
      {
        exclude: true,
        pattern: '.*parity.*'
      },
      {
        exclude: true,
        pattern: '.*mio.*'
      },
      {
        exclude: true,
        pattern: '.*tokio.*'
      },
      {
        exclude: true,
        pattern: '.*hyper.*'
      },
      {
        exclude: true,
        pattern: '.*rusoto_core.*'
      },
      {
        exclude: true,
        pattern: '.*want.*'
      },
      {
        exclude: true,
        pattern: '.*rpc.*'
      }
    ]
  },
  state_dump: false
}

const network =
  ( networkType === 'sim2h'
  ? {
    type: 'sim2h',
    sim2h_url: 'ws://localhost:9002'
  }

  : networkType === 'memory'
  ? Config.network('memory')

  : (() => {throw new Error(`Unsupported network type: ${networkType}`)})()
  )

const dna = Config.dna('passthrough-dna.dna.json', 'passthrough')

export const configCommon = {
    logger,
    network,
}

/** Generates a bunch of identical conductor configs with multiple identical instances */
export const configBatch = (numConductors, numInstances) => (
  configBatchSimple(numConductors, numInstances, dna, configCommon)
)
