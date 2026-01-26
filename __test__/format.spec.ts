// __test__/health.spec.ts
import test from 'ava'
import { ChildProcess } from 'node:child_process'
import axios from 'axios'

import * as server from './server.js'

let serverApp: ChildProcess
let port: number

test.before(async () => {
  const result = await server.start()
  serverApp = result.process
  port = result.port
})

test.after.always(() => {
  server.stop(serverApp)
})

test('/format', async (t) => {
  let res = await axios.get(`http://localhost:${port}/format`, {
    headers: {
      accept: 'text/plain',
    },
  })
  let data = res.data
  t.is(data, 'hey')

  //   res = await axios.get(`http://localhost:${port}/format`, {
  //     headers: {
  //       accept: 'text/html',
  //     },
  //   })
  //   data = res.data
  //   t.is(data, '<p>hey</p>')
})
