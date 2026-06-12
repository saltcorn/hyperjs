import {
  Server as RsServer,
  TcpServerListenOptions,
  IpcServerListenOptions,
  ListenCallbackFn,
} from '../hyperjs-core/index'

class HyperJs {
  /**
   * Listen for connections.
   */
  listen(port: number, hostname: string, backlog: number, callback?: ListenCallbackFn): RsServer
  listen(port: number, hostname: string, callback?: ListenCallbackFn): RsServer
  listen(port: number, callback?: ListenCallbackFn): RsServer
  listen(callback?: ListenCallbackFn): RsServer
  listen(path: string, callback?: ListenCallbackFn): RsServer
  listen(handle: any, listeningListener?: ListenCallbackFn): RsServer

  listen(): RsServer {
    const args = Array.prototype.slice.call(arguments)
    const done: ListenCallbackFn = typeof args[args.length - 1] === 'function' ? args[args.length - 1] : () => {}
    let rsServer: RsServer = new RsServer()

    // (port, hostname, backlog[, callback])
    if (typeof args[0] === 'number' && typeof args[1] === 'string' && typeof args[2] === 'number') {
      const options: TcpServerListenOptions = {
        port: args[0],
        host: args[1],
        backlog: args[2],
      }
      rsServer.listenTcp(options, done)
    }
    // (port, hostname[, callback])
    else if (typeof args[0] === 'number' && typeof args[1] === 'string') {
      const options: TcpServerListenOptions = {
        port: args[0],
        host: args[1],
      }
      rsServer.listenTcp(options, done)
    }
    // (port[, callback])
    else if (typeof args[0] === 'number') {
      const options: TcpServerListenOptions = {
        port: args[0],
      }
      rsServer.listenTcp(options, done)
    }
    // (path[, callback])
    else if (typeof args[0] === 'string') {
      const options: IpcServerListenOptions = {
        path: args[0],
      }
      rsServer.listenIpc(options, done)
    }
    // ([callback])
    else if (done) {
      rsServer.listenTcp({}, done)
    }
    // (handle, listeningListener)
    else {
      throw new Error('Listening on handle is not supported in this implementation')
    }

    return rsServer
  }
}

export { HyperJs }
