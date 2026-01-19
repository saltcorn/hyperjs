import test from 'ava'

import { Request } from '../index'

test('accepts', (t) => {
  const req = new Request()
  t.is(req.accepts('text/html'), 'text/html')
})
