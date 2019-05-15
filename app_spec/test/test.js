const path = require('path')
const { Config, Conductor, Scenario } = require('../../nodejs_conductor')
Scenario.setTape(require('tape'))

const dnaPath = path.join(__dirname, "../dist/app_spec.dna.json")
const dna = Config.dna(dnaPath, 'app-spec')
const agentAlice = Config.agent("alice")
const agentBob = Config.agent("bob")
const agentCarol = Config.agent("carol")
const sleep = require("sleep");

const instanceAlice = Config.instance(agentAlice, dna)
const instanceBob = Config.instance(agentBob, dna)
const instanceCarol = Config.instance(agentCarol, dna)

const scenario1 = new Scenario([instanceAlice], { debugLog:true })
const scenario2 = new Scenario([instanceAlice, instanceBob], { debugLog: true })
const scenario3 = new Scenario([instanceAlice, instanceBob, instanceCarol], { debugLog: true })

scenario1.runTape('query capability using link tags', async (t, {alice}) => {
  const params = {content: "Queryable Blog Post w/Topics", in_reply_to: null, topics: ["holochain", "distributed", "holo"]}
  const result = await alice.callSync("blog", "create_post", params)
  t.equal(result.Ok, 'QmVdNzdFJnMyKJd6L3k1rNZqptUgwbzJoh78HEQ33z36qm')

  const query_result1 = await alice.callSync("blog", "get_posts_by_topic", {topics: ["Holochain"]})
  t.equal(query_result1.Ok, 'QmVdNzdFJnMyKJd6L3k1rNZqptUgwbzJoh78HEQ33z36qm')
  const query_result2 = await alice.callSync("blog", "get_posts_by_topic", {topics: ["Holo"]})
  t.equal(query_result2.Ok, 'QmVdNzdFJnMyKJd6L3k1rNZqptUgwbzJoh78HEQ33z36qm')
  const query_result3 = await alice.callSync("blog", "get_posts_by_topic", {topics: ["distributed"]})
  t.equal(query_result3.Ok, 'QmVdNzdFJnMyKJd6L3k1rNZqptUgwbzJoh78HEQ33z36qm')

  const query_result4 = await alice.callSync("blog", "get_posts_by_topic", {topics: ["Holochain", "holo"]})
  t.equal(query_result4.Ok, 'QmVdNzdFJnMyKJd6L3k1rNZqptUgwbzJoh78HEQ33z36qm')
  const query_result5 = await alice.callSync("blog", "get_posts_by_topic", {topics: ["distributed", "Holochain"]})
  t.equal(query_result5.Ok, 'QmVdNzdFJnMyKJd6L3k1rNZqptUgwbzJoh78HEQ33z36qm')

  const query_result6 = await alice.callSync("blog", "get_posts_by_topic", {base_topic: ["holo", "distributed"]})
  t.equal(query_result6.Ok, 'QmVdNzdFJnMyKJd6L3k1rNZqptUgwbzJoh78HEQ33z36qm')

  const query_result7 = await alice.callSync("blog", "get_posts_by_topic", {base_topic: ["Holochain", "holo", "distributed"]})
  t.equal(query_result7.Ok, 'QmVdNzdFJnMyKJd6L3k1rNZqptUgwbzJoh78HEQ33z36qm')
})

scenario2.runTape('capabilities grant and claim', async (t, { alice, bob }) => {

    // Ask for alice to grant a token for bob  (it's hard-coded for bob in re function for now)
    const result = alice.call("blog", "request_post_grant", {})
    t.ok(result.Ok)
    t.notOk(result.Err)

    // Confirm that we can get back the grant
    const grants = alice.call("blog", "get_grants", {})
    t.ok(grants.Ok)
    t.notOk(grants.Err)
    t.equal(result.Ok, grants.Ok[0])

    // Bob stores the grant as a claim
    const claim = bob.call("blog", "commit_post_claim", { grantor: alice.agentId, claim: result.Ok })
    t.deepEqual(claim, { Ok: 'Qmebh1y2kYgVG1RPhDDzDFTAskPcRWvz5YNhiNEi17vW9G' });

    // Bob can now create a post on alice's chain via a node-to-node message with the claim
    const post_content = "Holo world"
    const params = { grantor: alice.agentId, content: post_content, in_reply_to: null }
    const create_result = bob.call("blog", "create_post_with_claim", params)
    t.deepEqual(create_result, {Ok: "QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk"})

    // Confirm that the post was actually added to alice's chain
    const get_post_result = alice.call("blog", "get_post", { post_address: create_result.Ok })
    const value = JSON.parse(get_post_result.Ok.App[1])
    t.equal(value.content, post_content)


    // Check that when bob tries to make this call it fails because there is no grant stored
    const params2 = { grantor: bob.agentId, content: post_content, in_reply_to: null }
    const create2_result = bob.call("blog", "create_post_with_claim", params2)
    t.deepEqual(create2_result, {Ok: "error: no matching grant for claim"})

})

const testBridge = Config.bridge('test-bridge', instanceAlice, instanceBob)
const scenarioBridge = new Scenario([instanceAlice, instanceBob], { bridges: [testBridge], debugLog: true })

scenario2.runTape('sign_and_verify_message', async (t, { alice, bob }) => {
    const message = "Hello everyone! Time to start the secret meeting";

    const SignResult = bob.call("converse", "sign_message", { key_id:"", message: message });
    t.deepEqual(SignResult, { Ok: 'YVystBCmNEJGW/91bg43cUUybbtiElex0B+QWYy+PlB+nE3W8TThYGE4QzuUexvzkGqSutV04dSN8oyZxTJiBg==' });

    const provenance = [bob.agentId, SignResult.Ok];

    const VerificationResult = alice.call("converse", "verify_message", { message, provenance });
    t.deepEqual(VerificationResult, { Ok: true });
})

scenario2.runTape('secrets', async (t, { alice }) => {
    const ListResult = alice.call("converse", "list_secrets", { });
    // it should start out with the genesis made seed
    t.deepEqual(ListResult, { Ok: [ 'app_root_seed', 'primary_keybundle:enc_key', 'primary_keybundle:sign_key', 'root_seed' ]  });

    const AddSeedResult = alice.call("converse", "add_seed", {src_id: "app_root_seed", dst_id: "app_seed:1", index: 1 });
    t.ok(AddSeedResult)

    const AddKeyResult = alice.call("converse", "add_key", {src_id: "app_seed:1", dst_id: "app_key:1" });
    t.ok(AddKeyResult)

    const ListResult1 = alice.call("converse", "list_secrets", { });
    // it should start out with the genesis made seed
    t.deepEqual(ListResult1, { Ok: [ 'app_key:1', 'app_root_seed', 'app_seed:1', 'primary_keybundle:enc_key', 'primary_keybundle:sign_key', 'root_seed' ]  });

    const message = "Hello everyone! Time to start the secret meeting";

    const SignResult = alice.call("converse", "sign_message", { key_id:"app_key:1", message: message });
    t.ok(SignResult)

    // use the public key returned by add key as the provenance source
    const provenance = [AddKeyResult.Ok, SignResult.Ok];
    const VerificationResult = alice.call("converse", "verify_message", { message, provenance });
    t.deepEqual(VerificationResult, { Ok: true });

    // use the agent key as the provenance source (which should fail)
    const provenance1 = [alice.agentId, SignResult.Ok];
    const VerificationResult1 = alice.call("converse", "verify_message", { message, provenance: provenance1 });
    t.deepEqual(VerificationResult1, { Ok: false });

    const GetKeyResult = alice.call("converse", "get_pubkey", {src_id: "app_key:1" });
    t.ok(GetKeyResult)
    t.deepEqual(GetKeyResult,AddKeyResult)

})

scenario2.runTape('agentId', async (t, { alice, bob }) => {
  t.ok(alice.agentId)
  t.notEqual(alice.agentId, bob.agentId)
})

scenario1.runTape('show_env', async (t, { alice }) => {
  const result = alice.call("blog", "show_env", {})

  t.equal(result.Ok.dna_address, alice.dnaAddress)
  t.equal(result.Ok.dna_name, "HDK-spec-rust")
  t.equal(result.Ok.agent_address, alice.agentId)
  t.equal(result.Ok.agent_id, '{"nick":"alice","pub_sign_key":"' + alice.agentId + '"}')

  // don't compare the public token because it changes every time we change the dna.
  t.deepEqual(result.Ok.cap_request.provenance, [ alice.agentId, '+78GKy9y3laBbCNK1ajrj2rYVV3lBOxzGAZuuLDqXL2MLJUbMaB4lv7ut/UPWSoEeHx7OuXrTFXfu+PihtMMBQ==' ]
);

})

scenario3.runTape('get sources', async (t, { alice, bob, carol }) => {
  const params = { content: 'whatever', in_reply_to: null, topics: []}
  const address = await alice.callSync('blog', 'create_post', params).then(x => x.Ok)
  const address1 = await alice.callSync('blog', 'create_post', params).then(x => x.Ok)
  const address2 = await bob.callSync('blog', 'create_post', params).then(x => x.Ok)
  const address3 = await carol.callSync('blog', 'create_post', params).then(x => x.Ok)
  t.equal(address, address1)
  t.equal(address, address2)
  t.equal(address, address3)
  const sources1 = alice.call('blog', 'get_sources', { address }).Ok.sort()
  const sources2 = bob.call('blog', 'get_sources', { address }).Ok.sort()
  const sources3 = carol.call('blog', 'get_sources', { address }).Ok.sort()
  // NB: alice shows up twice because she published the same entry twice
  const expected = [alice.agentId, alice.agentId, bob.agentId, carol.agentId].sort()
  t.deepEqual(sources1, expected)
  t.deepEqual(sources2, expected)
  t.deepEqual(sources3, expected)
})

scenario1.runTape('cross zome call', async (t, { alice }) => {

  const num1 = 2
  const num2 = 2
  const params = { num1, num2 }
  const result = alice.call("blog", "check_sum", params)
  t.notOk(result.Err)
  t.equal(result.Ok, 4)
})

scenario2.runTape('send ping', async (t, { alice, bob }) => {
  const params = { to_agent: bob.agentId, message: "hello" }
  const result = alice.call("blog", "ping", params)
    t.deepEqual(result, { Ok: { msg_type:"response", body: "got hello from HcScjwO9ji9633ZYxa6IYubHJHW6ctfoufv5eq4F7ZOxay8wR76FP4xeG9pY3ui" } })
})

scenario1.runTape('hash_post', async (t, { alice }) => {

  const params = { content: "Holo world" }
  const result = alice.call("blog", "post_address", params)

  t.equal(result.Ok, "QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk")
})

scenario1.runTape('hash_memo', async (t, { alice }) => {

  const params = { content: "Reminder: Buy some HOT." }
  const result = alice.call("blog", "memo_address", params)

  t.equal(result.Ok, "QmV8f47UiisfMYxqpTe7DA65eLJ9jqNvaeTNSVPC7ZVd4i")
})

scenario1.runTape('create_post', async (t, { alice }) => {

  const content = "Holo world"
  const in_reply_to = null
  const topics = [];
  const params = { content, in_reply_to, topics}
  const result = alice.call("blog", "create_post", params)

  t.ok(result.Ok)
  t.notOk(result.Err)
  t.equal(result.Ok, "QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk")
})

scenario2.runTape('create_post_countersigned', async (t, { alice, bob }) => {

  const content = "Holo world"
  const in_reply_to = null

  const address_params = { content }
  const address_result = bob.call("blog", "post_address", address_params)

  t.ok(address_result.Ok)
  const SignResult = bob.call("converse", "sign_message", { key_id:"", message: address_result.Ok });
  t.ok(SignResult.Ok)

  const counter_signature = [bob.agentId, SignResult.Ok];

  const params = { content, in_reply_to, counter_signature }
  const result = alice.call("blog", "create_post_countersigned", params)

  t.ok(result.Ok)
  t.notOk(result.Err)
  t.equal(result.Ok, "QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk")
})


scenario1.runTape('create_memo', async (t, { alice }) => {

  const content = "Reminder: Buy some HOT."
  const params = { content }
  const result = alice.call("blog", "create_memo", params)

  t.ok(result.Ok)
  t.notOk(result.Err)
  t.equal(result.Ok, "QmV8f47UiisfMYxqpTe7DA65eLJ9jqNvaeTNSVPC7ZVd4i")
})

scenario1.runTape('my_memos', async (t, { alice }) => {

  const content = "Reminder: Buy some HOT."
  const params = { content }
  const create_memo_result = alice.call("blog", "create_memo", params)

  t.ok(create_memo_result.Ok)
  t.notOk(create_memo_result.Err)
  t.equal(create_memo_result.Ok, "QmV8f47UiisfMYxqpTe7DA65eLJ9jqNvaeTNSVPC7ZVd4i")

  const my_memos_result = alice.call("blog", "my_memos", {})

  t.ok(my_memos_result.Ok)
  t.notOk(my_memos_result.Err)
  t.deepEqual(my_memos_result.Ok, ["QmV8f47UiisfMYxqpTe7DA65eLJ9jqNvaeTNSVPC7ZVd4i"])
})


scenario2.runTape('get_memo_returns_none', async (t, { alice, bob}) => {

  const content = "Reminder: Buy some HOT."
  const params = { content }
  const create_memo_result = alice.call("blog", "create_memo", params)

  t.ok(create_memo_result.Ok)
  t.notOk(create_memo_result.Err)
  t.equal(create_memo_result.Ok, "QmV8f47UiisfMYxqpTe7DA65eLJ9jqNvaeTNSVPC7ZVd4i")

  const alice_get_memo_result = alice.call("blog", "get_memo",
    { memo_address:create_memo_result.Ok })

  t.ok(alice_get_memo_result.Ok)
  t.notOk(alice_get_memo_result.Err)
  t.deepEqual(alice_get_memo_result.Ok,
    { App: [ 'memo', '{"content":"Reminder: Buy some HOT.","date_created":"now"}' ] })

  const bob_get_memo_result = bob.call("blog", "get_memo",
    { memo_address:create_memo_result.Ok })

  t.equal(bob_get_memo_result.Ok, null)
  t.notOk(bob_get_memo_result.Err)

})

scenario2.runTape('my_memos_are_private', async (t, { alice, bob }) => {

  const content = "Reminder: Buy some HOT."
  const params = { content }
  const create_memo_result = alice.call("blog", "create_memo", params)

  t.ok(create_memo_result.Ok)
  t.notOk(create_memo_result.Err)
  t.equal(create_memo_result.Ok, "QmV8f47UiisfMYxqpTe7DA65eLJ9jqNvaeTNSVPC7ZVd4i")

  const alice_memos_result = alice.call("blog", "my_memos", {})

  t.ok(alice_memos_result.Ok)
  t.notOk(alice_memos_result.Err)
  t.deepEqual(alice_memos_result.Ok,
    ["QmV8f47UiisfMYxqpTe7DA65eLJ9jqNvaeTNSVPC7ZVd4i"])

  const bob_memos_result = bob.call("blog", "my_memos", {})

  t.ok(bob_memos_result.Ok)
  t.notOk(bob_memos_result.Err)
  t.deepEqual(bob_memos_result.Ok, [])

})


scenario2.runTape('delete_post', async (t, { alice, bob }) => {

  //create post
  const alice_create_post_result = await alice.callSync("blog", "create_post",
    { "content": "Posty", "in_reply_to": "", "topics": [] }
  )

  const bob_create_post_result = await bob.callSync("blog", "posts_by_agent",
    { "agent": alice.agentId }
  )

  t.ok(bob_create_post_result.Ok)
  t.equal(bob_create_post_result.Ok.links.length, 1);

  //remove link by alicce
  await alice.callSync("blog", "delete_post", { "content": "Posty", "in_reply_to": "" })

  // get posts by bob
  const bob_agent_posts_expect_empty = bob.call("blog", "posts_by_agent", { "agent": alice.agentId })

  t.ok(bob_agent_posts_expect_empty.Ok)
  t.equal(bob_agent_posts_expect_empty.Ok.links.length, 0);
})

scenario2.runTape('delete_entry_post', async (t, { alice, bob }) => {
  const content = "Hello Holo world 321"
  const in_reply_to = null
  const params = { content, in_reply_to }

  //commit create_post
  const createResult = await alice.callSync("blog", "create_post", params)

  t.ok(createResult.Ok)


  //delete entry post
  const deletionParams = { post_address: createResult.Ok }
  const deletionResult = await alice.callSync("blog", "delete_entry_post", deletionParams)

  t.ok(deletionResult.Ok)


  //delete should fail
  const failedDelete = await bob.callSync("blog", "delete_entry_post", { post_address: createResult.Ok })
  t.deepEqual(failedDelete.Err, { Internal: 'Entry Could Not Be Found' });

  //get initial entry
  const GetInitialParamsResult = bob.call("blog", "get_initial_post", { post_address: createResult.Ok })
  t.deepEqual(JSON.parse(GetInitialParamsResult.Ok.App[1]), { content: "Hello Holo world 321", date_created: "now" });

  const entryWithOptionsGet = { post_address: createResult.Ok }
  const entryWithOptionsGetResult = bob.call("blog", "get_post_with_options", entryWithOptionsGet);
  t.deepEqual(JSON.parse(entryWithOptionsGetResult.Ok.result.All.items[0].entry.App[1]), { content: "Hello Holo world 321", date_created: "now" })
})

scenario2.runTape('update_entry_validation', async (t, { alice, bob }) => {
  //update entry does not exist
  const updateParams = { post_address: "1234", new_content: "Hello Holo V2" }
  const UpdateResult = await bob.callSync("blog", "update_post", updateParams)

  t.deepEqual(UpdateResult.Err, { Internal: 'failed to update post' });

  const content = "Hello Holo world 321"
  const in_reply_to = null
  const topics = []
  const params = { content, in_reply_to, topics}

  //commit create_post
  const createResult = await alice.callSync("blog", "create_post", params)

  t.ok(createResult.Ok)

  const updateParamsV2 = { post_address: createResult.Ok, new_content: "Hello Holo world 321" }
  const UpdateResultV2 = await bob.callSync("blog", "update_post", updateParamsV2)
  t.deepEqual(JSON.parse(UpdateResultV2.Err.Internal).kind.ValidationFailed, "Trying to modify with same data");


})

scenario2.runTape('update_post', async (t, { alice, bob }) => {
  const content = "Hello Holo world 123"
  const in_reply_to = null
  const topics = []
  const params = { content, in_reply_to, topics }

  //commit version 1
  const createResult = await alice.callSync("blog", "create_post", params)
  t.ok(createResult.Ok)

  //get v1
  const updatedPostV1 = alice.call("blog", "get_post", { post_address: createResult.Ok })
  const UpdatePostV1Content = { content: "Hello Holo world 123", date_created: "now" };
  t.ok(updatedPostV1.Ok)
  t.deepEqual(JSON.parse(updatedPostV1.Ok.App[1]), UpdatePostV1Content)

  //update to version 2
  const updatePostContentV2 = { content: "Hello Holo V2", date_created: "now" };
  const updateParamsV2 = { post_address: createResult.Ok, new_content: "Hello Holo V2" }
  const UpdateResultV2 = await bob.callSync("blog", "update_post", updateParamsV2)
  t.ok(UpdateResultV2.Ok)
  t.notOk(UpdateResultV2.Err)

  //get v2 using initial adderss
  const updatedPostv2Initial = alice.call("blog", "get_post", { post_address: createResult.Ok })
  t.ok(updatedPostv2Initial.Ok)
  t.notOk(updatedPostv2Initial.Err)
  t.deepEqual(JSON.parse(updatedPostv2Initial.Ok.App[1]), updatePostContentV2)

  //get v2 latest address
  const updatedPostv2Latest = alice.call("blog", "get_post", { post_address: UpdateResultV2.Ok })
  t.ok(updatedPostv2Latest.Ok)
  t.notOk(updatedPostv2Latest.Err)
  t.deepEqual(JSON.parse(updatedPostv2Latest.Ok.App[1]), updatePostContentV2)


  //get v1 using initial address
  const GetInitialPostV1Initial = alice.call("blog", "get_initial_post", { post_address: createResult.Ok })
  t.ok(GetInitialPostV1Initial.Ok)
  t.notOk(GetInitialPostV1Initial.Err)
  t.deepEqual(JSON.parse(GetInitialPostV1Initial.Ok.App[1]), { content: "Hello Holo world 123", date_created: "now" })

  //get v2 latest address
  const GetInitialPostV2Latest = alice.call("blog", "get_initial_post", { post_address: UpdateResultV2.Ok })
  t.ok(GetInitialPostV2Latest.Ok)
  t.notOk(GetInitialPostV2Latest.Err)
  t.deepEqual(JSON.parse(GetInitialPostV2Latest.Ok.App[1]), updatePostContentV2)

  //update to version 3
  const UpdatePostV3Content = { content: "Hello Holo V3", date_created: "now" };
  const updateParamsV3 = { post_address: createResult.Ok, new_content: "Hello Holo V3" }
  const UpdateResultV3 = await alice.callSync("blog", "update_post", updateParamsV3)
  t.ok(UpdateResultV3.Ok)
  t.notOk(UpdateResultV3.Err)

  //get v3 using initial adderss
  const updatedPostV3Initial = alice.call("blog", "get_post", { post_address: createResult.Ok })
  t.ok(updatedPostV3Initial.Ok)
  t.notOk(updatedPostV3Initial.Err)
  t.deepEqual(JSON.parse(updatedPostV3Initial.Ok.App[1]), UpdatePostV3Content)

  //get v3 using address of v2
  const updatedPostV3Latest = alice.call("blog", "get_post", { post_address: UpdateResultV2.Ok })
  t.ok(updatedPostV3Latest.Ok)
  t.notOk(updatedPostV3Latest.Err)
  t.deepEqual(JSON.parse(updatedPostV3Latest.Ok.App[1]), UpdatePostV3Content)

  //update to version 4
  const updatePostV4Content = { content: "Hello Holo V4", date_created: "now" };
  const updateParamsV4 = { post_address: createResult.Ok, new_content: "Hello Holo V4" }
  const UpdateResultV4 = await alice.callSync("blog", "update_post", updateParamsV4)
  t.notOk(UpdateResultV4.Err)
  t.ok(UpdateResultV4.Ok)

  //get history entry v4
  const entryHistoryV4Params = { post_address: UpdateResultV4.Ok }
  const entryHistoryV4 = alice.call("blog", "get_history_post", entryHistoryV4Params)
  t.ok(UpdateResultV4.Ok)
  t.notOk(UpdateResultV4.Err)
  t.deepEqual(entryHistoryV4.Ok.items.length, 1);
  t.deepEqual(JSON.parse(entryHistoryV4.Ok.items[0].entry.App[1]), updatePostV4Content);
  t.deepEqual(entryHistoryV4.Ok.items[0].meta.address, UpdateResultV4.Ok);
  t.deepEqual(entryHistoryV4.Ok.items[0].meta.crud_status, "live");

  //get history entry all
  const entryHistoryAllParams = { post_address: createResult.Ok }
  const entryHistoryAll = alice.call("blog", "get_history_post", entryHistoryAllParams)

  t.deepEqual(entryHistoryAll.Ok.items.length, 4);
  t.deepEqual(JSON.parse(entryHistoryAll.Ok.items[0].entry.App[1]), { content: "Hello Holo world 123", date_created: "now" });
  t.deepEqual(entryHistoryAll.Ok.items[0].meta.address, createResult.Ok);
  t.deepEqual(entryHistoryAll.Ok.items[0].meta.crud_status, "modified");
  t.deepEqual(entryHistoryAll.Ok.crud_links[createResult.Ok], UpdateResultV2.Ok)

  t.deepEqual(JSON.parse(entryHistoryAll.Ok.items[1].entry.App[1]), updatePostContentV2);
  t.deepEqual(entryHistoryAll.Ok.items[1].meta.address, UpdateResultV2.Ok);
  t.deepEqual(entryHistoryAll.Ok.items[1].meta.crud_status, "modified");
  t.deepEqual(entryHistoryAll.Ok.crud_links[UpdateResultV2.Ok], UpdateResultV3.Ok)

  t.deepEqual(JSON.parse(entryHistoryAll.Ok.items[2].entry.App[1]), UpdatePostV3Content);
  t.deepEqual(entryHistoryAll.Ok.items[2].meta.address, UpdateResultV3.Ok);
  t.deepEqual(entryHistoryAll.Ok.items[2].meta.crud_status, "modified");
  t.deepEqual(entryHistoryAll.Ok.crud_links[UpdateResultV3.Ok], UpdateResultV4.Ok)

  t.deepEqual(JSON.parse(entryHistoryAll.Ok.items[3].entry.App[1]), updatePostV4Content);
  t.deepEqual(entryHistoryAll.Ok.items[3].meta.address, UpdateResultV4.Ok);
  t.deepEqual(entryHistoryAll.Ok.items[3].meta.crud_status, "live");
  t.notOk(entryHistoryAll.Ok.crud_links[UpdateResultV4.Ok])

  const entryWithOptionsGet = { post_address: createResult.Ok }
  const entryWithOptionsGetResult = alice.call("blog", "get_post_with_options_latest", entryWithOptionsGet);

  t.deepEqual(JSON.parse(entryWithOptionsGetResult.Ok.App[1]), updatePostV4Content);
})


scenario2.runTape('remove_update_modifed_entry', async (t, { alice, bob }) => {
  const content = "Hello Holo world 123"
  const in_reply_to = null
  const topics = []
  const params = { content, in_reply_to, topics }

  //commit version 1
  const createResult = await alice.callSync("blog", "create_post", params)
  t.ok(createResult.Ok)
  //get entry
  const updatedPostV1 = alice.call("blog", "get_post", { post_address: createResult.Ok })
  t.ok(updatedPostV1.Ok)
  t.deepEqual(JSON.parse(updatedPostV1.Ok.App[1]), { content: "Hello Holo world 123", date_created: "now" })

  //delete
  const removeParamsV2 = { post_address: createResult.Ok }
  const removeResultV2 = await bob.callSync("blog", "delete_entry_post", removeParamsV2)
  t.ok(removeResultV2.Ok)

  //get v2 using initial adders
  const Postv2Initial = alice.call("blog", "get_initial_post", { post_address: createResult.Ok })
  t.ok(Postv2Initial.Ok)
  t.deepEqual(JSON.parse(Postv2Initial.Ok.App[1]), { content: "Hello Holo world 123", date_created: "now" })

  //failed delete
  const failedDelete = await alice.callSync("blog", "delete_entry_post", { post_address: createResult.Ok })
  t.deepEqual(failedDelete.Err, { Internal: 'Entry Could Not Be Found' });
})

scenario1.runTape('create_post with bad reply to', async (t, { alice }) => {
  const content = "Holo world"
  const in_reply_to = "bad"
  const topics = []
  const params = { content, in_reply_to, topics }
  const result = alice.call("blog", "create_post", params)

  // bad in_reply_to is an error condition
  t.ok(result.Err)
  t.notOk(result.Ok)
  const error = JSON.parse(result.Err.Internal)
  t.deepEqual(error.kind, { ErrorGeneric: "Base for link not found" })
  t.ok(error.file)
  t.ok(error.line)
})

scenario2.runTape('delete_post_with_bad_link', async (t, { alice, bob }) => {

  const result_bob_delete = await bob.callSync("blog", "delete_post",
    { "content": "Bad" }
  )

  // bad in_reply_to is an error condition
  t.ok(result_bob_delete.Err)
  t.notOk(result_bob_delete.Ok)
  const error = JSON.parse(result_bob_delete.Err.Internal)
  t.deepEqual(error.kind, { ErrorGeneric: "Target for link not found" })
  t.ok(error.file)
  t.ok(error.line)
})

scenario1.runTape('post max content size 280 characters', async (t, { alice }) => {

  const content = "Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum."
  const in_reply_to = null
  const topics = []
  const params = { content, in_reply_to, topics }
  const result = alice.call("blog", "create_post", params)

  // result should be an error
  t.ok(result.Err);
  t.notOk(result.Ok)

  const inner = JSON.parse(result.Err.Internal)

  t.ok(inner.file)
  t.deepEqual(inner.kind, { "ValidationFailed": "Content too long" })
  t.ok(inner.line)
})

scenario1.runTape('posts_by_agent', async (t, { alice }) => {

  const agent = "Bob"
  const params = { agent }

  const result = alice.call("blog", "posts_by_agent", params)

  t.deepEqual(result.Ok, { links : []})
})

scenario1.runTape('my_posts', async (t, { alice }) => {

  await alice.callSync("blog", "create_post",
    { "content": "Holo world", "in_reply_to": "", "topics": "[]" }
  )

  await alice.callSync("blog", "create_post",
    { "content": "Another post", "in_reply_to": "", "topics": "[]" }
  )

  const result = alice.call("blog", "my_posts", {})

  t.equal(result.Ok.links.length, 2)
})


scenario1.runTape('my_posts_immediate_timeout', async (t, { alice }) => {

  alice.call("blog", "create_post",
    { "content": "Holo world", "in_reply_to": "", "topics": "[]" }
  )

  const result = alice.call("blog", "my_posts_immediate_timeout", {})

  t.ok(result.Err)
  console.log(result)
  t.equal(JSON.parse(result.Err.Internal).kind, "Timeout")
})

scenario2.runTape('get_sources_from_link', async (t, { alice, bob }) => {

  await alice.callSync("blog", "create_post",
    { "content": "Holo world", "in_reply_to": null, "topics": [] }
  );

  await bob.callSync("blog", "create_post",
  { "content": "Another one", "in_reply_to": null, "topics": [] }
);
  const alice_posts = bob.call("blog","authored_posts_with_sources",
  {
    "agent" : alice.agentId
  });

  const bob_posts = alice.call("blog","authored_posts_with_sources",
  {
    "agent" : bob.agentId
  });

  t.equal(bob.agentId,bob_posts.Ok.links[0].headers[0].provenances[0][0]);
  t.equal(alice.agentId,alice_posts.Ok.links[0].headers[0].provenances[0][0]);

})

scenario2.runTape('get_sources_after_same_link', async (t, { alice, bob }) => {

  await bob.callSync("blog", "create_post_with_agent",
    { "agent_id": alice.agentId ,"content": "Holo world", "in_reply_to": null }
  );
  await alice.callSync("blog", "create_post_with_agent",
  { "agent_id": alice.agentId ,"content": "Holo world", "in_reply_to": null }
  );

  const alice_posts = bob.call("blog","authored_posts_with_sources",
  {
    "agent" : alice.agentId
  });
  const bob_posts = alice.call("blog","authored_posts_with_sources",
  {
    "agent" : alice.agentId
  });

  t.equal(bob.agentId,alice_posts.Ok.links[0].headers[0].provenances[0][0]);
  t.equal(alice.agentId,alice_posts.Ok.links[0].headers[1].provenances[0][0]);
  t.equal(bob.agentId,bob_posts.Ok.links[0].headers[1].provenances[0][0]);
  t.equal(alice.agentId,bob_posts.Ok.links[0].headers[0].provenances[0][0]);

})

scenario1.runTape('create/get_post roundtrip', async (t, { alice }) => {

  const content = "Holo world"
  const in_reply_to = null
  const topics = []
  const params = { content, in_reply_to, topics }
  const create_post_result = alice.call("blog", "create_post", params)
  const post_address = create_post_result.Ok

  const params_get = { post_address }
  const result = alice.call("blog", "get_post", params_get)

  const entry_value = JSON.parse(result.Ok.App[1])
  t.comment("get_post() entry_value = " + entry_value + "")
  t.equal(entry_value.content, content)
  t.equal(entry_value.date_created, "now")

})

scenario1.runTape('get_post with non-existant address returns null', async (t, { alice }) => {

  const post_address = "RANDOM"
  const params_get = { post_address }
  const result = alice.call("blog", "get_post", params_get)

  // should be Ok value but null
  // lookup did not error
  // successfully discovered the entry does not exity
  const entry = result.Ok
  t.same(entry, null)
})

scenario2.runTape('scenario test create & publish post -> get from other instance', async (t, { alice, bob }) => {

  const initialContent = "Holo world"
  const params = { content: initialContent, in_reply_to: null, topics: [] }
  const create_result = await alice.callSync("blog", "create_post", params)

  const params2 = { content: "post 2", in_reply_to: null, topics: [] }
  const create_result2 = await bob.callSync("blog", "create_post", params2)

  t.equal(create_result.Ok.length, 46)
  t.equal(create_result.Ok, "QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk")

  const post_address = create_result.Ok
  const params_get = { post_address }

  const result = bob.call("blog", "get_post", params_get)
  const value = JSON.parse(result.Ok.App[1])
  t.equal(value.content, initialContent)
})

scenarioBridge.runTape('scenario test create & publish -> getting post via bridge', async (t, {alice, bob}) => {

  const initialContent = "Holo world"
  const params = { content: initialContent, in_reply_to: null, topics: [] }
  const create_result = await bob.callSync("blog", "create_post", params)

  t.equal(create_result.Ok, "QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk")

  const post_address = create_result.Ok
  const params_get = { post_address }

  const result = alice.call("blog", "get_post_bridged", params_get)
  console.log("BRIDGE CALL RESULT: " + JSON.stringify(result))
  const value = JSON.parse(result.Ok.App[1])
  t.equal(value.content, initialContent)
})

scenario2.runTape('request grant', async (t, { alice, bob }) => {

    /*
      This is not a complete test of requesting a grant because currently there
      is no way in the test conductor to actually pass in the provenance of the
      call.  That will be added when we convert the test framework to being built
      on top of the rust conductor.   For now this is more a placeholder test, but
      note that the value returned is actually the capbability token value.
    */
    const result = alice.call("blog", "request_post_grant", {})
    t.ok(result.Ok)
    t.notOk(result.Err)

    const grants = alice.call("blog", "get_grants", {})
    t.ok(grants.Ok)
    t.notOk(grants.Err)

    t.equal(result.Ok, grants.Ok[0])
})
