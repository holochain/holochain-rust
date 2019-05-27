import { dnaInterfaceId } from './config'
import { connect } from '@holochain/hc-web-client'
import * as kill from 'tree-kill'

type ChildProcess = any
type WebsocketConnection = any

type Call = (...segments: Array<string>) => (params: any) => Promise<any>
type CallZome = (instanceId: string, zome: string, func: string) => (params: any) => Promise<any>

export default class ConductorHandle {
  adminPort: number
  instancePort: number
  handle: ChildProcess
  agentId: string
  agentName: string
  adminWs: WebsocketConnection
  instanceWs: WebsocketConnection
  callZome: CallZome
  callAdmin: Call

  constructor({ adminPort, instancePort, handle }) {
    this.adminPort = adminPort
    this.instancePort = instancePort
    this.handle = handle

    this.agentId = `agent-${this.instancePort}`
    this.agentName = `Agent${this.instancePort}`

    this.adminWs = null
    this.instanceWs = null
  }

  initialize() {
    return Promise.all([
      connect(`ws://localhost:${this.adminPort}`).then(async ({ call, ws }) => {
        this.callAdmin = call
        this.adminWs = ws
        return this.createAgent()
      }),
      connect(`ws://localhost:${this.instancePort}`).then(({ callZome, ws }) => {
        this.callZome = callZome
        this.instanceWs = ws
      })
    ])
  }

  createAgent() {
    return this.callAdmin('test/agent/add')({ id: this.agentId, name: this.agentName })
  }

  async createDnaInstance(instanceId: string, dnaPath: string) {
    const dnaId = 'dna' + instanceId
    await this.callAdmin('admin/dna/install_from_file')({
      id: dnaId,
      path: dnaPath,
    })
    const instanceInfo = {
      id: instanceId,
      agent_id: this.agentId,
      dna_id: dnaId,
    }
    await this.callAdmin('admin/instance/add')(instanceInfo)
    await this.callAdmin('admin/instance/start')(instanceInfo)

    // we know that calling add_instance is going to trigger
    // a websocket shutdown and reconnect, so we don't want to consider
    // this function call complete until we have the reconnection
    const promise = new Promise(resolve => this.instanceWs.once('open', resolve))
    this.callAdmin('admin/interface/add_instance')({
      interface_id: dnaInterfaceId,
      instance_id: instanceId,
    })
    return promise
  }

  onSignal(fn: (signal: object) => void) {
    this.instanceWs.socket.on('message', rawMessage => {
      const msg = JSON.parse(rawMessage)
      const isTrace = msg.signal && msg.signal.signal_type === 'Trace'
      if (isTrace) {
        const { action } = msg.signal
        fn(action)
      }
    })
  }

  shutdown() {
    return new Promise((resolve, reject) => {
      kill(this.handle.pid, (e) => {
        if (e) reject(e)
        else resolve()
      })
    })
  }
}
