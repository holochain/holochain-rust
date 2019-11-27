const { one } = require('../config')

const delay = ms => new Promise(resolve => setTimeout(resolve, ms)) 

module.exports = scenario => {

      scenario('Can retriev header entries using get_entry', async (s, t) => {
        const { alice, bob } = await s.players({ alice: one, bob: one })
        await alice.spawn()
        await bob.spawn()

        await s.consistency()

        // alice publishes a memo. This is private but should still publish a header
        const create_result = await alice.call('app', "blog", "create_memo", { content: "private memo" })
        t.comment(JSON.stringify(create_result))
        t.equal(create_result.Ok.length, 46)
        await s.consistency()

        // get all the chain header hashes and check if they are retrievable
        const maybe_chain_header_hashes = await alice.call('app', "blog", "get_chain_header_hashes", {})
        t.ok(maybe_chain_header_hashes.Ok)
        let chain_header_hashes = maybe_chain_header_hashes.Ok
        t.equal(chain_header_hashes.length, 4) // dna, agentId, cap grant, memo

        t.comment(JSON.stringify(chain_header_hashes))
        let chain_headers = []

        await s.consistency()

        for (let i=0; i< chain_header_hashes.length; i++) {
            // can use get_post because it just returns a raw entry given a hash
            let header_hash = chain_header_hashes[i]
            t.comment(header_hash)

            // check alice can retrieve their own header entries
            let header_alice = await alice.call('app', "blog", "get_post", { post_address: header_hash })
            t.ok(header_alice.Ok)

            // check bob can retrieve alices header entries
            let header_bob = await bob.call('app', "blog", "get_post", { post_address: header_hash })
            t.ok(header_bob.Ok)

            chain_headers.push(header_bob.Ok)
        }
        t.comment(JSON.stringify(chain_headers))
      })

      scenario('Can perform validation of an entry while the author is offline', async (s, t) => {
        
        const { alice, bob, carol } = await s.players({alice: one, bob: one, carol: one})
        // alice and bob start online
        await alice.spawn()
        await bob.spawn()

        // alice publishes the original entry. !This is an entry that requires full chain validation!
        const initialContent = "Holo world y'all"
        const params = { content: initialContent, in_reply_to: null }
        const create_result = await alice.call('app', "blog", "create_post", params)
        t.comment(JSON.stringify(create_result))
        t.equal(create_result.Ok.length, 46)
        await s.consistency()

        // get all the chain header hashes and check if they are retrievable
        const chain_header_hashes = await alice.call('app', "blog", "get_chain_header_hashes", {})
        let chain_headers = [];

        // check we can actually retrieve our own chain headers as the authoring agent
        for (header_hash in chain_header_hashes) {
            // can use get_post because it just returns a raw entry given a hash
            let header = await alice.call('app', "blog", "get_post", { post_address: header_hash })
            t.ok(header.Ok)
            chain_headers.push(header)
        }

        t.comment(JSON.stringify(chain_headers))

        t.comment('waiting for consistency between Alice and Bob')
        // bob will receive the entry and hold it
        await s.consistency()
        t.comment('consistency has been reached')
        
        // alice then goes offline
        t.comment('waiting for alice to go offline')
        await alice.kill()
        t.comment('alice has gone offline')

        // carol then comes online, will receive the entry via gossip from bob and need to validate
        // Since alice is offline the validation package cannot come direct and must
        // be regenerated from the published headers (which bob should hold)
        t.comment('waiting for Carol to come online')
        await carol.spawn()
        t.comment('Carol is online')

        t.comment('Waiting for Carol to get all data via gossip')
        await s.consistency()
        await delay(20000)
        t.comment('consistency has been reached')

        // Bob now go offline to ensure the following get_post uses carols local store only
        t.comment('waiting for Bob to go offline')
        await bob.kill()
        t.comment('Bob has gone offline')

        const post_address = create_result.Ok
        const params_get = { post_address }

        t.comment('Waiting for Carol to get post') // <- fails here. Times out when using memory/sim1h and returns null with sim2h
        const result = await carol.call('app', "blog", "get_post", params_get)
        t.comment(JSON.stringify(result))
        const value = JSON.parse(result.Ok.App[1])
        t.equal(value.content, initialContent)
      })

    }
