// __test__/echo.spec.ts
import test from 'ava'
import { ChildProcess } from 'node:child_process'
import axios from 'axios'

import * as server from '../server.js'

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

test('/raw-echo - no header', async (t) => {
  const res = await axios.post(`http://localhost:${port}/raw-echo`, 'ping')
  // Don't store the full response object, just extract the data
  const data = res.data
  t.is(data, 'OK')
})

test('/raw-echo - application/octet-stream header', async (t) => {
  const res = await axios.post(`http://localhost:${port}/raw-echo`, 'ping', {
    headers: {
      'content-type': 'application/octet-stream',
    },
  })
  const data = res.data
  t.is(res.headers['content-type'], 'application/octet-stream')
  t.deepEqual(data, 'ping')
})
