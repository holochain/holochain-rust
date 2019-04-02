const test = require('tape');

const { Conductor } = require('..')

const toml = `
[[agents]]
id = "test/agent/1"
name = "Holo Tester 1"
keystore_file = "holo_tester1.key"
public_address = "HcScJdXW5uHo9y8jryEwW8N59akhrgxh93acu33qe53ximagfiWu98j7J6Ofiur"

[[agents]]
id = "test/agent/2"
name = "Holo Tester 2"
keystore_file = "holo_tester2.key"
public_address = "HcScIrhJ5ECmano9jwiE9FWmacTybe7u9bpDURFGZixr7k5sVdAR4ABMpnywu5a"

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
