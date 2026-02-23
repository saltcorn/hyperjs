import { ChildProcess, spawn } from 'node:child_process'
import { join } from 'node:path'

// Use a Map to track controllers if you run tests in parallel
const controllers = new Map<number, AbortController>()

async function start(): Promise<{ process: ChildProcess; port: number }> {
  const controller = new AbortController()
  const { signal } = controller

  return await new Promise((resolve, reject) => {
    const port = Math.floor(Math.random() * 10000) + 10000
    const serverPath = join(process.cwd(), 'server.ts')
    const tsxPath = join(process.cwd(), 'node_modules', '.bin', 'tsx')

    const serverApp = spawn(tsxPath, [serverPath], {
      env: { ...process.env, PORT: String(port) },
      signal, // Attach the abort signal here
    })

    controllers.set(serverApp.pid!, controller)

    const onData = (data: Buffer | string) => {
      if (data.toString().includes('Server listening')) {
        serverApp.stdout?.off('data', onData)
        resolve({ process: serverApp, port })
      }
    }

    serverApp.stdout?.on('data', onData)

    // Handle the abort error specifically so it doesn't crash the runner
    serverApp.on('error', (err) => {
      if (err.name === 'AbortError') return
      reject(err)
    })

    setTimeout(() => {
      controller.abort()
      reject(new Error('Server startup timeout'))
    }, 10000)
  })
}

function stop(serverApp: ChildProcess) {
  const controller = controllers.get(serverApp.pid!)
  if (controller) {
    controller.abort() // This kills the process and its immediate children
    controllers.delete(serverApp.pid!)
  } else {
    serverApp.kill('SIGKILL') // Fallback
  }
}

export { start, stop }
