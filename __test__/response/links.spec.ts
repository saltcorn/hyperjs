import test from 'ava'

import { Response } from '../../hyperjs-core/index'

test('links', (t) => {
  let res = new Response()
  res.links({
    next: 'http://api.example.com/users?page=2',
    last: 'http://api.example.com/users?page=5',
  })
  t.is(
    res.get('link'),
    `<http://api.example.com/users?page=2>; rel="next", <http://api.example.com/users?page=5>; rel="last"`,
  )

  res.links({
    pages: ['http://api.example.com/users?page=1', 'http://api.example.com/users?page=2'],
  })
  t.is(
    res.get('link'),
    `<http://api.example.com/users?page=2>; rel="next",` +
      ` <http://api.example.com/users?page=5>; rel="last",` +
      ` <http://api.example.com/users?page=1, http://api.example.com/users?page=2>; rel="pages"`,
  )
})
