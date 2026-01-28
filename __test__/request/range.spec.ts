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

test('/range - invalid range', async (t) => {
  let res = await axios.get(`http://localhost:${port}/range`, {
    headers: {
      range: 'bytes=1000',
    },
  })
  let data = res.data
  t.is(data, -2)
})

test('/range - unsatisfiable range', async (t) => {
  let res = await axios.get(`http://localhost:${port}/range`, {
    headers: {
      range: 'bytes=1000-2000',
    },
  })
  let data = res.data
  t.is(data, -1)
})

test('/range', async (t) => {
  let res = await axios.get(`http://localhost:${port}/range`, {
    headers: {
      range: 'bytes=0-200,400-600',
    },
  })
  let data = res.data
  t.deepEqual(data, {
    rangeType: 'bytes',
    ranges: [
      {
        end: 200,
        start: 0,
      },
      {
        end: 600,
        start: 400,
      },
    ],
  })
})

test('/range - combine', async (t) => {
  let res = await axios.get(`http://localhost:${port}/range`, {
    headers: {
      range: 'bytes=0-4,90-99,5-75,100-199,101-102',
    },
  })
  let data = res.data
  t.deepEqual(data, {
    rangeType: 'bytes',
    ranges: [
      {
        end: 75,
        start: 0,
      },
      {
        end: 199,
        start: 90,
      },
    ],
  })
})
