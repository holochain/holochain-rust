const test = require('tape');
const Container = require('@holochain/holochain-nodejs');

const app = Container.loadAndInstantiate("../dist/app-spec-rust.hcpkg")
app.start()

test('call', (t) => {
  t.plan(1)

  const num1 = 2
  const num2 = 2
  const params = JSON.stringify({num1, num2})
  const result = app.call("blog", "main", "check_sum", params)

  t.equal(JSON.parse(result).value, JSON.stringify({"sum":"4"}))
})

test('get entry address', (t) => {
  t.plan(1)

  const params = JSON.stringify({content: "Holo world"})
  const result = app.call("blog", "main", "hash_post", params)

  t.equal(JSON.parse(result).address, "QmNndXfXcxqwsnAXdvbnzdZUS7bm4WqimY7w873C3Uttx1")
})

test('create_post', (t) => {
  t.plan(1)

  const content = "Holo world"
  const in_reply_to = ""
  const params = JSON.stringify({content, in_reply_to})
  const result = app.call("blog", "main", "create_post", params)
  t.equal(JSON.parse(result).address.length, 46) 
})

test('post max content size 280 characters', (t) => {
  t.plan(1)

  const content = "Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum."
  const in_reply_to = ""
  const params = JSON.stringify({content, in_reply_to})
  const result = app.call("blog", "main", "create_post", params)

  t.notEqual(JSON.parse(result).error, undefined)
})

test('posts_by_agent', (t) => {
  t.plan(1)

  const agent = "Bob"
  const params = JSON.stringify({agent})

  const result = app.call("blog", "main", "posts_by_agent", params)

  t.equal(result, JSON.stringify({"addresses":[]}))
})

test('my_posts', (t) => {
  t.plan(1)

  app.call("blog", "main", "create_post",
    JSON.stringify({"content": "Holo world", "in_reply_to": ""})
  )

  app.call("blog", "main", "create_post",
    JSON.stringify({"content": "Another post", "in_reply_to": ""})
  )

  const result = app.call("blog", "main", "my_posts", JSON.stringify({}))
  t.equal(JSON.parse(result).addresses.length, 2)
})


test('create/get_post rountrip', (t) => {
  t.plan(1)

  const content = "Holo world"
  const in_reply_to = ""
  const params = JSON.stringify({content, in_reply_to})
  const create_post_result = app.call("blog", "main", "create_post", params)
  const post_address = JSON.parse(create_post_result).address

  const params_get = JSON.stringify({post_address})
  const result = app.call("blog", "main", "get_post", params_get)

  const entry = JSON.parse(result)
  t.equal(entry.content, content)
})


test('get_post with non-existant hash returns null', (t) => {
  t.plan(1)

  const post_address = "RANDOM"
  const params_get = JSON.stringify({post_address})
  const result = app.call("blog", "main", "get_post", params_get)

  const entry = JSON.parse(result)
  t.same(entry, null)
})
