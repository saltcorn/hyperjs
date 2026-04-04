import test from 'ava'
import { ChildProcess } from 'node:child_process'
import axios from 'axios'

import * as server from '../server-setup.js'
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

test('/download', async (t) => {
  const res = await axios.get(`http://localhost:${port}/download/allow/.dotfile.html`)
  const data = res.data
  const contentDispositionHeader = res.headers['content-disposition']
  t.is(contentDispositionHeader, 'attachment; filename=".dotfile.html"')
  let dotfile_contents = readFileSync(path.join(__dirname, '../../public/.dotfile.html'), { encoding: 'utf-8' })
  t.is(data, dotfile_contents)
})
