module.exports = scenario => {
      
      scenario('get_memo_returns_none', async (s, t, { alice, bob}) => {
      
        const content = "Reminder: Buy some HOT."
        const params = { content }
        const create_memo_result = await alice.app.call("blog", "create_memo", params)
      
        t.ok(create_memo_result.Ok)
        t.notOk(create_memo_result.Err)
        t.equal(create_memo_result.Ok, "QmV8f47UiisfMYxqpTe7DA65eLJ9jqNvaeTNSVPC7ZVd4i")
      
        const alice_get_memo_result = await alice.app.call("blog", "get_memo",
          { memo_address:create_memo_result.Ok })
      
        t.ok(alice_get_memo_result.Ok)
        t.notOk(alice_get_memo_result.Err)
        t.deepEqual(alice_get_memo_result.Ok,
          { App: [ 'memo', '{"content":"Reminder: Buy some HOT.","date_created":"now"}' ] })
      
        const bob_get_memo_result = await bob.app.call("blog", "get_memo",
          { memo_address:create_memo_result.Ok })
      
        t.equal(bob_get_memo_result.Ok, null)
        t.notOk(bob_get_memo_result.Err)
      
      })
      
      scenario('my_memos_are_private', async (s, t, { alice, bob }) => {
      
        const content = "Reminder: Buy some HOT."
        const params = { content }
        const create_memo_result = await alice.app.call("blog", "create_memo", params)
      
        t.ok(create_memo_result.Ok)
        t.notOk(create_memo_result.Err)
        t.equal(create_memo_result.Ok, "QmV8f47UiisfMYxqpTe7DA65eLJ9jqNvaeTNSVPC7ZVd4i")
      
        const alice_memos_result = await alice.app.call("blog", "my_memos", {})
      
        t.ok(alice_memos_result.Ok)
        t.notOk(alice_memos_result.Err)
        t.deepEqual(alice_memos_result.Ok,
          ["QmV8f47UiisfMYxqpTe7DA65eLJ9jqNvaeTNSVPC7ZVd4i"])
      
        const bob_memos_result = await bob.app.call("blog", "my_memos", {})
      
        t.ok(bob_memos_result.Ok)
        t.notOk(bob_memos_result.Err)
        t.deepEqual(bob_memos_result.Ok, [])
      
      })
      
}