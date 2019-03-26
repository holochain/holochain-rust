'use strict'
/**
 * A module for testing for and extracting names from options (e.g. `--one`, `-o`)
 */

class Arg {
  constructor (re) {
    this.re = re
  }

  test (arg) {
    return this.re.test(arg)
  }
}

exports.isShort = new Arg(/^-([^\d-])$/)
exports.isLong = new Arg(/^--(\S+)/)
exports.isCombined = new Arg(/^-([^\d-]{2,})$/)
exports.isOption = function (arg) {
  return this.isShort.test(arg) || this.isLong.test(arg) || this.isCombined.test(arg)
}
exports.optEquals = new Arg(/^(--\S+)=(.*)/)
