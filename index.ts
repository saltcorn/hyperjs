import { Request, Response } from './hyperjs-core/index'
import { HyperJs } from './src-ts/hyper_js'

function hyperjs() {
  return new HyperJs()
}

export default hyperjs
module.exports = Object.assign(hyperjs, { Request, Response })
