const { one, two } = require('../config')

module.exports = scenario => {
  scenario('delete_entry_post', async (s, t) => {
    const { alice, bob } = await s.players({ alice: one, bob: one }, true)
    const content = 'Hello Holo world 321'
    const in_reply_to = null
    const params = { content, in_reply_to }

    // commit create_post
    const createResult = await alice.callSync('app', 'blog', 'create_post', params)

    t.ok(createResult.Ok)

    // delete entry post
    const deletionParams = { post_address: createResult.Ok }
    const deletionResult = await alice.callSync('app', 'blog', 'delete_entry_post', deletionParams)

    t.ok(deletionResult.Ok)

    // delete should fail
    const failedDelete = await bob.callSync('app', 'blog', 'delete_entry_post', { post_address: createResult.Ok })
    t.deepEqual(failedDelete.Err, { Internal: 'Entry Could Not Be Found' })

    // get initial entry
    const GetInitialParamsResult = await bob.call('app', 'blog', 'get_initial_post', { post_address: createResult.Ok })
    t.deepEqual(JSON.parse(GetInitialParamsResult.Ok.App[1]), { content: 'Hello Holo world 321', date_created: 'now' })

    const entryWithOptionsGet = { post_address: createResult.Ok }
    const entryWithOptionsGetResult = await bob.call('app', 'blog', 'get_post_with_options', entryWithOptionsGet)
    t.deepEqual(JSON.parse(entryWithOptionsGetResult.Ok.result.All.items[0].entry.App[1]), { content: 'Hello Holo world 321', date_created: 'now' })
  })

  scenario.only('update_entry_validation', async (s, t) => {
    const { alice, bob } = await s.players({ alice: one, bob: one }, true)
    // update entry does not exist
    const updateParams = { post_address: '1234', new_content: 'Hello Holo V2' }
    const UpdateResult = await bob.callSync('app', 'blog', 'update_post', updateParams)

    t.deepEqual(UpdateResult.Err, { Internal: 'failed to update post' })

    const content = 'Hello Holo world 321'
    const in_reply_to = null
    const params = { content, in_reply_to }

    // commit create_post
    const createResult = await alice.callSync('app', 'blog', 'create_post', params)

    t.ok(createResult.Ok)

    const updateParamsV2 = { post_address: createResult.Ok, new_content: 'Hello Holo world 321' }
    const UpdateResultV2 = await bob.callSync('app', 'blog', 'update_post', updateParamsV2)
    console.log("UpdateResultV2", UpdateResultV2)
    t.deepEqual(JSON.parse(UpdateResultV2.Err.Internal).kind.ValidationFailed, 'Trying to modify with same data')
  })

  scenario('update_post', async (s, t) => {
    const { alice, bob } = await s.players({ alice: one, bob: one }, true)
    const content = 'Hello Holo world 123'
    const in_reply_to = null
    const params = { content, in_reply_to }

    // commit version 1
    const createResult = await alice.call('app', 'blog', 'create_post', params)
    t.ok(createResult.Ok)

    await s.consistency()

    // get v1
    const updatedPostV1 = await alice.call('app', 'blog', 'get_post', { post_address: createResult.Ok })
    const UpdatePostV1Content = { content: 'Hello Holo world 123', date_created: 'now' }
    t.ok(updatedPostV1.Ok)
    t.deepEqual(JSON.parse(updatedPostV1.Ok.App[1]), UpdatePostV1Content)

    // update to version 2
    const updatePostContentV2 = { content: 'Hello Holo V2', date_created: 'now' }
    const updateParamsV2 = { post_address: createResult.Ok, new_content: 'Hello Holo V2' }
    const UpdateResultV2 = await bob.call('app', 'blog', 'update_post', updateParamsV2)
    t.ok(UpdateResultV2.Ok)
    t.notOk(UpdateResultV2.Err)

    await s.consistency()

    // get v2 using initial adderss
    const updatedPostv2Initial = await alice.call('app', 'blog', 'get_post', { post_address: createResult.Ok })
    t.ok(updatedPostv2Initial.Ok)
    t.notOk(updatedPostv2Initial.Err)
    t.deepEqual(JSON.parse(updatedPostv2Initial.Ok.App[1]), updatePostContentV2) // 8

    // get v2 latest address
    const updatedPostv2Latest = await alice.call('app', 'blog', 'get_post', { post_address: UpdateResultV2.Ok })
    t.ok(updatedPostv2Latest.Ok)
    t.notOk(updatedPostv2Latest.Err)
    t.deepEqual(JSON.parse(updatedPostv2Latest.Ok.App[1]), updatePostContentV2) // 11

    // get v1 using initial address
    const GetInitialPostV1Initial = await alice.call('app', 'blog', 'get_initial_post', { post_address: createResult.Ok })
    t.ok(GetInitialPostV1Initial.Ok)
    t.notOk(GetInitialPostV1Initial.Err)
    t.deepEqual(JSON.parse(GetInitialPostV1Initial.Ok.App[1]), { content: 'Hello Holo world 123', date_created: 'now' })

    // get v2 latest address
    const GetInitialPostV2Latest = await alice.call('app', 'blog', 'get_initial_post', { post_address: UpdateResultV2.Ok })
    t.ok(GetInitialPostV2Latest.Ok)
    t.notOk(GetInitialPostV2Latest.Err)
    t.deepEqual(JSON.parse(GetInitialPostV2Latest.Ok.App[1]), updatePostContentV2)

    // update to version 3
    const UpdatePostV3Content = { content: 'Hello Holo V3', date_created: 'now' }
    const updateParamsV3 = { post_address: createResult.Ok, new_content: 'Hello Holo V3' }
    const UpdateResultV3 = await alice.call('app', 'blog', 'update_post', updateParamsV3)
    t.ok(UpdateResultV3.Ok)
    t.notOk(UpdateResultV3.Err)

    await s.consistency()

    // get v3 using initial adderss
    const updatedPostV3Initial = await alice.call('app', 'blog', 'get_post', { post_address: createResult.Ok })
    t.ok(updatedPostV3Initial.Ok)
    t.notOk(updatedPostV3Initial.Err)
    t.deepEqual(JSON.parse(updatedPostV3Initial.Ok.App[1]), UpdatePostV3Content)

    // get v3 using address of v2
    const updatedPostV3Latest = await alice.call('app', 'blog', 'get_post', { post_address: UpdateResultV2.Ok })
    t.ok(updatedPostV3Latest.Ok)
    t.notOk(updatedPostV3Latest.Err)
    t.deepEqual(JSON.parse(updatedPostV3Latest.Ok.App[1]), UpdatePostV3Content)

    // update to version 4
    const updatePostV4Content = { content: 'Hello Holo V4', date_created: 'now' }
    const updateParamsV4 = { post_address: createResult.Ok, new_content: 'Hello Holo V4' }
    const UpdateResultV4 = await alice.call('app', 'blog', 'update_post', updateParamsV4)
    t.notOk(UpdateResultV4.Err)
    t.ok(UpdateResultV4.Ok)

    await s.consistency()

    // get history entry v4
    const entryHistoryV4Params = { post_address: UpdateResultV4.Ok }
    const entryHistoryV4 = await alice.call('app', 'blog', 'get_history_post', entryHistoryV4Params)
    t.ok(UpdateResultV4.Ok)
    t.notOk(UpdateResultV4.Err)
    t.deepEqual(entryHistoryV4.Ok.items.length, 1)
    t.deepEqual(JSON.parse(entryHistoryV4.Ok.items[0].entry.App[1]), updatePostV4Content)
    t.deepEqual(entryHistoryV4.Ok.items[0].meta.address, UpdateResultV4.Ok)
    t.deepEqual(entryHistoryV4.Ok.items[0].meta.crud_status, 'live')

    // get history entry all
    const entryHistoryAllParams = { post_address: createResult.Ok }
    const entryHistoryAll = await alice.call('app', 'blog', 'get_history_post', entryHistoryAllParams)

    t.deepEqual(entryHistoryAll.Ok.items.length, 4)
    t.deepEqual(JSON.parse(entryHistoryAll.Ok.items[0].entry.App[1]), { content: 'Hello Holo world 123', date_created: 'now' })
    t.deepEqual(entryHistoryAll.Ok.items[0].meta.address, createResult.Ok)
    t.deepEqual(entryHistoryAll.Ok.items[0].meta.crud_status, 'modified')
    t.deepEqual(entryHistoryAll.Ok.crud_links[createResult.Ok], UpdateResultV2.Ok)

    t.deepEqual(JSON.parse(entryHistoryAll.Ok.items[1].entry.App[1]), updatePostContentV2)
    t.deepEqual(entryHistoryAll.Ok.items[1].meta.address, UpdateResultV2.Ok)
    t.deepEqual(entryHistoryAll.Ok.items[1].meta.crud_status, 'modified')
    t.deepEqual(entryHistoryAll.Ok.crud_links[UpdateResultV2.Ok], UpdateResultV3.Ok)

    t.deepEqual(JSON.parse(entryHistoryAll.Ok.items[2].entry.App[1]), UpdatePostV3Content)
    t.deepEqual(entryHistoryAll.Ok.items[2].meta.address, UpdateResultV3.Ok)
    t.deepEqual(entryHistoryAll.Ok.items[2].meta.crud_status, 'modified')
    t.deepEqual(entryHistoryAll.Ok.crud_links[UpdateResultV3.Ok], UpdateResultV4.Ok)

    t.deepEqual(JSON.parse(entryHistoryAll.Ok.items[3].entry.App[1]), updatePostV4Content)
    t.deepEqual(entryHistoryAll.Ok.items[3].meta.address, UpdateResultV4.Ok)
    t.deepEqual(entryHistoryAll.Ok.items[3].meta.crud_status, 'live')
    t.notOk(entryHistoryAll.Ok.crud_links[UpdateResultV4.Ok])

    const entryWithOptionsGet = { post_address: createResult.Ok }
    const entryWithOptionsGetResult = await alice.call('app', 'blog', 'get_post_with_options_latest', entryWithOptionsGet)

    t.deepEqual(JSON.parse(entryWithOptionsGetResult.Ok.App[1]), updatePostV4Content)
  })

  scenario('remove_update_modifed_entry', async (s, t) => {
    const { alice, bob } = await s.players({ alice: one, bob: one }, true)
    const content = 'Hello Holo world 123'
    const in_reply_to = null
    const params = { content, in_reply_to }

    // commit version 1
    const createResult = await alice.callSync('app', 'blog', 'create_post', params)
    t.ok(createResult.Ok)
    // get entry
    const updatedPostV1 = await alice.call('app', 'blog', 'get_post', { post_address: createResult.Ok })
    t.ok(updatedPostV1.Ok)
    t.deepEqual(JSON.parse(updatedPostV1.Ok.App[1]), { content: 'Hello Holo world 123', date_created: 'now' })

    // delete
    const removeParamsV2 = { post_address: createResult.Ok }
    const removeResultV2 = await bob.callSync('app', 'blog', 'delete_entry_post', removeParamsV2)
    t.ok(removeResultV2.Ok)

    // get v2 using initial adders
    const Postv2Initial = await alice.call('app', 'blog', 'get_initial_post', { post_address: createResult.Ok })
    t.ok(Postv2Initial.Ok)
    t.deepEqual(JSON.parse(Postv2Initial.Ok.App[1]), { content: 'Hello Holo world 123', date_created: 'now' })

    // failed delete
    const failedDelete = await alice.callSync('app', 'blog', 'delete_entry_post', { post_address: createResult.Ok })
    t.deepEqual(failedDelete.Err, { Internal: 'Entry Could Not Be Found' })
  })

  scenario('get sources', async (s, t) => {
    const { alice, bob, carol } = await s.players({ alice: one, bob: one, carol: one }, true)

    const params = { content: 'whatever', in_reply_to: null }

    const address = await alice.call('app', 'blog', 'create_post', params).then(x => x.Ok)
    const address1 = await alice.call('app', 'blog', 'create_post', params).then(x => x.Ok)
    const address2 = await bob.call('app', 'blog', 'create_post', params).then(x => x.Ok)
    const address3 = await carol.call('app', 'blog', 'create_post', params).then(x => x.Ok)

    t.equal(address, address1)
    t.equal(address, address2)
    t.equal(address, address3)

    await s.consistency()

    const sources1 = await alice.call('app', 'blog', 'get_sources', { address }).then(x => x.Ok.sort())
    const sources2 = await bob.call('app', 'blog', 'get_sources', { address }).then(x => x.Ok.sort())
    const sources3 = await carol.call('app', 'blog', 'get_sources', { address }).then(x => x.Ok.sort())

    // NB: alice shows up twice because she published the same entry twice
    const expected = [
      alice.info('app').agentAddress,
      alice.info('app').agentAddress,
      bob.info('app').agentAddress,
      carol.info('app').agentAddress
    ].sort()

    t.deepEqual(sources1, expected)
    t.deepEqual(sources2, expected)
    t.deepEqual(sources3, expected)
  })

  scenario('scenario test create & publish post -> get from other instance', async (s, t) => {
    const { alice, bob } = await s.players({ alice: one, bob: one }, true)

    const initialContent = 'Holo world'
    const params = { content: initialContent, in_reply_to: null }
    const create_result = await alice.callSync('app', 'blog', 'create_post', params)

    const params2 = { content: 'post 2', in_reply_to: null }
    const create_result2 = await bob.callSync('app', 'blog', 'create_post', params2)

    t.equal(create_result.Ok.length, 46)
    t.equal(create_result.Ok, 'QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk')

    const post_address = create_result.Ok
    const params_get = { post_address }

    const result = await bob.call('app', 'blog', 'get_post', params_get)
    const value = JSON.parse(result.Ok.App[1])
    t.equal(value.content, initialContent)
  })

  scenario('get_post with non-existant address returns null', async (s, t) => {
    const { alice } = await s.players({ alice: one }, true)

    const post_address = 'RANDOM'
    const params_get = { post_address }
    const result = await alice.call('app', 'blog', 'get_post', params_get)

    // should be Ok value but null
    // lookup did not error
    // successfully discovered the entry does not exity
    const entry = result.Ok
    t.same(entry, null)
  })
}
