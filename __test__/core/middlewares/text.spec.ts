// __test__/echo.spec.ts
import test from 'ava'
import { ChildProcess } from 'node:child_process'
import axios from 'axios'

import * as server from '../server-setup.js'

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

test('/echo - no header', async (t) => {
  const res = await axios.post(`http://localhost:${port}/echo`, 'ping')
  // Don't store the full response object, just extract the data
  const data = res.data
  t.is(data, 'OK')
})

test('/echo - text/plain header', async (t) => {
  const res = await axios.post(`http://localhost:${port}/echo`, 'ping', {
    headers: {
      'content-type': 'text/plain',
    },
  })
  const data = res.data
  t.is(data, 'ping')
})
