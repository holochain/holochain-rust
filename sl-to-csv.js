#!/usr/bin/env node

const READ_SIZE = 4096

const path = require('path')
const fs = require('fs')

/** Read a file extracting json data wrapped by <SL< json-data-here >SL>
 *  Does this in a streaming manner so works with huge log files
 *
 *  @param filename {string} - the file to parse
 *  @param cb {function} - called back with any parsed json objects
 */
function parse_file(filename, cb) {
  const file = fs.openSync(filename, 'r')

  const check_publish = buf => {
    const str = buf.toString('utf8').trim()
    if (str.length > 0) {
      try {
        const data = JSON.parse(str)
        cb(data)
      } catch (e) {
        console.error('failed to parse json:', e)
      }
    }
  }

  let state = 0
  let log_buffer = Buffer.alloc(READ_SIZE)
  let write_pos = 0
  const write_char = c => {
    if (log_buffer.byteLength <= write_pos) {
      const tmp = Buffer.alloc(log_buffer.byteLength * 2)
      log_buffer.copy(tmp)
      log_buffer = tmp
    }
    log_buffer.writeUInt8(c, write_pos)
    write_pos += 1
  }

  while (true) {
    const buf = Buffer.alloc(READ_SIZE)
    const read = fs.readSync(file, buf, 0, READ_SIZE)

    for (let i = 0; i < read; ++i) {
      const c = buf.readUInt8(i)
      if (state === 0) {
        if (c == '<'.charCodeAt(0)) {
          state = 1
        }
      } else if (state === 1) {
        if (c == 'S'.charCodeAt(0)) {
          state = 2
        } else {
          state = 0
        }
      } else if (state === 2) {
        if (c == 'L'.charCodeAt(0)) {
          state = 3
        } else {
          state = 0
        }
      } else if (state === 3) {
        if (c == '<'.charCodeAt(0)) {
          state = 4
          write_pos = 0
        } else {
          state = 0
        }
      } else if (state === 4) {
        if (c == '>'.charCodeAt(0)) {
          state = 5
        } else {
          write_char(c)
        }
      } else if (state === 5) {
        if (c == 'S'.charCodeAt(0)) {
          state = 6
        } else {
          state = 4
          write_char('>'.charCodeAt(0))
          write_char(c)
        }
      } else if (state === 6) {
        if (c == 'L'.charCodeAt(0)) {
          state = 7
        } else {
          state = 4
          write_char('>'.charCodeAt(0))
          write_char('S'.charCodeAt(0))
          write_char(c)
        }
      } else if (state === 7) {
        if (c == '>'.charCodeAt(0)) {
          check_publish(log_buffer.slice(0, write_pos))
          state = 0
          write_pos = 0
        } else {
          state = 4
          write_char('>'.charCodeAt(0))
          write_char('S'.charCodeAt(0))
          write_char('L'.charCodeAt(0))
          write_char(c)
        }
      }
    }

    if (read < READ_SIZE) {
      break
    }
  }

  check_publish(log_buffer.slice(0, write_pos))
}

/** Main function - either parse a file or fail with usage info
 */
function main() {
  if (process.argv.length < 3) {
    console.error('usage: sl-to-csv.js col1 col2 ... colN log-filename')
    process.exit(1)
  }

  const filename = path.resolve(process.argv[process.argv.length - 1])

  const header_list = []
  let header = '';
  for (let i = 2; i < process.argv.length - 1; ++i) {
    if (header.length > 0) {
      header += '\t'
    }
    header_list.push(process.argv[i])
    header += JSON.stringify(process.argv[i])
  }
  if (header.length > 0) {
    header += '\t'
  }
  header += JSON.stringify('__full_log__')
  console.log(header)

  parse_file(filename, data => {
    let line = ''
    for (let header of header_list) {
      const value = data.hasOwnProperty(header) ? data[header] : ''
      if (line.length > 0) {
        line += '\t'
      }
      line += JSON.stringify(value.toString())
    }
    if (line.length > 0) {
      line += '\t'
    }
    line += '"' + JSON.stringify(data).replace(/"/g, '""') + '"'
    console.log(line)
  })
}

// entry point
main()
