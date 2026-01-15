import { Server, Request, Response, StatusCode, TextMiddleware } from './index.js'

// ============================================================================
// SETUP: Create router and register routes
// ============================================================================

// Create app with router
const app = new Server()

// ============================================================================
// ROUTE DEFINITIONS
// ============================================================================

// Simple synchronous route
app.get('/health', (_request: Request, res: Response) => {
  console.log('JS: GET /health callback called.')
  res.sendStatus(StatusCode.ok())
})

// GET | Support URL parameters
app.get('/users/{user_id}', async (req: Request, res: Response) => {
  // Get URL parameters for the request object
  console.log('Request:', req)
  const params = req.params
  console.log('URL parameters:', params)

  res.status(200).json(params)
})

// POST Echo
app.post('/echo', async (_request: Request, res: Response) => {
  console.log('JS: POST /echo callback called.')
  res.sendStatus(200)
})

// Async route with delay
app.get('/users', async (_request: Request, res: Response) => {
  console.log('JS: GET /users callback called.')
  // Simulate async database query
  await new Promise((resolve) => setTimeout(resolve, 100))

  const users = [
    { id: 1, name: 'Alice' },
    { id: 2, name: 'Bob' },
  ]

  res.status(200).json(users)
})

// POST endpoint
app.post('/users', async (_request: Request, res: Response) => {
  console.log('JS: POST /users callback called.')
  // In a real app, you'd parse the request body here
  const newUser = { id: 3, name: 'Charlie' }

  res.status(201).json(newUser)
})

// Route with error handling
app.get('/error', async (_request: Request) => {
  console.log('JS: GET /error callback called.')
  throw new Error('Intentional error for testing')
})

// ============================================================================
// MIDDLEWARE DEFINITIONS
// ============================================================================

// // Middleware that continues the chain
app.use('/health', async (_req: Request, _res: Response) => {
  console.log('JS: Logging middleware')
  return true // Continue to next middleware
})

// Text middleware
const textMiddleware = new TextMiddleware({})
app.use('/health', (req: Request, res: Response) => textMiddleware.run(req, res))

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
