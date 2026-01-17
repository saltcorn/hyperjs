import test from 'ava'

import { Response } from '../index'

test('append', (t) => {
  const res = new Response()
  res.append('Link', ['<http://localhost/>', '<http://localhost:3000/>'])
  res.append('Set-Cookie', 'foo=bar; Path=/; HttpOnly')
  res.append('Warning', '199 Miscellaneous warning')
  t.is(res.get('link'), '<http://localhost/>, <http://localhost:3000/>')
  t.is(res.get('set-cookie'), 'foo=bar; Path=/; HttpOnly')
  t.is(res.get('warning'), '199 Miscellaneous warning')
})

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

test('type', (t) => {
  const res = new Response()

  res.type('.html')
  t.is(res.get('content-type'), 'text/html')

  res.type('html')
  t.is(res.get('content-type'), 'text/html')

  res.type('json')
  t.is(res.get('content-type'), 'application/json')

  res.type('application/json')
  t.is(res.get('content-type'), 'application/json')

  res.type('png')
  t.is(res.get('content-type'), 'image/png')

  res.type('wagwan')
  t.is(res.get('content-type'), 'application/octet-stream')
})

test('contentType', (t) => {
  const res = new Response()

  res.contentType('.html')
  t.is(res.get('content-type'), 'text/html')
})

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

test('end', (t) => {
  let res = new Response()
  res.end()
  t.assert(true)
})
