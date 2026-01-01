import test from 'ava'

import { Response } from '../index'

test('Response.append', (t) => {
  const res = new Response()
  res.append('Link', ['<http://localhost/>', '<http://localhost:3000/>'])
  res.append('Set-Cookie', 'foo=bar; Path=/; HttpOnly')
  res.append('Warning', '199 Miscellaneous warning')
  const headers: Record<string, string> = { ...res.headers() }
  t.is(headers['link'], '<http://localhost/>, <http://localhost:3000/>')
  t.is(headers['set-cookie'], 'foo=bar; Path=/; HttpOnly')
  t.is(headers['warning'], '199 Miscellaneous warning')
})
