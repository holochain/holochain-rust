
const defaultOpts = {timeout: 2000, interval: 100}
const identity = x => x

const waitFor = (
  fn, 
  pred = identity, 
  {timeout, interval} = defaultOpts
) => new Promise(
  (fulfill, reject) => {
    let t = 0
    let timer = null
    const run = () => {
      const val = fn()
      if (pred(val)) {
        fulfill(val)
      } else {
        console.log(t)
        if (t >= timeout) {
          reject()
        } else {
          t += interval
          setTimeout(run, interval)
        }
      }
    }
    run()
  }
)

module.exports = {waitFor}
