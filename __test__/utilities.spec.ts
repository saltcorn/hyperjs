import test from 'ava'

import { serializeNapiObject } from '../hyperjs-core/index'

test('serializeNapiObject', (t) => {
  t.is(serializeNapiObject({}), '{}')
  t.is(serializeNapiObject({ a: 'a', b: 'b' }), '{"a":"a","b":"b"}')
})
