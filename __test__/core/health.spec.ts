// __test__/health.spec.ts
import test from 'ava'
import { ChildProcess } from 'node:child_process'
import axios from 'axios'

import * as server from './server-setup.js'

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

test('/health', async (t) => {
  const res = await axios.get(`http://localhost:${port}/health`)
  const data = res.data
  t.is(data, 'OK')
})
