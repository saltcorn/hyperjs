import { ChildProcess, spawn } from 'node:child_process'
import { join } from 'node:path'
import { pathToFileURL } from 'node:url'
import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)

async function start(): Promise<{ process: ChildProcess; port: number }> {
  const port = Math.floor(Math.random() * 10000) + 10000
  const serverPath = join(process.cwd(), 'server.ts')

  // Let Node find the correct path to the tsx package automatically
  const tsxEntry = require.resolve('tsx')
  const tsxUrl = pathToFileURL(tsxEntry).href

  return await new Promise((resolve, reject) => {
    const serverApp = spawn(process.execPath, ['--import', tsxUrl, '--no-warnings', serverPath], {
      env: { ...process.env, PORT: String(port) },
      windowsHide: true,
      // Use 'pipe' to ensure we can read the "Server listening" message
      stdio: ['ignore', 'pipe', 'pipe'],
    })

    const timeout = setTimeout(() => {
      serverApp.kill('SIGKILL')
      reject(new Error(`Server startup timeout on port ${port}`))
    }, 20000)

    const onData = (data: Buffer) => {
      if (data.toString().includes('Server listening')) {
        clearTimeout(timeout)
        serverApp.stdout?.off('data', onData)
        resolve({ process: serverApp, port })
      }
    }

    serverApp.stdout?.on('data', onData)
    serverApp.stderr?.on('data', (data) => console.error(`[Server Error]: ${data}`))

    serverApp.on('error', (err) => {
      clearTimeout(timeout)
      reject(err)
    })
  })
}

function stop(serverApp?: ChildProcess) {
  if (serverApp?.pid) {
    try {
      // Direct node process kill works on Windows/Linux when no .cmd wrapper is used
      serverApp.kill('SIGKILL')
    } catch (e) {
      // Already dead
    }
  }
}

export { start, stop }
