import { ChildProcess, spawn } from 'node:child_process'
import { join } from 'node:path'

async function start(): Promise<{ process: ChildProcess; port: number }> {
  return await new Promise((resolve, reject) => {
    // Generate random port between 10000-20000
    const port = Math.floor(Math.random() * 10000) + 10000
    const serverPath = join(process.cwd(), 'server.ts')

    const serverApp = spawn('node', [serverPath], {
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
  serverApp?.kill('SIGKILL')
}

class Port {
  private _value: number | undefined

  get value(): Promise<number> {
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        clearInterval(intervalHandler)
        reject(new Error('Timeout waiting for port'))
      }, 10000) // 10 second timeout

      const intervalHandler = setInterval(() => {
        if (this._value !== undefined) {
          clearInterval(intervalHandler)
          clearTimeout(timeout)
          resolve(this._value)
        }
      }, 100)
    })
  }

  set value(value: number) {
    this._value = value
  }
}

// const portPromise: Promise<number> = new Promise((resolve, reject) => {
//   const timeout = setTimeout(() => {
//     clearInterval(intervalHandler)
//     reject(new Error('Timeout waiting for port'))
//   }, 10000) // 10 second timeout

//   const intervalHandler = setInterval(() => {
//     if (port !== undefined) {
//       clearInterval(intervalHandler)
//       clearTimeout(timeout)
//       resolve(port)
//     }
//   }, 100)
// })

export { start, stop, Port }
