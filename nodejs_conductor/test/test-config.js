const test = require('tape')

const { Config } = require('..')
const C = Config

test('agent construction', t => {
    const name = 'alice'
    const agent = C.agent(name)
    t.deepEqual(agent, { name })
    t.end()
})

test('DNA construction with implicit name', t => {
    const path = 'path/to/dna'
    const dna = C.dna(path)
    t.deepEqual(dna, { path, name: path })
    t.end()
})

test('DNA construction with explicit name', t => {
    const path = 'path/to/dna'
    const dna = C.dna(path, 'george')
    t.deepEqual(dna, { path, name: 'george' })
    t.end()
})

test('instance construction with implicit name', t => {
    const path = 'path/to/dna'
    const agent = C.agent('allison')
    const dna = C.dna(path, 'dnaName')
    const instance = C.instance(agent, dna)
    t.deepEqual(instance, { agent, dna, name: 'allison' })
    t.end()
})

test('instance construction with explicit name', t => {
    const path = 'path/to/dna'
    const agent = C.agent('konstantin')
    const dna = C.dna(path)
    const instance = C.instance(agent, dna, 'kostya')
    t.deepEqual(instance, { agent, dna, name: 'kostya' })
    t.end()
})


//////////////////////////////////////////////////////////
/// Some more extensive config tests follow


const commonConfigAssertions = (t, config, path) => {
    t.deepEqual(
        config.agents.map(a => a.id).sort(),
        ['alessia', 'bartolini']
    )
    t.deepEqual(
        config.dnas.map(d => d.id).sort(),
        [path]
    )
    t.equal(config.instances[0].id, `alessia`)
    t.equal(config.instances[0].agent, `alessia`)
    t.equal(config.instances[0].dna, path)
    t.equal(config.instances[1].id, `bartolini`)
    t.equal(config.instances[1].agent, `bartolini`)
    t.equal(config.instances[1].dna, path)
    t.equal(config.interfaces.length, 0)
}

/**
 * Brittle check for whether debug logging has been activated.
 * Check this if a test is failing!
 */
const isFullDebugLogger = logger => logger.rules.rules.length > 1

test('config construction, two argument, backwards compatible version', t => {
    const path = 'path/to/dna'
    const agent1 = C.agent('alessia')
    const agent2 = C.agent('bartolini')
    const dna = C.dna(path)
    const config = C.conductor([
        C.instance(agent1, dna),
        C.instance(agent2, dna),
    ])
    console.log(config.logger.rules.rules)
    commonConfigAssertions(t, config, path)
    t.equal(config.bridges.length, 0)
    t.notOk(isFullDebugLogger(config.logger))
    t.end()
})

test('config construction, two argument, backwards compatible version with options', t => {
    const path = 'path/to/dna'
    const agent1 = C.agent('alessia')
    const agent2 = C.agent('bartolini')
    const dna = C.dna(path)
    const config = C.conductor([
        C.instance(agent1, dna),
        C.instance(agent2, dna),
    ], {
        debugLog: true
    })
    commonConfigAssertions(t, config, path)
    t.equal(config.bridges.length, 0)
    t.ok(isFullDebugLogger(config.logger))
    t.end()
})

test('config construction, single argument version', t => {
    const path = 'path/to/dna'
    const agent1 = C.agent('alessia')
    const agent2 = C.agent('bartolini')
    const dna = C.dna(path)
    const config = C.conductor({
        instances: [
            C.instance(agent1, dna),
            C.instance(agent2, dna),
        ]
    })
    commonConfigAssertions(t, config, path)
    t.equal(config.bridges.length, 0)
    t.notOk(isFullDebugLogger(config.logger))
    t.end()
})

test('config construction, single argument version, with bridges and logger', t => {
    const path = 'path/to/dna'
    const agent1 = C.agent('alessia')
    const agent2 = C.agent('bartolini')
    const dna = C.dna(path)
    const instance1 = C.instance(agent1, dna)
    const instance2 = C.instance(agent2, dna)
    const config = C.conductor({
        instances: [instance1, instance2],
        bridges: [
            C.bridge('bridge-forward', instance1, instance2),
            C.bridge('bridge-backward', instance2, instance1),
        ],
        debugLog: true,
    })
    commonConfigAssertions(t, config, path)
    t.ok(isFullDebugLogger(config.logger))
    t.deepEqual(config.bridges, [
        {handle: 'bridge-forward', caller_id: 'alessia', callee_id: 'bartolini'},
        {handle: 'bridge-backward', caller_id: 'bartolini', callee_id: 'alessia'},
    ])
    t.end()
})

test('config construction with dpki', t => {
    const agent1 = C.agent('alessia')
    const agent2 = C.agent('bartolini')
    const agentDpki = C.agent('dpki')
    const dnaApp = C.dna('path/to/dna/app')
    const dnaDpki = C.dna('path/to/dna/dpki')
    const instance1 = C.instance(agent1, dnaApp)
    const instance2 = C.instance(agent2, dnaApp)
    const instanceDpki = C.instance(agentDpki, dnaDpki, 'dpki-instance')
    const config = C.conductor({
        instances: [instance1, instance2],
        dpki: C.dpki(instanceDpki, JSON.stringify({foo: 'bar'}))
    })
    t.deepEqual(config.dpki, {
      instance_id: 'dpki-instance',
      init_params: '{"foo":"bar"}' })
    t.end()
})
