import { Server, Request, Response, Body, StatusCode } from './index.js'

// ============================================================================
// SETUP: Create router and register routes
// ============================================================================

// Create app with router
const app = new Server()

// ============================================================================
// ROUTE DEFINITIONS
// ============================================================================

// Simple synchronous route
app.get('/health', async (_request: Request) => {
  console.log('JS: GET /health callback called.')
  return Response.builder().status(StatusCode.ok()).body(Body.string('OK'))
})

// Async route with delay
app.get('/users', async (_request: Request) => {
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
})

// POST endpoint
app.post('/users', async (_request: Request) => {
  console.log('JS: POST /users callback called.')
  // In a real app, you'd parse the request body here
  const newUser = { id: 3, name: 'Charlie' }

  let builder = Response.builder()
  builder = builder.status(StatusCode.created())
  const response = builder.body(Body.string(JSON.stringify(newUser)))
  return response
})

// Route with error handling
app.get('/error', async (_request: Request) => {
  console.log('JS: GET /error callback called.')
  throw new Error('Intentional error for testing')
})

// ============================================================================
// SERVER STARTUP
// ============================================================================

async function startServer() {
  try {
    // Start listening
    const addr = '127.0.0.1:8080'
    console.log(`Starting app on ${addr}...`)
    // console.log(`Registered routes: ${router.getRoutes().join(', ')}`)

    // This will block and run the app
    await app.listen(addr)
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
