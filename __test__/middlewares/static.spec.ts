// __test__/echo.spec.ts
import test from 'ava'
import { ChildProcess } from 'node:child_process'
import axios from 'axios'

import * as server from '../server.js'
import { readFileSync } from 'node:fs'
import path from 'node:path'

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

test('fetch files in the public directory', async (t) => {
  const res = await axios.get(`http://localhost:${port}/public/index.html`)
  // Don't store the full response object, just extract the data
  const data = res.data
  let index_file_contents = readFileSync(path.join(__dirname, '../../public/index.html'), { encoding: 'utf-8' })
  t.is(data, index_file_contents)
})
