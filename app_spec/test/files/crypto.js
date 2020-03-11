const { one, two } = require('../config')

module.exports = scenario => {

  scenario('add Seeds', async (s, t) => {
    const { alice } = await s.players({ alice: one }, true)

      const ListResult = await alice.call('app', 'converse', 'list_secrets', { })
      // it should start out with the genesis made seed
      t.deepEqual(ListResult, { Ok: ['app_root_seed', 'primary_keybundle:enc_key', 'primary_keybundle:sign_key', 'root_seed'] })

      const AddSeedResult = await alice.call('app', 'converse', 'add_any_seed', { src_id: 'app_root_seed', dst_id: 'app_seed:1', context:"contexts" index: 1, seed_type: "Root" })
      t.ok(AddSeedResult)
      //
      // const AddKeyResult = await alice.call('app', 'converse', 'add_key', { src_id: 'app_seed:1', dst_id: 'app_key:1' })
      // t.ok(AddKeyResult)


  })

  // scenario('sign_and_verify_message', async (s, t) => {
  //   const { alice, bob } = await s.players({ alice: one, bob: one }, true)
  //   const message = 'Hello everyone! Time to start the secret meeting'
  //
  //   const SignResult = await bob.call('app', 'converse', 'sign_message', { key_id: '', message: message })
  //   t.ok(SignResult.Ok)
  //
  //   const provenance = [bob.info('app').agentAddress, SignResult.Ok]
  //
  //   const VerificationResult = await alice.call('app', 'converse', 'verify_message', { message, provenance })
  //   t.deepEqual(VerificationResult, { Ok: true })
  // })
  //
  // scenario('encrypt_and_decrypt_message', async (s, t) => {
  //   const { alice, bob } = await s.players({ alice: one, bob: one }, true)
  //   const message = 'Hello everyone! Time to start the secret meeting'
  //
  //   const EncryptResult = await bob.call('app', 'simple', 'encrypt', { payload: message })
  //
  //   t.ok(EncryptResult)
  //   const DecryptResult = await alice.call('app', 'simple', 'decrypt', { payload: EncryptResult.Ok })
  //   t.deepEqual(DecryptResult.Ok, message)
  // })
  //
  // scenario('secrets', async (s, t) => {
  //   const { alice } = await s.players({ alice: one }, true)
  //
  //   const ListResult = await alice.call('app', 'converse', 'list_secrets', { })
  //   // it should start out with the genesis made seed
  //   t.deepEqual(ListResult, { Ok: ['app_root_seed', 'primary_keybundle:enc_key', 'primary_keybundle:sign_key', 'root_seed'] })
  //
  //   const AddSeedResult = await alice.call('app', 'converse', 'add_seed', { src_id: 'app_root_seed', dst_id: 'app_seed:1', index: 1 })
  //   t.ok(AddSeedResult)
  //
  //   const AddKeyResult = await alice.call('app', 'converse', 'add_key', { src_id: 'app_seed:1', dst_id: 'app_key:1' })
  //   t.ok(AddKeyResult)
  //
  //   const ListResult1 = await alice.call('app', 'converse', 'list_secrets', { })
  //   // it should start out with the genesis made seed
  //   t.deepEqual(ListResult1, { Ok: ['app_key:1', 'app_root_seed', 'app_seed:1', 'primary_keybundle:enc_key', 'primary_keybundle:sign_key', 'root_seed'] })
  //
  //   const message = 'Hello everyone! Time to start the secret meeting'
  //
  //   const SignResult = await alice.call('app', 'converse', 'sign_message', { key_id: 'app_key:1', message: message })
  //   t.ok(SignResult)
  //
  //   // use the public key returned by add key as the provenance source
  //   const provenance = [AddKeyResult.Ok, SignResult.Ok]
  //   const VerificationResult = await alice.call('app', 'converse', 'verify_message', { message, provenance })
  //   t.deepEqual(VerificationResult, { Ok: true })
  //
  //   // use the agent key as the provenance source (which should fail)
  //   const provenance1 = [alice.info('app').agentAddress, SignResult.Ok]
  //   const VerificationResult1 = await alice.call('app', 'converse', 'verify_message', { message, provenance: provenance1 })
  //   t.deepEqual(VerificationResult1, { Ok: false })
  //
  //   const GetKeyResult = await alice.call('app', 'converse', 'get_pubkey', { src_id: 'app_key:1' })
  //   t.ok(GetKeyResult)
  //   t.deepEqual(GetKeyResult, AddKeyResult)
  // })
  //
  // scenario.skip('capabilities grant and claim', async (s, t) => {
  //   const { alice, bob } = await s.players({ alice: one, bob: one }, true)
  //
  //   // Ask for alice to grant a token for bob  (it's hard-coded for bob in re function for now)
  //   const result = await alice.call('app', 'blog', 'request_post_grant', {})
  //   t.ok(result.Ok)
  //   t.notOk(result.Err)
  //
  //   // Confirm that we can get back the grant
  //   const grants = await alice.call('app', 'blog', 'get_grants', {})
  //   t.ok(grants.Ok)
  //   t.notOk(grants.Err)
  //   t.equal(result.Ok, grants.Ok[0])
  //
  //   // Bob stores the grant as a claim
  //   const claim = await bob.call('app', 'blog', 'commit_post_claim', { grantor: alice.info('app').agentAddress, claim: result.Ok })
  //   t.deepEqual(claim, { Ok: 'QmYsFu7QGaVeUUac1E4BWST7BR38cYvzRaaTc3YS9WqsTu' })
  //
  //   // Bob can now create a post on alice's chain via a node-to-node message with the claim
  //   const post_content = 'Holo world'
  //   const params = { grantor: alice.info('app').agentAddress, content: post_content, in_reply_to: null }
  //   const create_result = await bob.call('app', 'blog', 'create_post_with_claim', params)
  //   t.deepEqual(create_result, { Ok: 'QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk' })
  //
  //   // Confirm that the post was actually added to alice's chain
  //   const get_post_result = await alice.call('app', 'blog', 'get_post', { post_address: create_result.Ok })
  //   const value = JSON.parse(get_post_result.Ok.App[1])
  //   t.equal(value.content, post_content)
  //
  //   // Check that when bob tries to make this call it fails because there is no grant stored
  //   const params2 = { grantor: bob.info('app').agentAddress, content: post_content, in_reply_to: null }
  //   const create2_result = await bob.call('app', 'blog', 'create_post_with_claim', params2)
  //   t.deepEqual(create2_result, { Ok: 'error: no matching grant for claim' })
  // })
  //
  // scenario('request grant', async (s, t) => {
  //   const { alice, bob } = await s.players({ alice: one, bob: one }, true)
  //
  //   /*
  //         This is not a complete test of requesting a grant because currently there
  //         is no way in the test conductor to actually pass in the provenance of the
  //         call.  That will be added when we convert the test framework to being built
  //         on top of the rust conductor.   For now this is more a placeholder test, but
  //         note that the value returned is actually the capbability token value.
  //       */
  //   const result = await alice.call('app', 'blog', 'request_post_grant', {})
  //   t.ok(result.Ok)
  //   t.notOk(result.Err)
  //
  //   const grants = await alice.call('app', 'blog', 'get_grants', {})
  //   t.ok(grants.Ok)
  //   t.notOk(grants.Err)
  //
  //   t.equal(result.Ok, grants.Ok[0])
  // })
  //
  // scenario('create_post_countersigned', async (s, t) => {
  //   const { alice, bob } = await s.players({ alice: one, bob: one }, true)
  //
  //   const content = 'Holo world'
  //   const in_reply_to = null
  //
  //   const address_params = { content }
  //   const address_result = await bob.call('app', 'blog', 'post_address', address_params)
  //
  //   t.ok(address_result.Ok)
  //   const SignResult = await bob.call('app', 'converse', 'sign_message', { key_id: '', message: address_result.Ok })
  //   t.ok(SignResult.Ok)
  //
  //   const counter_signature = [bob.info('app').agentAddress, SignResult.Ok]
  //
  //   const params = { content, in_reply_to, counter_signature }
  //   const result = await alice.call('app', 'blog', 'create_post_countersigned', params)
  //
  //   t.ok(result.Ok)
  //   t.notOk(result.Err)
  //   t.equal(result.Ok, 'QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk')
  // })
}
