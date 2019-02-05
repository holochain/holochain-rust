'use strict'
const Section = require('./section')
const Table = require('table-layout')
const ansi = require('ansi-escape-sequences')
const t = require('typical')
const arrayify = require('array-back')

class OptionList extends Section {
  constructor (data) {
    super()
    let definitions = arrayify(data.optionList)
    const hide = arrayify(data.hide)
    const groups = arrayify(data.group)

    /* filter out hidden definitions */
    if (hide.length) {
      definitions = definitions.filter(definition => {
        return hide.indexOf(definition.name) === -1
      })
    }

    if (data.header) this.header(data.header)

    if (groups.length) {
      definitions = definitions.filter(def => {
        const noGroupMatch = groups.indexOf('_none') > -1 && !t.isDefined(def.group)
        const groupMatch = intersect(arrayify(def.group), groups)
        if (noGroupMatch || groupMatch) return def
      })
    }

    const rows = definitions.map(def => {
      return {
        option: getOptionNames(def, 'bold', data.reverseNameOrder),
        description: ansi.format(def.description)
      }
    })

    const table = new Table(rows, {
      padding: { left: '  ', right: ' ' },
      columns: [
        { name: 'option', noWrap: true },
        { name: 'description', maxWidth: 80 }
      ]
    })
    this.add(table.renderLines())

    this.emptyLine()
  }
}

function getOptionNames (definition, optionNameStyles, reverseNameOrder) {
  let type = definition.type ? definition.type.name.toLowerCase() : ''
  const multiple = definition.multiple ? '[]' : ''
  if (type) {
    type = type === 'boolean' ? '' : `[underline]{${type}${multiple}}`
  }
  type = ansi.format(definition.typeLabel || type)

  let result = ''
  if (definition.alias) {
    if (reverseNameOrder) {
      result = `${ansi.format(`--${definition.name}`, optionNameStyles)}, ${ansi.format('-' + definition.alias, optionNameStyles)} ${type}`
    } else {
      result = `${ansi.format('-' + definition.alias, optionNameStyles)}, ${ansi.format(`--${definition.name} ${type}`, optionNameStyles)}`
    }
  } else {
    result = `${ansi.format(`--${definition.name}`, optionNameStyles)} ${type}`
  }
  return result
}

function intersect (arr1, arr2) {
  return arr1.some(function (item1) {
    return arr2.some(function (item2) {
      return item1 === item2
    })
  })
}

module.exports = OptionList
