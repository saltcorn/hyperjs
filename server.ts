import { Server, Request, Response, StatusCode, TextMiddleware, JsTextOptions } from './index.js'

// ============================================================================
// SETUP: Create router and register routes
// ============================================================================

// Create app with router
const app = new Server()

// ============================================================================
// ROUTE DEFINITIONS
// ============================================================================

// Simple synchronous route
app.get('/health', (_req: Request, res: Response) => {
  console.log('JS: GET /health callback called.')
  res.sendStatus(StatusCode.ok())
})

// Test Response.end
app.get('/end', (_req: Request, res: Response) => {
  console.log('JS: GET /end callback called.')
  res.status(StatusCode.ok()).end()
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
app.post('/echo', async (req: Request, res: Response) => {
  console.log('JS: POST /echo callback called.')
  if (typeof req.body === 'string') res.status(200).send(req.body)
  else res.sendStatus(200)
})

// Async route with delay
app.get('/users', async (_req: Request, res: Response) => {
  console.log('JS: GET /users callback called.')
  // Simulate async database query
  await new Promise((resolve) => setTimeout(resolve, 100))

  const users = [
    { id: 1, name: 'Alice' },
    { id: 2, name: 'Bob' },
  ]

  res.status(200).json(users)
})

app.get('/format', async (_req: Request, res: Response) => {
  console.log('JS: GET /format callback called.')
  res.format({
    'text/plain'() {
      console.log('JS: text/plain handler executed.')
      res.send('hey')
    },

    'text/html'() {
      console.log('JS: text/htmp handler executed.')
      res.send('<p>hey</p>')
    },

    'application/json'() {
      console.log('JS: application/json handler executed.')
      res.send({ message: 'hey' })
    },

    default() {
      // log the request and respond with 406
      res.status(200).send('Not Acceptable')
    },
  })
})

// POST endpoint
app.post('/users', async (_req: Request, res: Response) => {
  console.log('JS: POST /users callback called.')
  // In a real app, you'd parse the request body here
  const newUser = { id: 3, name: 'Charlie' }

  res.status(201).json(newUser)
})

// Route with error handling
app.get('/error', async (_req: Request, _res: Response) => {
  console.log('JS: GET /error callback called.')
  throw new Error('Intentional error for testing')
})

// Request.method() test routes
app.get('/method', async (req: Request, res: Response) => {
  console.log('JS: GET /method callback called.')
  res.status(200).send(req.method)
})
app.post('/method', async (req: Request, res: Response) => {
  console.log('JS: POST /method callback called.')
  res.status(200).send(req.method)
})
app.put('/method', async (req: Request, res: Response) => {
  console.log('JS: PUT /method callback called.')
  res.status(200).send(req.method)
})
app.delete('/method', async (req: Request, res: Response) => {
  console.log('JS: DELETE /method callback called.')
  res.status(200).send(req.method)
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
const textMiddleware = new TextMiddleware(new JsTextOptions({}))
app.use('/echo', (req: Request, res: Response) => textMiddleware.run(req, res))

// ============================================================================
// SERVER STARTUP
// ============================================================================

async function startServer() {
  try {
    // Get port from environment variable or use a random port
    const port = process.env.PORT || 8080
    const addr = `127.0.0.1:${port}`

    console.log(`Starting app on ${addr}...`)
    // console.log(`Registered routes: ${router.getRoutes().join(', ')}`)

    // This will block and run the app
    await app.listen(addr)

    // Log this exact message so tests can detect when server is ready
    console.log('Server listening')
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
