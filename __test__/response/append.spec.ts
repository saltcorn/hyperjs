import test from 'ava'

import { Response } from '../../index'

test('append', (t) => {
  const res = new Response()
  res.append('Link', ['<http://localhost/>', '<http://localhost:3000/>'])
  res.append('Set-Cookie', 'foo=bar; Path=/; HttpOnly')
  res.append('Warning', '199 Miscellaneous warning')
  t.is(res.get('link'), '<http://localhost/>, <http://localhost:3000/>')
  t.is(res.get('set-cookie'), 'foo=bar; Path=/; HttpOnly')
  t.is(res.get('warning'), '199 Miscellaneous warning')
})
