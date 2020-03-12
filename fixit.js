#!/usr/bin/env node
'use strict'

const INIT_BUF_SIZE = 2

function get_times(cb) {
  let buf = ''

  const appendBuf = chunk => {
    buf = buf + chunk
  }

  const checkProcess = () => {
    const m = buf.match(/TEST_TIME\(([^)]*)\)\s+((?:\d+\.\d+|\d+))/)
    if (!m || m.length != 3) {
      return
    }
    buf = buf.substr(m.index + m[0].length)
    cb(m[1], parseFloat(m[2]))
    checkProcess()
  }

  process.stdin.setEncoding('utf8')

  process.stdin.on('readable', () => {
    let chunk = ''
    while ((chunk = process.stdin.read()) !== null) {
      appendBuf(chunk)
      checkProcess()
    }
  })

  process.stdin.on('end', () => {
    checkProcess()
  })

  process.stdin.on('error', e => {
    console.error(e)
    process.exit(1)
  })
}

function main() {
  let prev_name = null
  let prev_time = null
  get_times((name, time) => {
    if (name === prev_name) {
      return
    }
    if (prev_time) {
      console.log(parseInt((time - prev_time) * 1000) / 1000, 's', '|', name, time)
    } else {
      console.log('first timestamp', name, time)
    }
    prev_name = name
    prev_time = time
  })
}

// entry point
main()
