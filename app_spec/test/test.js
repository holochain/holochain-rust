const test = require('tape');
const Container = require('../../nodejs_container');

const app = Container.instanceFromNameAndDna("bob", "dist/app_spec.hcpkg")
app.start()

const app2 = Container.instanceFromNameAndDna("alice", "dist/app_spec.hcpkg")
app2.start()

test('call', (t) => {
  t.plan(1)

  const num1 = 2
  const num2 = 2
  const params = {num1, num2}
  const result = app.call("blog", "main", "check_sum", params)

  t.equal(result.value, JSON.stringify({"sum":"4"}))
})

test('hash_post', (t) => {
  t.plan(1)

  const params = {content: "Holo world"}
  const result = app.call("blog", "main", "hash_post", params)

  t.equal(result.address, "QmNndXfXcxqwsnAXdvbnzdZUS7bm4WqimY7w873C3Uttx1")
})

test('create_post', (t) => {
  t.plan(1)

  const content = "Holo world"
  const in_reply_to = ""
  const params = {content, in_reply_to}
  const result = app.call("blog", "main", "create_post", params)
  t.equal(result.address.length, 46)
})

test('post max content size 280 characters', (t) => {
  t.plan(1)

  const content = "Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum."
  const in_reply_to = ""
  const params = {content, in_reply_to}
  const result = app.call("blog", "main", "create_post", params)

  t.notEqual(result.error, undefined)
})

test('posts_by_agent', (t) => {
  t.plan(1)

  const agent = "Bob"
  const params = {agent}

  const result = app.call("blog", "main", "posts_by_agent", params)

  t.deepEqual(result, {"addresses":[]})
})

test('my_posts', (t) => {
  t.plan(1)

  app.call("blog", "main", "create_post",
    {"content": "Holo world", "in_reply_to": ""}
  )

  app.call("blog", "main", "create_post",
    {"content": "Another post", "in_reply_to": ""}
  )

  const result = app.call("blog", "main", "my_posts", {})
  t.equal(result.addresses.length, 2)
})


test('create/get_post rountrip', (t) => {
  t.plan(1)

  const content = "Holo world"
  const in_reply_to = ""
  const params = {content, in_reply_to}
  const create_post_result = app.call("blog", "main", "create_post", params)
  const post_address = create_post_result.address

  const params_get = {post_address}
  const result = app.call("blog", "main", "get_post", params_get)

    t.comment("get_post() result = " + get_result)
  const entry = result
  t.equal(entry.content, content)
})


test('get_post with non-existant hash returns null', (t) => {
  t.plan(1)

  const post_address = "RANDOM"
  const params_get = {post_address}
  const result = app.call("blog", "main", "get_post", params_get)

  const entry = result
  t.same(entry, null)
})

// this test is flaky!
// even when we loop and wait sometimes app2 never sees the published entry
test('scenario test create & publish post -> get from other instance', (t) => {
    t.plan(3)

    const content = "Holo world"
    const in_reply_to = ""
    const params = {content, in_reply_to}
    const create_result = app.call("blog", "main", "create_post", params)

    t.equal(create_result.address.length, 46)
    t.equal(create_result.address, "QmNndXfXcxqwsnAXdvbnzdZUS7bm4WqimY7w873C3Uttx1")

    const post_address = create_result.address
    const params_get = {post_address}
    t.comment(t.comment("params_get = " + params_get)
    const check_get_result = function check_get_result (i = 0, get_result) {
      t.comment('checking get result for the ' + i + 1 + 'th time')
      t.comment(t.comment("\t -> result = " + get_result)

      if (get_result) {
        t.equal(get_result.content, content)
      }
      else if (i < 50) {
        setTimeout(function() {
          check_get_result(
            ++i,
            app2.call("blog", "main", "get_post", params_get)
          )
        }, 100)
      }
      else {
        t.end()
      }

    }()
})
