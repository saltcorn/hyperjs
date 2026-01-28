import test from 'ava'

import { Response } from '../../index'

test('vary', (t) => {
  let res = new Response()

  res.set({
    'user-agent':
      'Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Mobile Safari/537.36',
  })
  res.vary('user-agent')
  t.is(res.get('vary'), 'user-agent')
})
