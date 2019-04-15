
const defaultOpts = {timeout: 20000, interval: 100}
const identity = x => x

/**
 * Run a function at intervals until some condition is met,
 * capturing the result (or timeout event) in a Promise
 */
const pollFor = (
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
        if (t >= timeout) {
          reject(`pollFor timed out after ${timeout}ms`)
        } else {
          t += interval
          setTimeout(run, interval)
        }
      }
    }
    run()
  }
)

module.exports = {pollFor}
