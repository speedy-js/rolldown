const { join } = require('path')

const { rolldown } = require('./binding')

module.exports.rolldown = function (entry, options = {}) {
  return rolldown(entry, Buffer.from(JSON.stringify(options)))
}
