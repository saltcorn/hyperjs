import test from 'ava'

import { Response } from '../../hyperjs-core/index'

test('type', (t) => {
  const res = new Response()

  res.type('.html')
  t.is(res.get('content-type'), 'text/html')

  res.type('html')
  t.is(res.get('content-type'), 'text/html')

  res.type('json')
  t.is(res.get('content-type'), 'application/json')

  res.type('application/json')
  t.is(res.get('content-type'), 'application/json')

  res.type('png')
  t.is(res.get('content-type'), 'image/png')

  res.type('wagwan')
  t.is(res.get('content-type'), 'application/octet-stream')
})

test('contentType', (t) => {
  const res = new Response()

  res.contentType('.html')
  t.is(res.get('content-type'), 'text/html')
})
