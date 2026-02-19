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

test('/json-echo - no header', async (t) => {
  const res = await axios.post(`http://localhost:${port}/json-echo`, JSON.stringify({ greeting: 'Hello, world!' }))
  // Don't store the full response object, just extract the data
  const data = res.data
  t.is(data, 'OK')
})

test('/json-echo', async (t) => {
  const res = await axios.post(`http://localhost:${port}/json-echo`, JSON.stringify({ greeting: 'Hello, world!' }), {
    headers: {
      'content-type': 'application/json',
    },
  })
  // Don't store the full response object, just extract the data
  const data = res.data
  t.deepEqual(data, { greeting: 'Hello, world!' })
})
