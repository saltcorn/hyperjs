// __test__/health.spec.ts
import test from 'ava'
import { ChildProcess } from 'node:child_process'
import axios, { isAxiosError } from 'axios'

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

test('/redirect', async (t) => {
  try {
    await axios.get(`http://localhost:${port}/redirect`, {
      headers: {
        accept: 'text/plain',
      },
      maxRedirects: 0,
    })
    t.fail('Expected request to fail.')
  } catch (e) {
    if (isAxiosError(e)) {
      let data = e.response?.data
      t.is(e.response?.status, 302)
      t.is(data, 'Found. Redirecting to https://example.com')
    } else {
      t.fail('Expected an AxiosError.')
    }
  }

  try {
    await axios.get(`http://localhost:${port}/redirect`, {
      headers: {
        accept: 'text/html',
      },
      maxRedirects: 0,
    })
    t.fail('Expected request to fail.')
  } catch (e) {
    if (isAxiosError(e)) {
      let data = e.response?.data
      t.is(e.response?.status, 302)
      t.is(
        data,
        '<!DOCTYPE html><head><title>Found</title></head><body><p>Found. Redirecting to https://example.com</p></body>',
      )
    } else {
      t.fail('Expected an AxiosError.')
    }
  }

  try {
    await axios.get(`http://localhost:${port}/redirect`, {
      headers: {
        accept: 'application/unknown',
      },
      maxRedirects: 0,
    })
    t.fail('Expected request to fail.')
  } catch (e) {
    if (isAxiosError(e)) {
      let data = e.response?.data
      t.is(e.response?.status, 302)
      t.is(data, '')
    } else {
      t.fail('Expected an AxiosError.')
    }
  }
})
