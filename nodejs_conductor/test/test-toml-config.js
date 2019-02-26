const test = require('tape');

const { Conductor } = require('..')

const toml = `
[[agents]]
id = "test/agent/1"
name = "Holo Tester 1"
key_file = "holo_tester1.key"
public_address = "jtXczt_fyYXJWvmb-BR4Gsf-QCnnLybzZCwGKis_-T0WEiRtv64xt102HHmsYmJTQRWMqmrLWhr40rt11W0aI3S7VIZD"

[[agents]]
id = "test/agent/2"
name = "Holo Tester 2"
key_file = "holo_tester2.key"
public_address = "QOnZBLAxv01QT5arAKVwk7P4XDnApsItDqtxoMENACux4_PrFegIdgbEp2h-sz3vGWCQUBygskcXOJ_Da7d_JkvPzRbV"

[[dnas]]
id = "test/dna"
file = "test/test.dna.json"
hash = "Qm328wyq38924y"

[[instances]]
id = "test/instance/1"
dna = "test/dna"
agent = "test/agent/1"
[instances.storage]
type = "memory"

[[instances]]
id = "test/instance/2"
dna = "test/dna"
agent = "test/agent/2"
[instances.storage]
type = "memory"

[[interfaces]]
id = "test/interface"
[interfaces.driver]
type = "websocket"
port = 8888
[[interfaces.instances]]
id = "test/instance/1"
[[interfaces.instances]]
id = "test/instance/2"

[logger]
type = "debug"
`

test('can create config from TOML (run)', t => {
    Conductor.run(toml, (stop, conductor) => {
        t.throws(
            () => conductor.call('x', 'x', 'x', 'x'),
            /No instance with id/
        )
        t.throws(
            () => conductor.call(
                'test/instance/1', 'blog', 'not-a-function', 'param'
            ),
            /Zome function .*? not found/
        )
        stop()
        t.end()
    })
})

test('can create config from TOML', t => {
    const conductor = new Conductor(toml)
    conductor.start()
    t.throws(
        () => conductor.call('x', 'x', 'x', 'x'),
        /No instance with id/
    )
    t.throws(
        () => conductor.call(
            'test/instance/1', 'blog', 'not-a-function', 'param'
        ),
        /Zome function .*? not found/
    )
    conductor.stop()
    t.end()
})
