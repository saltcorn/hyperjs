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

test('/urlencoded - no header', async (t) => {
  const res = await axios.post(
    `http://localhost:${port}/urlencoded`,
    { greeting: 'Hello, world!' },
    {
      headers: {
        'content-type': '',
      },
    },
  )
  // Don't store the full response object, just extract the data
  const data = res.data
  t.is(data, 'OK')
})

test('/urlencoded', async (t) => {
  const res = await axios.post(
    `http://localhost:${port}/urlencoded`,
    { greeting: 'Hello, world!' },
    {
      headers: {
        'content-type': 'application/x-www-form-urlencoded',
      },
    },
  )
  // Don't store the full response object, just extract the data
  const data = res.data
  t.deepEqual(data, { greeting: 'Hello, world!' })
})
