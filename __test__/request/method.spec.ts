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

test('/method - get', async (t) => {
  let res = await axios.get(`http://localhost:${port}/method`)
  let data = res.data
  t.is(data, 'GET')
})

test('/method - post', async (t) => {
  let res = await axios.post(`http://localhost:${port}/method`)
  let data = res.data
  t.is(data, 'POST')
})

test('/method - put', async (t) => {
  let res = await axios.put(`http://localhost:${port}/method`)
  let data = res.data
  t.is(data, 'PUT')
})

test('/delete - put', async (t) => {
  let res = await axios.delete(`http://localhost:${port}/method`)
  let data = res.data
  t.is(data, 'DELETE')
})
