import test from 'ava'

import { Response } from '../../hyperjs-core/index'

test('cookie - multiple', (t) => {
  const res = new Response()

  res.cookie('SID', '31d4d96e407aad42', { path: '/', secure: true, httpOnly: true })
  res.cookie('lang', 'en-US', { path: '/', domain: 'example.com' })
  t.is(res.get('set-cookie'), 'SID=31d4d96e407aad42; Path=/; Secure; HttpOnly, lang=en-US; Domain=example.com; Path=/')
})

test('cookie - default value encoding', (t) => {
  const res = new Response()

  res.cookie('some_cross_domain_cookie', 'http://mysubdomain.example.com', { domain: 'example.com' })
  t.is(
    res.get('set-cookie'),
    'some_cross_domain_cookie=http%3A%2F%2Fmysubdomain.example.com; Domain=example.com; Path=/',
  )
})

test('cookie - custom value encoding', (t) => {
  const res = new Response()

  res.cookie('some_cross_domain_cookie', 'http://mysubdomain.example.com', { domain: 'example.com', encode: String })
  t.is(res.get('set-cookie'), 'some_cross_domain_cookie=http://mysubdomain.example.com; Domain=example.com; Path=/')
})

test('cookie - expires', (t) => {
  let res = new Response()

  res.cookie('rememberme', '1', { expires: new Date('2026-01-04T05:45:09.535Z') })
  t.is(res.get('set-cookie'), 'rememberme=1; Expires=Sun, 04 Jan 2026 05:45:09 GMT; Path=/')

  res = new Response()

  res.cookie('rememberme', '1', { expires: new Date(new Date('2026-01-04T05:45:09.535Z').getTime() + 8 * 3600000) })
  t.is(res.get('set-cookie'), 'rememberme=1; Expires=Sun, 04 Jan 2026 13:45:09 GMT; Path=/')
})

test('clearCookie', (t) => {
  let res = new Response()

  res.clearCookie('rememberme', { expires: new Date('2026-01-04T05:45:09.535Z') })
  t.is(res.get('set-cookie'), 'rememberme=; Expires=Thu, 01 Jan 1970 00:00:00 GMT; Path=/')
})
