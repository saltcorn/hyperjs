import test from 'ava'
import { ChildProcess } from 'node:child_process'
import axios, { isAxiosError } from 'axios'

import * as server from '../server.js'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'url'
import { dirname } from 'path'
import path from 'node:path'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

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

test('/send-file - dotfiles = allow', async (t) => {
  const res = await axios.get(`http://localhost:${port}/send-file/allow/.dotfile.html`)
  const data = res.data
  let dotfile_contents = readFileSync(path.join(__dirname, '../../public/.dotfile.html'), { encoding: 'utf-8' })
  t.is(data, dotfile_contents)
})

test('/send-file - dotfiles = deny', async (t) => {
  try {
    await axios.get(`http://localhost:${port}/send-file/deny/.dotfile.html`)
    t.fail('Expected request to fail.')
  } catch (e) {
    if (isAxiosError(e)) {
      t.is(e.response?.status, 403)
    } else {
      t.fail('Expected an AxiosError.')
    }
  }
})

test('/send-file - dotfiles = ignore', async (t) => {
  try {
    await axios.get(`http://localhost:${port}/send-file/ignore/.dotfile.html`)
    t.fail('Expected request to fail.')
  } catch (e) {
    if (isAxiosError(e)) {
      t.is(e.response?.status, 404)
    } else {
      t.fail('Expected an AxiosError.')
    }
  }
})

test('/folder - dir default index', async (t) => {
  const res = await axios.get(`http://localhost:${port}/folder`)
  const data = res.data
  let index_file_contents = readFileSync(path.join(__dirname, '../../public/index.html'), { encoding: 'utf-8' })
  t.is(data, index_file_contents)
})
