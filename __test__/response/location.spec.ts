import test from 'ava'

import { Response } from '../../index'

test('location', (t) => {
  let res = new Response()
  res.location('/foo/bar')
  t.is(res.get('location'), '/foo/bar')

  res.location('../login')
  t.is(res.get('location'), '../login')

  res.location('http://example.com')
  t.is(res.get('location'), 'http://example.com')
})
