import test from 'ava'

import { Response } from '../../index'

test('attachment - no value', (t) => {
  const res = new Response()
  res.attachment()
  t.is(res.get('content-disposition'), 'attachment')
})

test('attachment', (t) => {
  const res = new Response()
  res.attachment('path/to/logo.png')
  t.is(res.get('content-disposition'), 'attachment; filename="logo.png"')
  t.is(res.get('content-type'), 'image/png')
})
