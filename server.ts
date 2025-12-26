import { Router, Server, Request, Response, Body, StatusCode, completeRequest, RequestContext } from './index.js'

// ============================================================================
// SETUP: Create router and register routes
// ============================================================================

const router = new Router()
const handlers = new Map()

/**
 * Register a route with its handler
 * @param {string} path - The route path
 * @param {Function} handler - Async or sync function that returns a Response
 * @param {string|null} method - Optional HTTP method
 */
function register(path: string, handler: Function, method: string | null = null) {
  router.addRoute(path, method)
  const key = method ? `${method}:${path}` : path
  handlers.set(key, handler)
}

// ============================================================================
// ROUTE DEFINITIONS
// ============================================================================

// Simple synchronous route
register('/health', (_request: Request) => {
  console.log('JS: GET /health callback called.')
  return Response.builder().status(StatusCode.ok()).body(Body.string('OK'))
})

// Async route with delay
register(
  '/users',
  async (_request: Request) => {
    console.log('JS: GET /users callback called.')
    // Simulate async database query
    await new Promise((resolve) => setTimeout(resolve, 100))

    const users = [
      { id: 1, name: 'Alice' },
      { id: 2, name: 'Bob' },
    ]

    let builder = Response.builder()
    builder = builder.status(StatusCode.ok())
    const response = builder.body(Body.string(JSON.stringify(users)))
    return response
  },
  'GET',
)

// POST endpoint
register(
  '/users',
  async (_request: Request) => {
    console.log('JS: POST /users callback called.')
    // In a real app, you'd parse the request body here
    const newUser = { id: 3, name: 'Charlie' }

    let builder = Response.builder()
    builder = builder.status(StatusCode.created())
    const response = builder.body(Body.string(JSON.stringify(newUser)))
    return response
  },
  'POST',
)

// Route with error handling
register('/error', async (_request: Request) => {
  console.log('JS: GET /error callback called.')
  throw new Error('Intentional error for testing')
})

// ============================================================================
// REQUEST HANDLER
// ============================================================================

/**
 * Main request handler that the Rust server will call
 * This function is invoked via ThreadsafeFunction for each incoming request
 */
async function handleRequest(ctx: RequestContext) {
  console.log('JS: handleRequest called.')
  const { request, requestId } = ctx

  try {
    const path = request.uri()
    const method = request.method().toString()

    console.log(`[${requestId}] ${method} ${path}`)

    let response

    // Check if route exists in router
    if (router.hasRoute(path)) {
      // Try method-specific handler first, then fallback to path-only handler
      const methodKey = `${method}:${path}`
      const handler = handlers.get(methodKey) || handlers.get(path)

      if (handler) {
        try {
          // Call the handler (might be async)
          response = await handler(request)
        } catch (error: any) {
          console.error(`[${requestId}] Handler error:`, error)
          response = Response.builder()
          response = response.status(StatusCode.internalServerError())
          response = response.body(Body.string(`Internal Server Error: ${error.message}`))
        }
      } else {
        // Route exists but no handler
        response = Response.builder()
        response = response.status(StatusCode.notImplemented())
        response = response.body(Body.string(`Handler not implemented for: ${path}`))
      }
    } else {
      // No route found
      response = Response.builder()
      response = response.status(StatusCode.notFound())
      response = response.body(Body.string(`Not Found: ${path}`))
    }

    // Send response back to Rust
    completeRequest(requestId, response)
    console.log(`[${requestId}] Response sent`)
  } catch (error) {
    console.error(`[${requestId}] Fatal error:`, error)

    // Try to send error response
    try {
      const errorResponse = Response.builder()
        .status(StatusCode.internalServerError())
        .body(Body.string('Internal Server Error'))
      completeRequest(requestId, errorResponse)
    } catch (e) {
      console.error(`[${requestId}] Failed to send error response:`, e)
    }
  }
}

// ============================================================================
// SERVER STARTUP
// ============================================================================

async function startServer() {
  try {
    // Create server with router
    const server = new Server(router)

    // Set the request handler
    server.setHandler(handleRequest)

    // Start listening
    const addr = '127.0.0.1:8080'
    console.log(`Starting server on ${addr}...`)
    console.log(`Registered routes: ${router.getRoutes().join(', ')}`)

    // This will block and run the server
    await server.listen(addr)
  } catch (error) {
    console.error('Server error:', error)
    process.exit(1)
  }
}

// ============================================================================
// GRACEFUL SHUTDOWN
// ============================================================================

process.on('SIGINT', () => {
  console.log('\nShutting down gracefully...')
  process.exit(0)
})

process.on('SIGTERM', () => {
  console.log('\nShutting down gracefully...')
  process.exit(0)
})

// ============================================================================
// START
// ============================================================================

startServer()
