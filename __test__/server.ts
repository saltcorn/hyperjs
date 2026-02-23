import { ChildProcess, spawn } from 'node:child_process'
import { join } from 'node:path'
import kill from 'tree-kill'

const tsxPath = join(process.cwd(), 'node_modules', '.bin', 'tsx')

async function start(): Promise<{ process: ChildProcess; port: number }> {
  return await new Promise((resolve, reject) => {
    // Generate random port between 10000-20000
    const port = Math.floor(Math.random() * 10000) + 10000
    const serverPath = join(process.cwd(), 'server.ts')

    const serverApp = spawn(tsxPath, [serverPath], {
      env: { ...process.env, PORT: String(port) },
    })

    const onData = (data: Buffer | string) => {
      const output = data.toString()
      if (output.includes('Server listening')) {
        serverApp.stdout?.off('data', onData)
        resolve({ process: serverApp, port })
      }
    }

    serverApp.stdout?.on('data', onData)
    serverApp.stderr?.on('data', (data) => {
      console.error('Server error:', data.toString())
    })
    serverApp.on('error', reject)

    setTimeout(() => reject(new Error('Server startup timeout')), 10000)
  })
}

function stop(serverApp: ChildProcess) {
  if (serverApp.pid) {
    kill(serverApp.pid, 'SIGKILL', (err) => {
      if (err) console.error('Tree-kill failed:', err)
    })
  }
}

export { start, stop }
