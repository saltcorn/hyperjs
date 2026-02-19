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

test('cookie setting and parsing', async (t) => {
  const res = await axios.get(`http://localhost:${port}/cookie/show`, {
    headers: {
      cookie: 'id=a3fWa; Expires=Wed, 21 Oct 2015 07:28:00 GMT',
    },
  })
  const data = res.data
  t.deepEqual(data, { id: 'a3fWa', Expires: 'Wed, 21 Oct 2015 07:28:00 GMT' })
})
