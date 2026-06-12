import {
  Server as RsServer,
  TcpServerListenOptions,
  IpcServerListenOptions,
  ListenCallbackFn,
} from '../hyperjs-core/index'
import type { IRouter, IRouterMatcher } from 'express-serve-static-core'

type IRouterPartial = {
  get: IRouterMatcher<IRouterPartial, 'get'>
}

class HyperJs implements IRouterPartial {
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

  // get
  // 1. (string | regex, ...)
  //    ... = one or more
  //        - (req, res, next): unknown                          | RequestHandler
  //                                                             | Request.params has properties extracted from path
  //
  // 2. (string | regex, ...)
  //    ... = one or more
  //        - (req, res, next): unknown                          | RequestHandler
  //                                                             | Request.params has properties extracted from path
  //
  //        - (err, req, res, next)                              | ErrorRequestHandler
  //                                                             | Request.params has properties extracted from path
  //
  //        - (RequestHandler | ErrorRequestHandler)[]           | An Array containing a mixture of RequestHandlers and ErrorRequestHandlers
  //                                                             | Request.params has properties extracted from path
  //
  // 3. (string | regex | (string | regex)[], ...)
  //        - (req, res, next): unknown                          | RequestHandler
  //                                                             | Request.params = { number: string; string: string; string: string[] }
  //
  // 4. (string | regex | (string | regex)[], ...)
  //    ... = one or more
  //        - (req, res, next): unknown                          | RequestHandler
  //                                                             | Request.params = { number: string; string: string; string: string[] }
  //
  //        - (err, req, res, next)                              | ErrorRequestHandler
  //                                                             | Request.params = { number: string; string: string; string: string[] }
  //
  //        - (RequestHandler | ErrorRequestHandler)[]           | An Array containing a mixture of RequestHandlers and ErrorRequestHandlers
  //                                                             | Request.params = { number: string; string: string; string: string[] }
  //
  // 5. (string | regex | (string | regex)[], Application)
  get(...args: any[]): IRouterPartial {
    // Here are three tricky things I need to be mindful of:
    // 1. Flattening Arrays:
    //    In cases 2, 4, and 5, users can pass nested arrays of middleware
    //    handlers. I need to flatten them into a single-level list of
    //    functions.
    //
    // 2. Identifying Error Handlers:
    //    Normal handlers take 3 arguments (req, res, next). Error handlers
    //    take 4 arguments (err, req, res, next). JavaScript lets me check
    //    this at runtime using fn.length.
    //
    // 3. Handling Sub-Applications:
    //    Case 5 passes a sub-app instead of callback functions.
    const path = args[0]
    const rawHandlers = args.slice(1)

    // 1. Flatten any nested arrays from signatures 2 and 4
    const flattenedHandlers = rawHandlers.flat(Infinity)

    // 2. Check if signature 5 was used (mounting a sub-application)
    if (flattenedHandlers.length === 1 && typeof flattenedHandlers[0] === 'object') {
      const subApp = flattenedHandlers[0]
      // Pass the sub-app mounting logic to your Rust backend
      // this.rsServer.mountSubApp(path, subApp);
      return this
    }

    // 3. Separate standard handlers from error handlers using fn.length
    const normalHandlers = []
    const errorHandlers = []

    for (const handler of flattenedHandlers) {
      if (typeof handler === 'function') {
        if (handler.length === 4) {
          // (err, req, res, next) -> ErrorRequestHandler
          errorHandlers.push(handler)
        } else {
          // (req, res, next) -> RequestHandler
          normalHandlers.push(handler)
        }
      }
    }

    // 4. Pass the cleaned data to your Rust Napi bindings
    // Example:
    // this.rsServer.registerGetRoute(path, normalHandlers, errorHandlers);

    return this
  }
}

export { HyperJs }
