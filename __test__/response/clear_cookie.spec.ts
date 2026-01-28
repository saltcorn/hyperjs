import test from 'ava'

import { Response } from '../../index'

test('clearCookie', (t) => {
  let res = new Response()

  res.clearCookie('rememberme', { expires: new Date('2026-01-04T05:45:09.535Z') })
  t.is(res.get('set-cookie'), 'rememberme=; Expires=Thu, 01 Jan 1970 00:00:00 GMT; Path=/')
})
