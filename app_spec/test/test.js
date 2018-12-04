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

  t.equal(result.Ok.value, JSON.stringify({"sum":"4"}))
})

test('get entry address', (t) => {
  t.plan(1)

  const params = {content: "Holo world"}
  const result = app.call("blog", "main", "post_address", params)

  t.equal(result.Ok, "QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk")
})

test('create_post', (t) => {
  t.plan(3)

  const content = "Holo world"
  const in_reply_to = null
  const params = {content, in_reply_to}
  const result = app.call("blog", "main", "create_post", params)

  t.ok(result.Ok)
  t.notOk(result.Err)
  t.equal(result.Ok, "QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk")
})

test('create_post with bad reply to', (t) => {
  t.plan(5)

  const content = "Holo world"
  const in_reply_to = "bad"
  const params = {content, in_reply_to}
  const result = app.call("blog", "main", "create_post", params)

  // bad in_reply_to is an error condition
  t.ok(result.Err)
  t.notOk(result.Ok)
  
  const error = JSON.parse(result.Err.error.Internal)
  t.deepEqual(error.kind, {ErrorGeneric: "Base for link not found"})
  t.ok(error.file)
  t.equal(error.line, "86")
})

test('post max content size 280 characters', (t) => {
  t.plan(5)

  const content = "Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum."
  const in_reply_to = null
  const params = {content, in_reply_to}
  const result = app.call("blog", "main", "create_post", params)

  // result should be an error
  t.ok(result.Err);
  t.notOk(result.Ok)

  const inner = JSON.parse(result.Err.error.Internal)

  t.ok(inner.file)
  t.deepEqual(inner.kind, {"ValidationFailed": "Content too long"})
  t.equals(inner.line, "86")
})

test('posts_by_agent', (t) => {
  t.plan(1)

  const agent = "Bob"
  const params = {agent}

  const result = app.call("blog", "main", "posts_by_agent", params)

  t.deepEqual(result.Ok, {"addresses":[]})
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

  t.equal(result.Ok.addresses.length, 2)
})

test('create/get_post roundtrip', (t) => {
  t.plan(2)

  const content = "Holo world"
  const in_reply_to = null
  const params = {content, in_reply_to}
  const create_post_result = app.call("blog", "main", "create_post", params)
  const post_address = create_post_result.Ok

  const params_get = {post_address}
  const result = app.call("blog", "main", "get_post", params_get)

  const entry_value = JSON.parse(result.Ok.App[1])
  t.equal(entry_value.content, content)
  t.equal(entry_value.date_created, "now")
})

test('get_post with non-existant address returns null', (t) => {
  t.plan(1)

  const post_address = "RANDOM"
  const params_get = {post_address}
  const result = app.call("blog", "main", "get_post", params_get)

  // should be Ok value but null
  // lookup did not error
  // successfully discovered the entry does not exity
  const entry = result.Ok
  t.same(entry, null)
})

test('scenario test create & publish post -> get from other instance', (t) => {
    t.plan(4)

    const content = "Holo world"
    const in_reply_to = null
    const params = {content, in_reply_to}
    const create_result = app.call("blog", "main", "create_post", params)

    t.equal(create_result.Ok.length, 46)
    t.equal(create_result.Ok, "QmY6MfiuhHnQ1kg7RwNZJNUQhwDxTFL45AAPnpJMNPEoxk")

    const post_address = create_result.Ok
    const params_get = {post_address}

    const check_get_result = function check_get_result (i = 0, get_result) {
      t.comment('checking get result for the ' + i + 'th time')

      if (get_result) {
        t.comment(JSON.stringify(get_result))
        const entry_value = JSON.parse(get_result.Ok.App[1])

        t.equal(entry_value.content, content)
        t.equal(entry_value.date_created, "now")
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
