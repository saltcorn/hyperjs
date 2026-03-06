import {
  Server,
  Request,
  Response,
  StatusCode,
  TextMiddleware,
  JsonMiddleware,
  RawMiddleware,
  StaticMiddleware,
  FileStat,
  UrlencodedMiddleware,
  CookieParserMiddleware,
} from './index.js'
import path from 'path'
import process from 'process'

const __dirname = path.dirname(__filename)

// ============================================================================
// SETUP: Create router and register routes
// ============================================================================

// Create app with router
const app = new Server()

// ============================================================================
// ROUTE DEFINITIONS
// ============================================================================

// // Middleware that continues the chain
app.use('/health', async (_req: Request, _res: Response) => {
  console.log('JS: Logging middleware')
  return true // Continue to next middleware
})

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

// Text middleware
const textMiddleware = new TextMiddleware({
  limit: '100mb',
})
app.use('/echo', (req: Request, res: Response) => textMiddleware.run(req, res))

// POST Echo
app.post('/echo', async (req: Request, res: Response) => {
  console.log('JS: POST /echo callback called.')
  if (typeof req.body === 'string') res.status(200).send(req.body)
  else if (typeof req.body === 'undefined') res.sendStatus(200)
  else res.status(500).send(`Expected string, found '${typeof req.body}'`)
})

// JSON middleware
const jsonMiddleware = new JsonMiddleware({
  strict: true,
})
app.use('/json-echo', (req: Request, res: Response) => jsonMiddleware.run(req, res))

// POST Echo
app.post('/json-echo', async (req: Request, res: Response) => {
  console.log('JS: POST /json-echo callback called.')
  if (typeof req.body === 'object') res.status(200).send(req.body)
  else if (typeof req.body === 'undefined') res.sendStatus(200)
  else res.status(500).send(`Expected object, found '${typeof req.body}'`)
})

// RAW middleware
const rawMiddleware = new RawMiddleware()
app.use('/raw-echo', (req: Request, res: Response) => rawMiddleware.run(req, res))

// RAW Echo
app.post('/raw-echo', async (req: Request, res: Response) => {
  console.log('JS: POST /raw-echo callback called.')
  let data = req.body
  if (data instanceof Buffer) res.status(200).send(data)
  else if (typeof req.body === 'undefined') res.sendStatus(200)
  else res.status(500).send(`Expected Buffer, found '${typeof req.body}'`)
})

// Urlencoded middleware
const urlencodedMiddleware = new UrlencodedMiddleware({
  extended: true,
})
app.use('/urlencoded', (req: Request, res: Response) => urlencodedMiddleware.run(req, res))

// POST Echo
app.post('/urlencoded', async (req: Request, res: Response) => {
  console.log('JS: POST /urlencoded callback called.')
  let data = req.body
  if (typeof data === 'object') res.status(200).send(data)
  else if (typeof req.body === 'undefined') res.sendStatus(200)
  else res.status(500).send(`Expected Buffer, found '${typeof req.body}'`)
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

// Redirection
app.get('/redirect', async (_req: Request, res: Response) => {
  console.log('JS: GET /redirect callback called.')
  res.redirect('https://example.com')
})

// Range
app.get('/range', async (req: Request, res: Response) => {
  console.log('JS: GET /range callback called.')
  res.json(req.range(1000, { combine: true }))
})

// Send a file
app.get('/send-file/{dotfiles}/{name}', async (req: Request, res: Response) => {
  const options = {
    root: path.join(__dirname, 'public'),
    dotfiles: (req.params as any).dotfiles,
    extensions: ['html', 'htm'],
  }

  console.log(options)

  const fileName = (req.params as any).name
  console.log('JS: fileName =', fileName)
  await res.sendFile(fileName, options)
})

// Respond with index.html
app.get('/folder', async (_req: Request, res: Response) => {
  const options = {
    root: path.join(__dirname, 'public'),
  }

  await res.sendFile('/', options)
})

// Download a file
app.get('/download/{dotfiles}/{name}', async (req: Request, res: Response) => {
  const options = {
    root: path.join(__dirname, 'public'),
    dotfiles: (req.params as any).dotfiles,
    extensions: ['html', 'htm'],
  }

  console.log(options)

  const fileName = (req.params as any).name
  console.log('JS: fileName =', fileName)
  await res.download(fileName, options)
})

// Cookie testing endpoints
// set-cookie
const cookieParserMiddleware = new CookieParserMiddleware(null, {})
app.use(null, (req: Request, res: Response) => cookieParserMiddleware.run(req, res))

app.get('/cookie/show', async (req: Request, res: Response) => {
  console.log(req.cookies, typeof req.cookies)
  res.json(req.cookies as any)
})

// ============================================================================
// AFTER-ROUTES APPLICATION-WIDE MIDDLEWARE DEFINITIONS
// ============================================================================

// Static middleware
const staticMiddleware = new StaticMiddleware('public', {
  dotfiles: 'ignore',
  etag: false,
  extensions: ['htm', 'html'],
  index: false,
  redirect: false,
  fallthrough: true,
  setHeaders(res: Response, _path: string, _stat: FileStat) {
    res.set('x-timestamp', Date.now().toString())
  },
})
app.use(null, (req: Request, res: Response) => staticMiddleware.run(req, res))

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
