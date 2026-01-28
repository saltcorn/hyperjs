import test from 'ava'

import { Response } from '../../index'

test('set - (string, string)', (t) => {
  let res = new Response()

  res.set('content-type', 'application/json')
  t.is(res.get('content-type'), 'application/json')
})

test('set - (object)', (t) => {
  let res = new Response()

  res.set({
    'content-type': 'application/json',
    'content-disposition': 'attachment',
    Link: ['<http://localhost/>', '<http://localhost:3000/>'],
    'Set-Cookie': 'foo=bar; Path=/; HttpOnly',
    Warning: '199 Miscellaneous warning',
  })
  t.is(res.get('content-type'), 'application/json')
  t.is(res.get('content-disposition'), 'attachment')
  t.is(res.get('link'), '<http://localhost/>, <http://localhost:3000/>')
  t.is(res.get('set-cookie'), 'foo=bar; Path=/; HttpOnly')
  t.is(res.get('warning'), '199 Miscellaneous warning')
})
